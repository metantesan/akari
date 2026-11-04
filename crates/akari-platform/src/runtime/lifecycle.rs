use super::*;

impl<A: Application> ApplicationHandler<A::Message> for Runtime<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.system_theme = akari_native::ThemeProvider::system_theme(
            &akari_native::current(),
            event_loop
                .system_theme()
                .map(|theme| theme == WindowTheme::Dark),
        );
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
            && let Err(e) = self.create_graphics()
        {
            self.fail(event_loop, e);
            return;
        }
        if first {
            let tasks = self.app.startup();
            self.tasks(tasks);
        }
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
                self.pointer = [p.x, p.y];
                self.dispatch(Event::PointerMove(p.x, p.y));
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
                match t.phase {
                    TouchPhase::Started if self.touch.is_none() => {
                        self.touch = Some((t.id, [p.x, p.y], false));
                        self.dispatch(Event::PointerDown(p.x, p.y));
                        self.position_cursor(p.x, p.y);
                    }
                    TouchPhase::Moved => {
                        if let Some((id, old, moved)) = self.touch
                            && id == t.id
                        {
                            let delta = old[1] - p.y;
                            let moved = moved || delta.abs() > 4.;
                            if moved {
                                self.dispatch(Event::Cancel);
                                self.dispatch(Event::Scroll(p.x, p.y, delta));
                            }
                            self.touch = Some((id, [p.x, p.y], moved));
                        }
                    }
                    TouchPhase::Ended => {
                        if let Some((id, _, moved)) = self.touch
                            && id == t.id
                        {
                            self.touch = None;
                            if !moved {
                                self.dispatch(Event::PointerUp(p.x, p.y));
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
            WindowEvent::Ime(Ime::Preedit(text, _)) => {
                self.ime = !text.is_empty();
                self.dispatch(Event::Composition(text));
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
                let draws = self.ui.draw();
                if let Some(g) = &mut self.graphics {
                    match g.surface.get_current_texture() {
                        wgpu_surface::CurrentSurfaceTexture::Success(frame)
                        | wgpu_surface::CurrentSurfaceTexture::Suboptimal(frame) => {
                            let view = frame.texture.create_view(&Default::default());
                            if let Err(e) = g.renderer.render(
                                &view,
                                [size.width, size.height],
                                scale as f32,
                                self.app.theme(),
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
                            if let Err(e) = self.create_graphics() {
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
