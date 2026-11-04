use akari::{Draw, SimpleComponent, Ui};
use akari_tasks::{Message, Tasks};
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<_>>();
    let output_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("/tmp/akari-tasks.ppm");
    let width = args
        .get(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(780_u32);
    let height = args
        .get(3)
        .and_then(|value| value.parse().ok())
        .unwrap_or(800_u32);
    let mut app = Tasks::default();
    let system_dark = akari::native::ThemeProvider::system_theme(&akari::native::current(), None)
        == akari::native::theme::SystemTheme::Dark;
    let requested_dark = match args.get(4).map(String::as_str) {
        Some("dark") => Some(true),
        Some("light") => Some(false),
        _ => None,
    };
    if requested_dark.is_some_and(|dark| dark != system_dark) {
        app.update(Message::ToggleTheme);
    }
    let mut ui = Ui::default();
    ui.theme = app.theme();
    ui.rebuild(
        app.view([width as f32, height as f32]),
        [width as f32, height as f32],
    );
    let draws = ui.draw();
    let instance = akari_renderer::gpu::Instance::new(akari_renderer::instance_descriptor());
    let mut renderer = pollster::block_on(akari_renderer::Renderer::new(&instance, None))?;
    let output_texture = renderer
        .device
        .create_texture(&akari_renderer::gpu::TextureDescriptor {
            label: Some("Akari task app preview"),
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
    let view = output_texture.create_view(&Default::default());
    renderer.render(&view, [width, height], 1., app.theme(), &draws)?;
    let pixels = renderer.read_rgba(&output_texture, [width, height]);
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
