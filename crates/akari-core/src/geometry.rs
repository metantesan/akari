//! Shared geometry, sizing, and directional alignment primitives.

pub type Key = u64;
pub type Color = [f32; 4];
pub(crate) const TEXT_FIELD_HORIZONTAL_INSET: f32 = 12.;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x,
            y,
            w: w.max(0.),
            h: h.max(0.),
        }
    }
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
    pub fn intersect(self, b: Self) -> Self {
        let x = self.x.max(b.x);
        let y = self.y.max(b.y);
        Self::new(
            x,
            y,
            (self.x + self.w).min(b.x + b.w) - x,
            (self.y + self.h).min(b.y + b.h) - y,
        )
    }
    pub fn inset(self, p: f32) -> Self {
        Self::new(self.x + p, self.y + p, self.w - 2. * p, self.h - 2. * p)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Constraints {
    pub min: [f32; 2],
    pub max: [f32; 2],
}
impl Constraints {
    pub fn constrain(self, size: [f32; 2]) -> [f32; 2] {
        std::array::from_fn(|i| {
            size[i].clamp(self.min[i].max(0.), self.max[i].max(self.min[i]).max(0.))
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Fill,
    Fixed(f32),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Align {
    Start,
    Center,
    End,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    LeftToRight,
    RightToLeft,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}
pub(crate) fn directional_align(direction: Direction, align: Align) -> TextAlign {
    match (direction, align) {
        (Direction::LeftToRight, Align::Start) | (Direction::RightToLeft, Align::End) => {
            TextAlign::Left
        }
        (Direction::RightToLeft, Align::Start) | (Direction::LeftToRight, Align::End) => {
            TextAlign::Right
        }
        (_, Align::Center) => TextAlign::Center,
    }
}
