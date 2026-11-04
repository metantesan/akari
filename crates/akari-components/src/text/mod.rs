use akari_core::{Element, Key};

/// Creates a regular text label.
pub fn label<M: Clone>(key: Key, text: impl Into<String>) -> Element<M> {
    Element::text(key, text, 16.)
}

/// Creates a larger heading.
pub fn heading<M: Clone>(key: Key, text: impl Into<String>) -> Element<M> {
    Element::text(key, text, 24.)
}
