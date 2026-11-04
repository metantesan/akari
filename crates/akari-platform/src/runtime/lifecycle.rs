use super::*;

impl<A: Application> ApplicationHandler<A::Message> for Runtime<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.system_theme = akari_native::theme::from_event_loop(event_loop);
        let first = self.window.is_none();
        if first {
            match event_loop.create_window(
                Window::default_attributes()
                    .with_title(self.app.title())
                    .with_inner_size(LogicalSize::new(780., 800.))
                    .with_min_inner_size(LogicalSize::new(320., 400.)),
            ) {
                Ok(w) => self.window = Some(Arc::new(w)),
                Err(e) => {
                    self.fail(event_loop, e.to_string());
                    return;
                }
            }
        }
        if self.graphics.is_none()
            && let Err(e) = self.create_graphics(event_loop)
        {
            self.fail(event_loop, e);
            return;
        }
        if first {
            let tasks = self.app.startup();
            self.tasks(tasks);
        }
        self.update_insets(event_loop);
        self.rebuild();
    }
    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.graphics = None;
        self.ime = false;
        self.dispatch(Event::FocusLost);
    }
    fn user_event(&mut self, _: &ActiveEventLoop, message: A::Message) {
        let tasks = self.app.update(message);
        self.tasks(tasks);
        self.rebuild();
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Android's content-rectangle callback is not exposed as a winit
        // WindowEvent. Query native insets after each event-loop turn so
        // system bars, cutout, and IME changes update layout and redraw.
        let previous = self.last_insets;
        self.update_insets(event_loop);
        let insets_changed = self.last_insets != previous;
        #[cfg(target_os = "android")]
        let theme_changed = {
            let system_theme = akari_native::theme::from_event_loop(event_loop);
            let changed = system_theme != self.system_theme;
            self.system_theme = system_theme;
            changed
        };
        #[cfg(not(target_os = "android"))]
        let theme_changed = false;
        if insets_changed || theme_changed {
            self.rebuild();
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.clone() else {
            return;
        };
        if window.id() != id {
            return;
        }
        let scale = window.scale_factor();
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                self.system_theme =
                    akari_native::theme::update_from_window(&window, self.system_theme);
                self.update_insets(event_loop);
                if let Some(g) = &mut self.graphics {
                    let s = window.inner_size();
                    g.renderer.configure(&g.surface, [s.width, s.height]);
                }
                self.rebuild();
            }
            WindowEvent::Focused(false) => {
                self.ime = false;
                self.dispatch(Event::FocusLost);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let p = position.to_logical::<f32>(scale);
                self.pointer = self.content_point(p.x, p.y);
                self.dispatch(Event::PointerMove(self.pointer[0], self.pointer[1]));
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                let [x, y] = self.pointer;
                self.dispatch(if state == ElementState::Pressed {
                    Event::PointerDown(x, y)
                } else {
                    Event::PointerUp(x, y)
                });
                if state == ElementState::Pressed {
                    self.position_cursor(x, y);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -y * 42.,
                    MouseScrollDelta::PixelDelta(p) => -p.y as f32 / scale as f32,
                };
                self.dispatch(Event::Scroll(self.pointer[0], self.pointer[1], d));
            }
            WindowEvent::Touch(t) => {
                let p = t.location.to_logical::<f32>(scale);
                let p = self.content_point(p.x, p.y);
                match t.phase {
                    TouchPhase::Started if self.touch.is_none() => {
                        self.touch = Some((t.id, p, false));
                        self.dispatch(Event::PointerDown(p[0], p[1]));
                        self.position_cursor(p[0], p[1]);
                    }
                    TouchPhase::Moved => {
                        if let Some((id, old, moved)) = self.touch
                            && id == t.id
                        {
                            let delta = old[1] - p[1];
                            let began_drag = !moved && delta.abs() > 4.;
                            let moved = moved || began_drag;
                            if moved {
                                if began_drag {
                                    self.dispatch(Event::Cancel);
                                }
                                self.dispatch(Event::Scroll(p[0], p[1], delta));
                            }
                            self.touch = Some((id, p, moved));
                        }
                    }
                    TouchPhase::Ended => {
                        if let Some((id, _, moved)) = self.touch
                            && id == t.id
                        {
                            self.touch = None;
                            if !moved {
                                self.dispatch(Event::PointerUp(p[0], p[1]));
                            }
                        }
                    }
                    TouchPhase::Cancelled => {
                        self.touch = None;
                        self.dispatch(Event::Cancel);
                    }
                    _ => {}
                }
            }
            WindowEvent::ModifiersChanged(m) => self.modifiers = m.state(),
            WindowEvent::Ime(Ime::Enabled) => self.ime = false,
            WindowEvent::Ime(Ime::Disabled) => {
                self.ime = false;
                self.dispatch(Event::Composition(String::new()));
            }
            WindowEvent::Ime(Ime::Preedit(text, cursor)) => {
                self.ime = !text.is_empty();
                self.dispatch(Event::CompositionWithCursor(text, cursor));
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                self.ime = false;
                self.dispatch(Event::Text(text));
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                if let Some(input) = keyboard_event(
                    &event.logical_key,
                    event.text.as_deref(),
                    self.modifiers,
                    self.ime,
                ) {
                    match input {
                        Event::Edit(EditKey::Left, shift) => self.move_text_cursor(false, shift),
                        Event::Edit(EditKey::Right, shift) => self.move_text_cursor(true, shift),
                        Event::Text(ref text) if text == " " && !self.ui.text_focused() => {
                            self.dispatch(Event::Activate)
                        }
                        other => self.dispatch(other),
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                if size.width == 0 || size.height == 0 {
                    return;
                }
                let mut draws = self.ui.draw();
                let insets = self.insets();
                Runtime::<A>::offset_draws(&mut draws, [insets.left, insets.top]);
                let theme = self.active_theme();
                if let Some(g) = &mut self.graphics {
                    match g.surface.get_current_texture() {
                        wgpu_surface::CurrentSurfaceTexture::Success(frame)
                        | wgpu_surface::CurrentSurfaceTexture::Suboptimal(frame) => {
                            let view = frame.texture.create_view(&Default::default());
                            if let Err(e) = g.renderer.render(
                                &view,
                                [size.width, size.height],
                                scale as f32,
                                theme,
                                &draws,
                            ) {
                                self.fail(event_loop, e);
                                return;
                            }
                            window.pre_present_notify();
                            g.renderer.queue.present(frame);
                        }
                        wgpu_surface::CurrentSurfaceTexture::Outdated => {
                            g.renderer.configure(&g.surface, [size.width, size.height]);
                            window.request_redraw();
                        }
                        wgpu_surface::CurrentSurfaceTexture::Timeout => window.request_redraw(),
                        wgpu_surface::CurrentSurfaceTexture::Occluded => {}
                        wgpu_surface::CurrentSurfaceTexture::Lost => {
                            self.graphics = None;
                            if let Err(e) = self.create_graphics(event_loop) {
                                self.fail(event_loop, e)
                            } else {
                                window.request_redraw();
                            }
                        }
                        wgpu_surface::CurrentSurfaceTexture::Validation => {
                            self.fail(event_loop, "Surface validation failed".into())
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
