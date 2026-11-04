use akari_core::{Element, Key, Role};
use akari_icons::lucide;

/// Controlled on/off switch with an accessible checked state.
pub struct Switch<M> {
    key: Key,
    label: String,
    checked: bool,
    on_toggle: M,
}

impl<M> Switch<M> {
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
                lucide::toggle_right
            } else {
                lucide::toggle_left
            },
            self.on_toggle,
        )
        .label(self.label)
        .semantic_role(Role::Switch)
        .checked(self.checked)
    }
}
