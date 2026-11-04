//! Reusable, message-driven Akari components built on `akari-core` elements.
//!
//! This crate only describes widgets. Rendering stays in Akari's own `wgpu`
//! renderer and WGSL pipeline.

pub mod controls;
pub mod feedback;
pub mod layout;
pub mod media;
pub mod navigation;
pub mod text;

pub use controls::{
    Button, ButtonStyle, Checkbox, Chip, Input, RadioItem, Segment, SegmentedButton, Select, Switch,
};
pub use feedback::ProgressBar;
pub use layout::{Accordion, Card, ScrollView, Table, Tile};
pub use media::Image;
pub use navigation::{TabBar, TabItem};
pub use text::{heading, label};
