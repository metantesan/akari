//! Cross-platform window lifecycle and input adapters.

mod keyboard;
mod runtime;

pub use runtime::{run, run_with_event_loop};
pub use winit;
