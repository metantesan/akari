use akari_core::{Element, Key};

/// Compact, clickable label for filters and small actions.
pub struct Chip<M> {
    key: Key,
    label: String,
    message: M,
    selected: bool,
}

impl<M> Chip<M> {
    pub fn new(key: Key, label: impl Into<String>, message: M) -> Self {
        Self {
            key,
            label: label.into(),
            message,
            selected: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn build(self) -> Element<M>
    where
        M: Clone,
    {
        let mut chip = Element::button(self.key, self.label, self.message)
            .height(36.)
            .padding(8.)
            .card()
            .selected(self.selected);
        if self.selected {
            chip.surface = akari_core::Surface::Glass(akari_core::Glass {
                radius: 18.,
                ..akari_core::Glass::default()
            });
        }
        chip
    }
}
