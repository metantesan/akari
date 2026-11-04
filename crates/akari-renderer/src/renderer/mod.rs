//! Ordered GPU compositor with a glyph atlas and custom liquid glass shader.
use akari_core::{Color, Draw, Icon, Rect, TextAlign, Theme};
use cosmic_text::{
    Affinity, Attrs, Buffer, CacheKey, Cursor, FontSystem, Metrics, Shaping, SwashCache,
    SwashContent, Wrap,
};
use std::{collections::HashMap, sync::Arc};
use unicode_bidi::BidiInfo;
use wgpu::util::DeviceExt;

mod composition;
mod text;

const ATLAS: u32 = 2048;

use super::backend::{backend_priority, device_priority};
use super::icons::flatten_icon;
#[cfg(test)]
use super::icons::flatten_svg_path;
fn natural_text_align(text: &str) -> TextAlign {
    if !text.is_ascii()
        && BidiInfo::new(text, None)
            .paragraphs
            .first()
            .is_some_and(|paragraph| paragraph.level.is_rtl())
    {
        TextAlign::Right
    } else {
        TextAlign::Left
    }
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    rect: [f32; 4],
    clip: [f32; 4],
    color: Color,
    uv: [f32; 4],
    params: [f32; 4],
    optical: [f32; 4],
}
impl Instance {
    fn new(rect: Rect, clip: Rect, color: Color, mode: f32, radius: f32, border: f32) -> Self {
        Self {
            rect: [rect.x, rect.y, rect.w, rect.h],
            clip: [clip.x, clip.y, clip.w, clip.h],
            color,
            uv: [0.; 4],
            params: [radius, border, mode, 0.],
            optical: [0.; 4],
        }
    }
}
#[derive(Clone, Copy)]
struct Glyph {
    uv: [f32; 4],
    left: i32,
    top: i32,
    width: u32,
    height: u32,
    color: bool,
}
struct Targets {
    scene: wgpu::Texture,
    snapshot: wgpu::Texture,
    bind: wgpu::BindGroup,
}
pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    adapter_info: wgpu::AdapterInfo,
    pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    atlas: wgpu::Texture,
    sampler: wgpu::Sampler,
    targets: Option<Targets>,
    size: [u32; 2],
    format: wgpu::TextureFormat,
    fonts: FontSystem,
    swash: SwashCache,
    glyphs: HashMap<CacheKey, Glyph>,
    icons: HashMap<&'static str, Vec<[f32; 4]>>,
    pack: [u32; 3],
    scale: f32,
}
impl Renderer {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, String> {
        let mut adapters = instance.enumerate_adapters(wgpu::Backends::all()).await;
        adapters
            .retain(|adapter| surface.is_none_or(|surface| adapter.is_surface_supported(surface)));
        adapters.sort_by_key(|adapter| {
            let info = adapter.get_info();
            (
                backend_priority(info.backend),
                device_priority(info.device_type),
            )
        });
        let adapter = adapters.into_iter().next().ok_or_else(|| {
            "no wgpu adapter supports the configured platform backends and surface".to_owned()
        })?;
        let adapter_info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Akari GPU"),
                ..Default::default()
            })
            .await
            .map_err(|e| e.to_string())?;
        let format = surface
            .map(|s| {
                s.get_capabilities(&adapter)
                    .formats
                    .into_iter()
                    .find(|f| f.is_srgb())
                    .unwrap_or(wgpu::TextureFormat::Bgra8Unorm)
            })
            .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb);
        Self::from_device(device, queue, format, adapter_info)
    }
    fn from_device(
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        adapter_info: wgpu::AdapterInfo,
    ) -> Result<Self, String> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Akari liquid glass"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../glass.wgsl").into()),
        });
        let attrs = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4,2=>Float32x4,3=>Float32x4,4=>Float32x4,5=>Float32x4];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Akari compositor"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &attrs,
                })],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewport"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let atlas = texture(
            &device,
            [ATLAS, ATLAS],
            wgpu::TextureFormat::Rgba8Unorm,
            "glyph atlas",
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("clamped linear"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Ok(Self {
            device,
            queue,
            adapter_info,
            pipeline,
            globals,
            atlas,
            sampler,
            targets: None,
            size: [0, 0],
            format,
            fonts: FontSystem::new(),
            swash: SwashCache::new(),
            glyphs: HashMap::new(),
            icons: HashMap::new(),
            pack: [1, 1, 0],
            scale: 1.,
        })
    }
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }
    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }
    pub fn configure(&mut self, surface: &wgpu::Surface<'_>, size: [u32; 2]) {
        if size.contains(&0) {
            return;
        }
        surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                width: size[0],
                height: size[1],
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                color_space: Default::default(),
            },
        );
    }
    fn resize(&mut self, size: [u32; 2], scale: f32) {
        if self.scale != scale {
            self.glyphs.clear();
            self.icons.clear();
            self.pack = [1, 1, 0];
            self.swash = SwashCache::new();
            self.scale = scale;
        }
        if self.size == size {
            return;
        }
        self.size = size;
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC;
        let scene = texture(
            &self.device,
            size,
            self.format,
            "scene",
            usage | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let snapshot = texture(
            &self.device,
            size,
            self.format,
            "glass backdrop",
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let av = self.atlas.create_view(&Default::default());
        let sv = snapshot.create_view(&Default::default());
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compositor textures"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.globals.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&av),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&sv),
                },
            ],
        });
        self.targets = Some(Targets {
            scene,
            snapshot,
            bind,
        });
    }
    fn allocate(&mut self, w: u32, h: u32) -> Option<[f32; 4]> {
        if w + 2 >= ATLAS || h + 2 >= ATLAS {
            return None;
        }
        if self.pack[0] + w + 1 >= ATLAS {
            self.pack[0] = 1;
            self.pack[1] += self.pack[2] + 2;
            self.pack[2] = 0;
        }
        if self.pack[1] + h + 1 >= ATLAS {
            return None;
        }
        let p = self.pack;
        self.pack[0] += w + 2;
        self.pack[2] = self.pack[2].max(h);
        Some([
            p[0] as f32 / ATLAS as f32,
            p[1] as f32 / ATLAS as f32,
            w as f32 / ATLAS as f32,
            h as f32 / ATLAS as f32,
        ])
    }
    fn upload(&self, uv: [f32; 4], w: u32, h: u32, data: &[u8]) {
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.atlas,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: (uv[0] * ATLAS as f32) as u32,
                    y: (uv[1] * ATLAS as f32) as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }
    fn icon_segments(&mut self, icon: Icon) -> &[[f32; 4]] {
        self.icons
            .entry(icon.name)
            .or_insert_with(|| flatten_icon(icon))
    }
    /// Useful for deterministic offscreen renderer validation and screenshots.
    pub fn offscreen_texture(&self, size: [u32; 2]) -> wgpu::Texture {
        texture(
            &self.device,
            size,
            self.format,
            "offscreen output",
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        )
    }
    /// Read an offscreen output after rendering. Blocks until GPU work completes.
    pub fn read_rgba(&self, texture: &wgpu::Texture, size: [u32; 2]) -> Vec<u8> {
        let pitch = (size[0] * 4).div_ceil(256) * 256;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (pitch * size[1]) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(pitch),
                    rows_per_image: Some(size[1]),
                },
            },
            wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).expect("value must be present")
            });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("value must be present");
        rx.recv()
            .expect("value must be present")
            .expect("value must be present");
        let data = buffer
            .slice(..)
            .get_mapped_range()
            .expect("value must be present");
        let result = data
            .chunks(pitch as usize)
            .flat_map(|row| row[..size[0] as usize * 4].iter().copied())
            .collect();
        drop(data);
        buffer.unmap();
        result
    }
    /// Register bundled fonts before first render, particularly on mobile.
    pub fn load_font(&mut self, bytes: Arc<Vec<u8>>) {
        self.fonts
            .db_mut()
            .load_font_source(cosmic_text::fontdb::Source::Binary(bytes));
        self.glyphs.clear();
        self.pack = [1, 1, 0];
    }
}

fn texture(
    device: &wgpu::Device,
    size: [u32; 2],
    format: wgpu::TextureFormat,
    label: &str,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}
#[cfg(test)]
mod tests;
