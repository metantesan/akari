use akari_core::{Element, Key};

/// Controlled single-line text input with submit and semantic-label support.
pub struct Input<M, F> {
    key: Key,
    value: String,
    placeholder: String,
    on_change: F,
    on_submit: Option<M>,
    label: Option<String>,
}

impl<M, F> Input<M, F> {
    pub fn new(
        key: Key,
        value: impl Into<String>,
        placeholder: impl Into<String>,
        on_change: F,
    ) -> Self {
        Self {
            key,
            value: value.into(),
            placeholder: placeholder.into(),
            on_change,
            on_submit: None,
            label: None,
        }
    }

    pub fn on_submit(mut self, message: M) -> Self {
        self.on_submit = Some(message);
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl<M, F> Input<M, F>
where
    M: Clone,
    F: Fn(String) -> M + 'static,
{
    pub fn build(self) -> Element<M> {
        let mut input = Element::text_field(self.key, self.value, self.placeholder, self.on_change);
        if let Some(message) = self.on_submit {
            input = input.on_submit(message);
        }
        if let Some(label) = self.label {
            input = input.label(label);
        }
        input
    }
}
