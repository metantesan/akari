use akari_core::{Element, Key};

/// Controlled disclosure section. `open` is owned by the application.
pub struct Accordion<M> {
    key: Key,
    title: String,
    open: bool,
    on_toggle: M,
    content: Vec<Element<M>>,
}

impl<M: Clone> Accordion<M> {
    pub fn new(
        key: Key,
        title: impl Into<String>,
        open: bool,
        on_toggle: M,
        content: Vec<Element<M>>,
    ) -> Self {
        Self {
            key,
            title: title.into(),
            open,
            on_toggle,
            content,
        }
    }

    pub fn build(self) -> Element<M> {
        let mut children = vec![
            Element::button(self.key.wrapping_add(1), self.title, self.on_toggle)
                .selected(self.open),
        ];
        if self.open {
            children.extend(self.content);
        }
        Element::column(self.key, children)
    }
}
