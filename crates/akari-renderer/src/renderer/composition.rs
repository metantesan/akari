use super::*;

impl Renderer {
    pub fn render(
        &mut self,
        output: &wgpu::TextureView,
        size: [u32; 2],
        scale: f32,
        theme: Theme,
        draws: &[Draw],
    ) -> Result<(), String> {
        if size.contains(&0) {
            return Ok(());
        }
        self.resize(size, scale);
        // Reset bounded caches before preparing a frame, never halfway through it.
        if self.pack[1] > ATLAS * 3 / 4 {
            self.glyphs.clear();
            self.icons.clear();
            self.pack = [1, 1, 0];
        }
        self.queue.write_buffer(
            &self.globals,
            0,
            bytemuck::cast_slice(&[size[0] as f32, size[1] as f32, 0., 0.]),
        );
        let full = Rect::new(0., 0., size[0] as f32, size[1] as f32);
        let mut items = vec![Instance::new(full, full, theme.background, 0., 0., 0.)];
        let mut glass_indices = vec![];
        let scaled = |r: Rect| Rect::new(r.x * scale, r.y * scale, r.w * scale, r.h * scale);
        for draw in draws {
            match draw {
                Draw::Quad {
                    rect,
                    clip,
                    color,
                    radius,
                    border,
                } => items.push(Instance::new(
                    scaled(*rect),
                    scaled(*clip),
                    *color,
                    0.,
                    radius * scale,
                    border * scale,
                )),
                Draw::Glass { rect, clip, style } => {
                    glass_indices.push(items.len());
                    let mut i = Instance::new(
                        scaled(*rect),
                        scaled(*clip),
                        style.tint,
                        2.,
                        style.radius * scale,
                        0.,
                    );
                    i.optical = [
                        style.blur.clamp(0., 64.) * scale,
                        style.refraction.clamp(-32., 32.) * scale,
                        style.border.max(0.) * scale,
                        0.,
                    ];
                    items.push(i);
                }
                Draw::Text {
                    rect,
                    clip,
                    text,
                    size,
                    color,
                    align,
                } => self.text(
                    &mut items,
                    scaled(*rect),
                    scaled(*clip),
                    text,
                    (size * scale, *color, *align),
                ),
                field @ Draw::TextField { .. } => {
                    self.text_field(&mut items, field, scale, theme.accent);
                }
                Draw::Image { rect, clip, image } => {
                    if image.width > 0
                        && image.height > 0
                        && image.rgba.len() == image.width as usize * image.height as usize * 4
                        && let Some(uv) = self.allocate(image.width, image.height)
                    {
                        self.upload(uv, image.width, image.height, &image.rgba);
                        let mut i =
                            Instance::new(scaled(*rect), scaled(*clip), [1.; 4], 1., 0., 0.);
                        i.uv = uv;
                        items.push(i);
                    }
                }
                Draw::Icon {
                    rect,
                    clip,
                    icon,
                    color,
                } => {
                    let rect = scaled(*rect);
                    let clip = scaled(*clip);
                    for segment in self.icon_segments(*icon) {
                        let mut i = Instance::new(rect, clip, *color, 3., 1., 0.);
                        i.optical = *segment;
                        items.push(i);
                    }
                }
            }
        }
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ordered quads"),
                contents: bytemuck::cast_slice(&items),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let targets = self.targets.as_ref().expect("value must be present");
        let view = targets.scene.create_view(&Default::default());
        let mut start = 0;
        for boundary in glass_indices
            .into_iter()
            .chain(std::iter::once(items.len()))
        {
            if boundary > start {
                self.pass(
                    &mut encoder,
                    &view,
                    &targets.bind,
                    &buffer,
                    start as u32..boundary as u32,
                    start == 0,
                );
            }
            if boundary < items.len() {
                encoder.copy_texture_to_texture(
                    targets.scene.as_image_copy(),
                    targets.snapshot.as_image_copy(),
                    wgpu::Extent3d {
                        width: size[0],
                        height: size[1],
                        depth_or_array_layers: 1,
                    },
                );
            }
            start = boundary;
        }
        // Present the composed scene using the same texture path as image quads.
        let final_bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("present scene"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.globals.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &targets.snapshot.create_view(&Default::default()),
                    ),
                },
            ],
        });
        let mut final_quad = Instance::new(full, full, [1.; 4], 1., 0., 0.);
        final_quad.uv = [0., 0., 1., 1.];
        let final_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&final_quad),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.pass(&mut encoder, output, &final_bind, &final_buffer, 0..1, true);
        self.queue.submit([encoder.finish()]);
        Ok(())
    }
    fn pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        bind: &wgpu::BindGroup,
        buffer: &wgpu::Buffer,
        range: std::ops::Range<u32>,
        clear: bool,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Akari layer"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if clear {
                        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                    } else {
                        wgpu::LoadOp::Load
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind, &[]);
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..6, range);
    }
}
