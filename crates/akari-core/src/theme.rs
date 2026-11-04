//! Theme values and accessibility semantics shared by widgets and runtime.

use super::{Color, Key, Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glass {
    pub blur: f32,
    pub refraction: f32,
    pub tint: Color,
    pub radius: f32,
    pub border: f32,
}
impl Default for Glass {
    fn default() -> Self {
        Self {
            blur: 12.,
            refraction: 5.,
            tint: [0.8, 0.9, 1., 0.18],
            radius: 18.,
            border: 1.,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Surface {
    Plain,
    Card,
    Glass(Glass),
}
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub outline: Color,
    pub glass: Glass,
}

/// Identifies the GPU adapter selected by Akari's renderer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RendererInfo {
    pub adapter_name: String,
    pub backend: String,
    pub device_type: String,
}
/// Selects whether an application follows the operating system appearance or
/// supplies its own palette through `theme()`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemePreference {
    #[default]
    System,
    Custom,
}
impl Theme {
    pub fn light() -> Self {
        Self {
            background: [0.957, 0.965, 0.976, 1.],
            surface: [1., 1., 1., 1.],
            text: [0.10, 0.13, 0.18, 1.],
            muted: [0.36, 0.40, 0.46, 1.],
            accent: [0.11, 0.32, 0.64, 1.],
            outline: [0.84, 0.87, 0.90, 1.],
            glass: Glass::default(),
        }
    }
    pub fn dark() -> Self {
        Self {
            background: [0.075, 0.09, 0.115, 1.],
            surface: [0.13, 0.15, 0.18, 1.],
            text: [0.91, 0.93, 0.95, 1.],
            muted: [0.63, 0.67, 0.72, 1.],
            accent: [0.43, 0.65, 0.93, 1.],
            outline: [0.25, 0.28, 0.32, 1.],
            glass: Glass {
                tint: [0.13, 0.22, 0.4, 0.4],
                ..Glass::default()
            },
        }
    }
}
impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Role {
    Text,
    Button,
    CheckBox,
    Switch,
    Radio,
    Slider,
    ProgressBar,
    Select,
    Accordion,
    TabList,
    Tab,
    TextField,
    Group,
    ScrollView,
    Image,
}
#[derive(Clone, Debug)]
pub struct Semantic {
    pub key: Key,
    pub role: Role,
    pub label: String,
    pub bounds: Rect,
    pub focused: bool,
    pub selected: Option<bool>,
    pub checked: Option<bool>,
    pub value: Option<f32>,
    pub actions: Vec<&'static str>,
}
