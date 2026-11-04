use akari_core::{Element, Key};

/// A compact card intended for a single item in a grid or list.
pub struct Tile<M> {
    key: Key,
    child: Element<M>,
}

impl<M: Clone> Tile<M> {
    pub fn new(key: Key, child: Element<M>) -> Self {
        Self { key, child }
    }

    pub fn build(self) -> Element<M> {
        Element::container(self.key, self.child).padding(12.).card()
    }
}
