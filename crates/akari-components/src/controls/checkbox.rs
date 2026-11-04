use akari_core::{Element, Key, Role};
use akari_icons::lucide;

/// Controlled checkbox. The application owns `checked` and handles `on_toggle`.
pub struct Checkbox<M> {
    key: Key,
    label: String,
    checked: bool,
    on_toggle: M,
}

impl<M> Checkbox<M> {
    pub fn new(key: Key, label: impl Into<String>, checked: bool, on_toggle: M) -> Self {
        Self {
            key,
            label: label.into(),
            checked,
            on_toggle,
        }
    }

    pub fn build(self) -> Element<M>
    where
        M: Clone,
    {
        Element::icon_button(
            self.key,
            if self.checked {
                lucide::square_check
            } else {
                lucide::square
            },
            self.on_toggle,
        )
        .label(self.label)
        .semantic_role(Role::CheckBox)
        .checked(self.checked)
    }
}
