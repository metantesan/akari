use akari_core::{Element, Key, Role};

/// Controlled select trigger. Applications render the option popup and update
/// the displayed value in their normal `update`/`view` cycle.
pub struct Select<M> {
    key: Key,
    value: String,
    on_activate: M,
    label: Option<String>,
}

impl<M> Select<M> {
    pub fn new(key: Key, value: impl Into<String>, on_activate: M) -> Self {
        Self {
            key,
            value: value.into(),
            on_activate,
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn build(self) -> Element<M>
    where
        M: Clone,
    {
        let mut trigger =
            Element::button(self.key, self.value, self.on_activate).semantic_role(Role::Select);
        if let Some(label) = self.label {
            trigger = trigger.label(label);
        }
        trigger
    }
}
