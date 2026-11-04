use akari_core::{Element, Key};

/// A vertical scrolling region with stable child keys.
pub struct ScrollView<M> {
    key: Key,
    children: Vec<Element<M>>,
}

impl<M: Clone> ScrollView<M> {
    pub fn new(key: Key, children: Vec<Element<M>>) -> Self {
        Self { key, children }
    }

    pub fn build(self) -> Element<M> {
        Element::scroll(self.key, self.children)
    }
}
