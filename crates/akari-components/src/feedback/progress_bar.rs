use akari_core::{Element, Key};
use std::marker::PhantomData;

/// Displays a bounded completion value from 0.0 to 1.0.
pub struct ProgressBar<M = ()> {
    key: Key,
    value: f32,
    label: Option<String>,
    message: PhantomData<fn() -> M>,
}

impl<M> ProgressBar<M> {
    pub fn new(key: Key, value: f32) -> Self {
        Self {
            key,
            value: value.clamp(0.0, 1.0),
            label: None,
            message: PhantomData,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn build(self) -> Element<M>
    where
        M: Clone,
    {
        let mut element = Element::progress_bar(self.key, self.value);
        if let Some(label) = self.label {
            element = element.label(label);
        }
        element
    }
}
