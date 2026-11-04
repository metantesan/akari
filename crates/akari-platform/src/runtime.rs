//! Winit lifecycle adapter. Public application code remains platform independent.
use super::keyboard::keyboard_event;
use akari_core::{Application, EditKey, Event, Task, Theme, ThemePreference, Ui};

mod lifecycle;
mod text_input;
use akari_renderer::Renderer;
use std::{sync::Arc, thread};
pub use winit;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::ModifiersState,
    window::{Window, WindowId},
};

pub fn run<A: Application>(app: A) -> Result<(), Box<dyn std::error::Error>> {
    run_with_event_loop(app, EventLoop::<A::Message>::with_user_event().build()?)
}
pub fn run_with_event_loop<A: Application>(
    app: A,
    event_loop: EventLoop<A::Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    let proxy = event_loop.create_proxy();
    let mut runtime = Runtime {
        app,
        proxy,
        ui: Ui::default(),
        window: None,
        graphics: None,
        pointer: [0.; 2],
        modifiers: ModifiersState::empty(),
        touch: None,
        ime: false,
        system_theme: akari_native::theme::SystemTheme::Unknown,
        error: None,
    };
    event_loop.run_app(&mut runtime)?;
    if let Some(error) = runtime.error {
        return Err(error.into());
    }
    Ok(())
}
struct Graphics {
    renderer: Renderer,
    surface: wgpu_surface::Surface<'static>,
    _instance: wgpu_surface::Instance,
}
// Re-export GPU types internally through the renderer to keep one wgpu version.
mod wgpu_surface {
    pub use akari_renderer::gpu::{CurrentSurfaceTexture, Instance, Surface};
}
struct Runtime<A: Application> {
    app: A,
    proxy: EventLoopProxy<A::Message>,
    ui: Ui<A::Message>,
    window: Option<Arc<Window>>,
    graphics: Option<Graphics>,
    pointer: [f32; 2],
    modifiers: ModifiersState,
    touch: Option<(u64, [f32; 2], bool)>,
    ime: bool,
    system_theme: akari_native::theme::SystemTheme,
    error: Option<String>,
}
impl<A: Application> Runtime<A> {
    fn active_theme(&self) -> Theme {
        if self.app.theme_preference() != ThemePreference::System {
            return self.app.theme();
        }
        match self.system_theme {
            akari_native::theme::SystemTheme::Dark => Theme::dark(),
            akari_native::theme::SystemTheme::Light => Theme::light(),
            akari_native::theme::SystemTheme::Unknown => self.app.theme(),
        }
    }

    fn tasks(&self, tasks: Vec<Task<A::Message>>) {
        for task in tasks {
            let proxy = self.proxy.clone();
            thread::spawn(move || {
                let message = (task.0)();
                let _ = proxy.send_event(message);
            });
        }
    }
    fn size(&self) -> [f32; 2] {
        self.window
            .as_ref()
            .map(|w| {
                let size = w.inner_size().to_logical::<f32>(w.scale_factor());
                [size.width, size.height]
            })
            .unwrap_or([0., 0.])
    }
    fn rebuild(&mut self) {
        let size = self.size();
        self.ui.theme = self.active_theme();
        self.ui.rebuild(self.app.view(size), size);
        if let Some(w) = &self.window {
            w.set_ime_allowed(self.ui.text_focused());
            if let Some(r) = self.ui.focused_bounds() {
                w.set_ime_cursor_area(LogicalPosition::new(r.x, r.y), LogicalSize::new(r.w, r.h));
            }
            w.request_redraw();
        }
    }
    fn dispatch(&mut self, event: Event) {
        let messages = self.ui.event(event);
        for m in messages {
            let tasks = self.app.update(m);
            self.tasks(tasks);
        }
        self.rebuild();
    }
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: String) {
        eprintln!("Akari: {error}");
        self.error = Some(error);
        event_loop.exit();
    }
    fn create_graphics(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let Some(window) = &self.window else {
            return Ok(());
        };
        let instance = wgpu_surface::Instance::new(akari_renderer::instance_descriptor());
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| e.to_string())?;
        let mut renderer = pollster::block_on(Renderer::new(&instance, Some(&surface)))?;
        let system_fonts = akari_native::fonts::from_event_loop(event_loop);
        renderer.set_default_font_family(system_fonts.family());
        for font in system_fonts.faces() {
            renderer.load_font(font.clone());
        }
        let adapter = renderer.adapter_info();
        self.app.renderer_info(akari_core::RendererInfo {
            adapter_name: adapter.name.clone(),
            backend: match adapter.backend {
                wgpu::Backend::Gl => "OpenGL / OpenGL ES",
                wgpu::Backend::Vulkan => "Vulkan",
                wgpu::Backend::Dx12 => "Direct3D 12",
                wgpu::Backend::Metal => "Metal",
                wgpu::Backend::BrowserWebGpu => "WebGPU",
                wgpu::Backend::Noop => "No-op",
            }
            .to_owned(),
            device_type: match adapter.device_type {
                wgpu::DeviceType::DiscreteGpu => "discrete GPU",
                wgpu::DeviceType::IntegratedGpu => "integrated GPU",
                wgpu::DeviceType::VirtualGpu => "virtual GPU",
                wgpu::DeviceType::Cpu => "CPU",
                wgpu::DeviceType::Other => "unknown device",
            }
            .to_owned(),
        });
        for font in self.app.fonts() {
            renderer.load_font(Arc::new(font.to_vec()));
        }
        let s = window.inner_size();
        renderer.configure(&surface, [s.width, s.height]);
        self.graphics = Some(Graphics {
            renderer,
            surface,
            _instance: instance,
        });
        Ok(())
    }
}
