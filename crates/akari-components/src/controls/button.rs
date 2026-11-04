use akari_core::{Element, Key};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    #[default]
    Standard,
    Card,
}

/// A message-driven button with a reusable appearance choice.
pub struct Button<M> {
    key: Key,
    label: String,
    message: M,
    height: f32,
    style: ButtonStyle,
}

impl<M> Button<M> {
    pub fn new(key: Key, label: impl Into<String>, message: M) -> Self {
        Self {
            key,
            label: label.into(),
            message,
            height: 48.,
            style: ButtonStyle::Standard,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(36.);
        self
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn build(self) -> Element<M>
    where
        M: Clone,
    {
        let mut button = Element::button(self.key, self.label, self.message).height(self.height);
        if self.style == ButtonStyle::Card {
            button = button.card();
        }
        button
    }
}
