use crate::{TabBar, TabItem};
use akari_core::{Element, Icon, Key};

pub struct Segment<M> {
    label: String,
    icon: Icon,
    message: M,
}

impl<M> Segment<M> {
    pub fn new(label: impl Into<String>, icon: Icon, message: M) -> Self {
        Self {
            label: label.into(),
            icon,
            message,
        }
    }
}

/// A controlled row of mutually exclusive options, sharing tab keyboard behavior.
pub struct SegmentedButton<M> {
    key: Key,
    selected: usize,
    segments: Vec<Segment<M>>,
    label: String,
}

impl<M> SegmentedButton<M> {
    pub fn new(key: Key, selected: usize, segments: Vec<Segment<M>>) -> Self {
        Self {
            key,
            selected,
            segments,
            label: "Options".into(),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn build(self) -> Element<M>
    where
        M: Clone,
    {
        let items = self
            .segments
            .into_iter()
            .map(|segment| TabItem::new(segment.label, segment.icon, segment.message))
            .collect();
        TabBar::new(self.key, self.selected, items)
            .label(self.label)
            .height(52.)
            .solid()
            .build()
    }
}
