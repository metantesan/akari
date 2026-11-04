use akari_core::{Element, Key, Role};
use akari_icons::lucide;

/// Controlled radio choice. Group selection is owned by the application.
pub struct RadioItem<M> {
    key: Key,
    label: String,
    selected: bool,
    on_select: M,
}

impl<M> RadioItem<M> {
    pub fn new(key: Key, label: impl Into<String>, selected: bool, on_select: M) -> Self {
        Self {
            key,
            label: label.into(),
            selected,
            on_select,
        }
    }

    pub fn build(self) -> Element<M>
    where
        M: Clone,
    {
        Element::icon_button(
            self.key,
            if self.selected {
                lucide::circle_dot
            } else {
                lucide::circle
            },
            self.on_select,
        )
        .label(self.label)
        .semantic_role(Role::Radio)
        .selected(self.selected)
        .checked(self.selected)
    }
}
