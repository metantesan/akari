//! Declarative element descriptions and renderer-neutral draw commands.

use super::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}
/// A monochrome, 24×24 outline icon. Icon data is supplied by a separate
/// crate so applications link only the named icons they reference.
#[derive(Clone, Copy, Debug)]
pub struct Icon {
    pub name: &'static str,
    pub paths: &'static [&'static str],
    pub circles: &'static [(f32, f32, f32)],
}
#[derive(Clone)]
pub enum Kind<M> {
    Text(String, f32),
    Button(String, M),
    IconButton(Icon, M),
    Tab {
        label: String,
        icon: Icon,
        selected: bool,
        on_select: M,
    },
    ProgressBar(f32),
    TextField {
        value: String,
        placeholder: String,
        on_change: Arc<dyn Fn(String) -> M>,
        on_submit: Option<M>,
    },
    Row,
    Column,
    Stack,
    Container,
    Scroll,
    Image(Image),
    Icon(Icon),
}
#[derive(Clone)]
pub struct Element<M> {
    pub key: Key,
    pub kind: Kind<M>,
    pub children: Vec<Self>,
    pub width: Length,
    pub height: Length,
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub padding: f32,
    pub gap: f32,
    pub align: Align,
    pub text_align: Align,
    pub direction: Direction,
    pub surface: Surface,
    pub label: Option<String>,
    pub semantic_role: Option<Role>,
    pub selected: Option<bool>,
    pub checked: Option<bool>,
}
impl<M: Clone> Element<M> {
    fn new(key: Key, kind: Kind<M>) -> Self {
        Self {
            key,
            kind,
            children: vec![],
            width: Length::Fill,
            height: Length::Fixed(48.),
            min: [0.; 2],
            max: [f32::INFINITY; 2],
            padding: 0.,
            gap: 8.,
            align: Align::Start,
            text_align: Align::Start,
            direction: Direction::LeftToRight,
            surface: Surface::Plain,
            label: None,
            semantic_role: None,
            selected: None,
            checked: None,
        }
    }
    pub fn text(key: Key, text: impl Into<String>, size: f32) -> Self {
        Self::new(key, Kind::Text(text.into(), size)).height(size * 1.5)
    }
    pub fn button(key: Key, label: impl Into<String>, message: M) -> Self {
        let mut element = Self::new(key, Kind::Button(label.into(), message));
        element.text_align = Align::Center;
        element
    }
    pub fn icon(key: Key, icon: Icon) -> Self {
        Self::new(key, Kind::Icon(icon)).width(24.).height(24.)
    }
    pub fn icon_button(key: Key, icon: Icon, message: M) -> Self {
        let mut element = Self::new(key, Kind::IconButton(icon, message));
        element.width = Length::Fixed(44.);
        element.text_align = Align::Center;
        element
    }
    pub fn tab(key: Key, label: impl Into<String>, icon: Icon, selected: bool, message: M) -> Self {
        let label = label.into();
        let mut element = Self::new(
            key,
            Kind::Tab {
                label: label.clone(),
                icon,
                selected,
                on_select: message,
            },
        );
        element.height = Length::Fixed(72.);
        element.text_align = Align::Center;
        element.label = Some(label);
        element.semantic_role = Some(Role::Tab);
        element.selected = Some(selected);
        element
    }
    pub fn progress_bar(key: Key, value: f32) -> Self {
        let mut element = Self::new(key, Kind::ProgressBar(value.clamp(0., 1.)));
        element.height = Length::Fixed(8.);
        element.semantic_role = Some(Role::ProgressBar);
        element
    }
    pub fn text_field(
        key: Key,
        value: impl Into<String>,
        placeholder: impl Into<String>,
        on_change: impl Fn(String) -> M + 'static,
    ) -> Self {
        Self::new(
            key,
            Kind::TextField {
                value: value.into(),
                placeholder: placeholder.into(),
                on_change: Arc::new(on_change),
                on_submit: None,
            },
        )
    }
    pub fn on_submit(mut self, message: M) -> Self {
        if let Kind::TextField { on_submit, .. } = &mut self.kind {
            *on_submit = Some(message)
        }
        self
    }
    fn group(key: Key, kind: Kind<M>, children: Vec<Self>) -> Self {
        let mut e = Self::new(key, kind);
        e.children = children;
        e.height = Length::Fill;
        e
    }
    pub fn row(key: Key, children: Vec<Self>) -> Self {
        Self::group(key, Kind::Row, children)
    }
    pub fn column(key: Key, children: Vec<Self>) -> Self {
        Self::group(key, Kind::Column, children)
    }
    pub fn stack(key: Key, children: Vec<Self>) -> Self {
        Self::group(key, Kind::Stack, children)
    }
    pub fn container(key: Key, child: Self) -> Self {
        Self::group(key, Kind::Container, vec![child])
    }
    /// A container whose custom shader blurs and refracts previously drawn layers.
    pub fn liquid_glass(key: Key, child: Self, style: Glass) -> Self {
        Self::container(key, child).glass(style)
    }
    pub fn scroll(key: Key, children: Vec<Self>) -> Self {
        Self::group(key, Kind::Scroll, children)
    }
    pub fn image(key: Key, image: Image) -> Self {
        Self::new(key, Kind::Image(image))
    }
    pub fn width(mut self, v: f32) -> Self {
        self.width = Length::Fixed(v);
        self
    }
    pub fn height(mut self, v: f32) -> Self {
        self.height = Length::Fixed(v);
        self
    }
    pub fn fill_height(mut self) -> Self {
        self.height = Length::Fill;
        self
    }
    pub fn padding(mut self, v: f32) -> Self {
        self.padding = v.max(0.);
        self
    }
    pub fn gap(mut self, v: f32) -> Self {
        self.gap = v.max(0.);
        self
    }
    pub fn align(mut self, v: Align) -> Self {
        self.align = v;
        self
    }
    /// Aligns a text label inside a text or button widget.
    pub fn text_align(mut self, v: Align) -> Self {
        self.text_align = v;
        self
    }
    /// Sets text and row direction for this subtree.
    pub fn direction(mut self, direction: Direction) -> Self {
        fn apply<M>(element: &mut Element<M>, direction: Direction) {
            element.direction = direction;
            for child in &mut element.children {
                apply(child, direction);
            }
        }
        apply(&mut self, direction);
        self
    }
    pub fn constraints(mut self, min: [f32; 2], max: [f32; 2]) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn glass(mut self, style: Glass) -> Self {
        self.surface = Surface::Glass(style);
        self
    }
    /// Gives a widget an opaque, softly outlined surface from the current theme.
    pub fn card(mut self) -> Self {
        self.surface = Surface::Card;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn semantic_role(mut self, role: Role) -> Self {
        self.semantic_role = Some(role);
        self
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }
}
#[derive(Clone)]
pub enum Draw {
    Quad {
        rect: Rect,
        clip: Rect,
        color: Color,
        radius: f32,
        border: f32,
    },
    Glass {
        rect: Rect,
        clip: Rect,
        style: Glass,
    },
    Text {
        rect: Rect,
        clip: Rect,
        text: String,
        size: f32,
        color: Color,
        align: TextAlign,
    },
    TextField {
        rect: Rect,
        clip: Rect,
        text: String,
        size: f32,
        color: Color,
        align: TextAlign,
        state: WidgetState,
        focused: bool,
    },
    Image {
        rect: Rect,
        clip: Rect,
        image: Image,
    },
    Icon {
        rect: Rect,
        clip: Rect,
        icon: Icon,
        color: Color,
    },
}
