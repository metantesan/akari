use akari_core::{EditKey, Event};
use winit::keyboard::{Key, ModifiersState, NamedKey};

pub(super) fn keyboard_event(
    key: &Key,
    text: Option<&str>,
    modifiers: ModifiersState,
    composing: bool,
) -> Option<Event> {
    if composing {
        return None;
    }
    let shift = modifiers.shift_key();
    let command = modifiers.control_key() || modifiers.super_key();
    let edit = match key {
        Key::Named(NamedKey::Tab) => return Some(Event::Tab(shift)),
        Key::Named(NamedKey::Enter) => return Some(Event::Activate),
        Key::Named(NamedKey::Escape) => return Some(Event::Cancel),
        Key::Named(NamedKey::ArrowLeft) => Some(EditKey::Left),
        Key::Named(NamedKey::ArrowRight) => Some(EditKey::Right),
        Key::Named(NamedKey::Home) => Some(EditKey::Home),
        Key::Named(NamedKey::End) => Some(EditKey::End),
        Key::Named(NamedKey::Backspace) => Some(EditKey::Backspace),
        Key::Named(NamedKey::Delete) => Some(EditKey::Delete),
        Key::Character(c) if command && c.eq_ignore_ascii_case("a") => Some(EditKey::SelectAll),
        _ => None,
    };
    if let Some(edit) = edit {
        return Some(Event::Edit(edit, shift));
    }
    if command {
        return None;
    }
    text.filter(|t| !t.is_empty() && !t.chars().any(char::is_control))
        .map(|t| Event::Text(t.to_owned()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typing_spaces_shortcuts_and_composition() {
        let none = ModifiersState::empty();
        assert!(
            matches!(keyboard_event(&Key::Character("a".into()),Some("a"),none,false),Some(Event::Text(t)) if t=="a")
        );
        assert!(
            matches!(keyboard_event(&Key::Named(NamedKey::Space),Some(" "),none,false),Some(Event::Text(t)) if t==" ")
        );
        assert!(keyboard_event(&Key::Character("a".into()), Some("a"), none, true).is_none());
        assert!(keyboard_event(&Key::Named(NamedKey::Enter), Some("\r"), none, true).is_none());
        assert!(matches!(
            keyboard_event(
                &Key::Character("a".into()),
                Some("a"),
                ModifiersState::CONTROL,
                false
            ),
            Some(Event::Edit(EditKey::SelectAll, _))
        ));
        assert!(matches!(
            keyboard_event(
                &Key::Named(NamedKey::Tab),
                None,
                ModifiersState::SHIFT,
                false
            ),
            Some(Event::Tab(true))
        ));
    }
}
