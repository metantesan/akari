use akari_core::{Element, Key};

/// Padded surface for grouping related content.
pub struct Card<M> {
    key: Key,
    child: Element<M>,
    padding: f32,
    height: Option<f32>,
}

impl<M: Clone> Card<M> {
    pub fn new(key: Key, child: Element<M>) -> Self {
        Self {
            key,
            child,
            padding: 16.,
            height: None,
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding.max(0.);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height.max(1.));
        self
    }

    pub fn build(self) -> Element<M> {
        let mut card = Element::container(self.key, self.child)
            .padding(self.padding)
            .card();
        if let Some(height) = self.height {
            card = card.height(height);
        }
        card
    }
}
