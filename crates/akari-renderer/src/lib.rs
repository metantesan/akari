//! Portable custom renderer and GPU backend selection.

mod backend;
mod icons;
mod renderer;

pub use backend::instance_descriptor;
pub use renderer::Renderer;
#[doc(hidden)]
pub use wgpu as gpu;
