use akari::{Draw, SimpleComponent, Ui};
use akari_showcase::{Message, Page, Showcase};
use std::{env, fs};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let args = env::args().collect::<Vec<_>>();
    let output_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("/tmp/akari-showcase.ppm");
    let width = args
        .get(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(900_u32);
    let height = args
        .get(3)
        .and_then(|value| value.parse().ok())
        .unwrap_or(900_u32);
    let mut app = Showcase::default();
    match args.get(4).map(String::as_str) {
        Some("layout") => app.update(Message::Navigate(Page::Layout)),
        Some("media") => app.update(Message::Navigate(Page::Media)),
        _ => Vec::new(),
    };
    let instance = akari_renderer::gpu::Instance::new(akari_renderer::instance_descriptor());
    let mut renderer = pollster::block_on(akari_renderer::Renderer::new(&instance, None))
        .map_err(|error| eyre::eyre!(error))?;
    let adapter = renderer.adapter_info();
    app.renderer_info(akari::RendererInfo {
        adapter_name: adapter.name.clone(),
        backend: match adapter.backend {
            akari_renderer::gpu::Backend::Gl => "OpenGL / OpenGL ES",
            akari_renderer::gpu::Backend::Vulkan => "Vulkan",
            akari_renderer::gpu::Backend::Dx12 => "Direct3D 12",
            akari_renderer::gpu::Backend::Metal => "Metal",
            akari_renderer::gpu::Backend::BrowserWebGpu => "WebGPU",
            akari_renderer::gpu::Backend::Noop => "No-op",
        }
        .to_owned(),
        device_type: format!("{:?}", adapter.device_type),
    });
    let mut ui = Ui::default();
    ui.theme = app.theme();
    ui.rebuild(
        app.view([width as f32, height as f32]),
        [width as f32, height as f32],
    );
    if args.get(5).is_some_and(|arg| arg == "scroll") {
        ui.event(akari::Event::Scroll(80., height as f32 - 260., 64.));
        ui.rebuild(
            app.view([width as f32, height as f32]),
            [width as f32, height as f32],
        );
    }
    let draws = ui.draw();
    let texture = renderer
        .device
        .create_texture(&akari_renderer::gpu::TextureDescriptor {
            label: Some("Akari component showcase preview"),
            size: akari_renderer::gpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: akari_renderer::gpu::TextureDimension::D2,
            format: renderer.format(),
            usage: akari_renderer::gpu::TextureUsages::RENDER_ATTACHMENT
                | akari_renderer::gpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
    let view = texture.create_view(&Default::default());
    renderer
        .render(&view, [width, height], 1., app.theme(), &draws)
        .map_err(|error| eyre::eyre!(error))?;
    let pixels = renderer.read_rgba(&texture, [width, height]);
    let mut ppm = format!("P6\n{width} {height}\n255\n").into_bytes();
    for pixel in pixels.chunks_exact(4) {
        ppm.extend_from_slice(&pixel[..3]);
    }
    fs::write(output_path, ppm)?;
    println!(
        "Rendered {} draw commands ({} glass effects) to {output_path}",
        draws.len(),
        draws
            .iter()
            .filter(|draw| matches!(draw, Draw::Glass { .. }))
            .count()
    );
    Ok(())
}
