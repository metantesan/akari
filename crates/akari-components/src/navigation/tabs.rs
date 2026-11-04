use akari_core::{Element, Glass, Icon, Key, Role};

/// One controlled destination in a [`TabBar`]. Selecting it emits its message;
/// the application decides what content to show next.
pub struct TabItem<M> {
    label: String,
    icon: Icon,
    message: M,
}

impl<M> TabItem<M> {
    pub fn new(label: impl Into<String>, icon: Icon, message: M) -> Self {
        Self {
            label: label.into(),
            icon,
            message,
        }
    }
}

/// A controlled, accessible tab bar suitable for top or bottom navigation.
///
/// The selected index belongs to the app model. Tab changes are ordinary app
/// messages, so the component works with Akari's `update`/`view` cycle.
pub struct TabBar<M> {
    key: Key,
    selected: usize,
    tabs: Vec<TabItem<M>>,
    height: f32,
    label: String,
    glass: Option<Glass>,
}

impl<M> TabBar<M> {
    pub fn new(key: Key, selected: usize, tabs: Vec<TabItem<M>>) -> Self {
        Self {
            key,
            selected,
            tabs,
            height: 72.,
            label: "Navigation".to_owned(),
            glass: None,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(40.);
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Applies Akari's own liquid-glass effect to the tab bar surface.
    pub fn liquid_glass(mut self, style: Glass) -> Self {
        self.glass = Some(style);
        self
    }

    pub fn solid(mut self) -> Self {
        self.glass = None;
        self
    }

    pub fn build(self) -> Element<M>
    where
        M: Clone,
    {
        let tabs = self
            .tabs
            .into_iter()
            .enumerate()
            .map(|(index, tab)| {
                Element::tab(
                    self.key + index as Key + 1,
                    tab.label,
                    tab.icon,
                    index == self.selected,
                    tab.message,
                )
            })
            .collect();
        let mut bar = Element::row(self.key, tabs)
            .height(self.height)
            .padding(4.)
            .gap(0.)
            .semantic_role(Role::TabList)
            .label(self.label);
        if let Some(style) = self.glass {
            bar = bar.glass(style);
        }
        bar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use akari_core::{Draw, Event, Semantic, Ui};

    #[derive(Clone, Debug, PartialEq)]
    enum Message {
        Select(usize),
    }

    fn bar() -> Element<Message> {
        TabBar::new(
            10,
            1,
            vec![
                TabItem::new(
                    "Chats",
                    akari_icons::lucide::messages_square,
                    Message::Select(0),
                ),
                TabItem::new(
                    "Contacts",
                    akari_icons::lucide::users_round,
                    Message::Select(1),
                ),
                TabItem::new("Calls", akari_icons::lucide::phone, Message::Select(2)),
            ],
        )
        .liquid_glass(Glass::default())
        .build()
    }

    #[test]
    fn tabs_expose_selection_and_accessible_list_semantics() {
        let mut ui = Ui::default();
        ui.rebuild(bar(), [360., 80.]);
        let semantics = ui.semantics();
        let list = semantics
            .iter()
            .find(|item| item.key == 10)
            .expect("tab list semantics");
        assert_eq!(list.role, Role::TabList);
        assert_eq!(list.label, "Navigation");
        let contacts = semantics
            .iter()
            .find(|item| item.key == 12)
            .expect("contacts tab semantics");
        assert_eq!(contacts.role, Role::Tab);
        assert_eq!(contacts.label, "Contacts");
        assert_eq!(contacts.selected, Some(true));
        assert!(
            ui.draw()
                .iter()
                .any(|draw| matches!(draw, Draw::Glass { .. }))
        );
    }

    #[test]
    fn pointer_and_keyboard_selection_emit_app_messages() {
        let mut ui = Ui::default();
        ui.rebuild(bar(), [360., 80.]);
        let bounds = ui
            .semantics()
            .into_iter()
            .find(|item: &Semantic| item.key == 13)
            .expect("calls tab")
            .bounds;
        let x = bounds.x + bounds.w * 0.5;
        let y = bounds.y + bounds.h * 0.5;
        ui.event(Event::PointerDown(x, y));
        assert_eq!(ui.event(Event::PointerUp(x, y)), vec![Message::Select(2)]);
        ui.event(Event::Tab(false));
        assert_eq!(ui.focused, Some(11));
        assert_eq!(ui.event(Event::Activate), vec![Message::Select(0)]);
    }
}
