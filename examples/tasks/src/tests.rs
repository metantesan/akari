use super::*;

#[test]
fn tasks_can_be_added_edited_completed_and_removed() {
    let mut app = Tasks {
        draft: "Write tests".into(),
        ..Tasks::default()
    };
    app.update(Message::Add);
    let id = app.items[7].id;
    assert_eq!(app.items[7].title, "Write tests");
    app.update(Message::Toggle(id));
    assert!(app.items[7].done);
    app.update(Message::TaskText(id, "Review tests".into()));
    app.update(Message::CommitTask(id));
    assert_eq!(app.items[7].title, "Review tests");
    app.update(Message::Delete(id));
    assert!(!app.items.iter().any(|item| item.id == id));
}

#[test]
fn task_titles_stay_inline_fields_while_their_text_changes() {
    let mut app = Tasks::default();
    let size = [390., 500.];
    let mut ui = akari::Ui::default();
    ui.rebuild(app.view(size), size);
    let first_title = ui
        .semantics()
        .into_iter()
        .find(|semantic| semantic.key == 1_012)
        .expect("value must be present");
    assert_eq!(first_title.role, akari::Role::TextField);
    ui.event(akari::Event::PointerDown(
        first_title.bounds.x + 10.,
        first_title.bounds.y + 10.,
    ));
    let mut messages = ui.event(akari::Event::Edit(akari::EditKey::SelectAll, false));
    messages.extend(ui.event(akari::Event::Text("A clearer title".into())));
    for message in messages {
        app.update(message);
    }
    assert_eq!(app.items[0].title, "A clearer title");
    ui.rebuild(app.view(size), size);
    assert_eq!(
        ui.semantics()
            .into_iter()
            .find(|semantic| semantic.key == 1_012)
            .expect("value must be present")
            .role,
        akari::Role::TextField
    );
}

#[test]
fn theme_toggle_uses_shared_light_and_dark_tokens() {
    let mut app = Tasks::default();
    let was_dark = Tasks::system_dark();
    app.update(Message::ToggleTheme);
    assert_eq!(
        app.theme().background,
        if was_dark {
            Theme::light().background
        } else {
            Theme::dark().background
        }
    );
}

#[test]
fn task_tabs_filter_open_and_completed_items() {
    let mut app = Tasks::default();
    let size = [390., 640.];
    let mut ui = akari::Ui::default();
    ui.rebuild(app.view(size), size);
    let done_tab = ui
        .semantics()
        .into_iter()
        .find(|semantic| semantic.key == 73)
        .expect("Done tab");
    assert_eq!(done_tab.role, akari::Role::Tab);
    let x = done_tab.bounds.x + done_tab.bounds.w * 0.5;
    let y = done_tab.bounds.y + done_tab.bounds.h * 0.5;
    ui.event(akari::Event::PointerDown(x, y));
    for message in ui.event(akari::Event::PointerUp(x, y)) {
        app.update(message);
    }
    assert_eq!(app.filter, Filter::Done);
    ui.rebuild(app.view(size), size);
    assert!(ui.semantics().iter().any(|semantic| semantic.key == 1_032));
    assert!(!ui.semantics().iter().any(|semantic| semantic.key == 1_012));
}

#[test]
fn app_uses_system_font_fallback_and_has_scrollable_task_rows() {
    let app = Tasks::default();
    assert!(app.fonts().is_empty());
    let mut ui = akari::Ui::default();
    ui.rebuild(app.view([360., 360.]), [360., 360.]);
    assert!(
        ui.states
            .get(&10)
            .is_some_and(|state| state.scroll_max > 0.)
    );
    assert!(!ui.draw().is_empty());
}

#[test]
fn keyboard_and_pointer_routes_reach_add_and_scroll_messages() {
    let mut app = Tasks::default();
    let size = [390., 400.];
    let mut ui = akari::Ui::default();
    ui.rebuild(app.view(size), size);

    ui.event(akari::Event::Tab(false));
    ui.event(akari::Event::Tab(false));
    for message in ui.event(akari::Event::Text("Keyboard task".into())) {
        app.update(message);
    }
    for message in ui.event(akari::Event::Activate) {
        app.update(message);
    }
    assert!(app.items.iter().any(|item| item.title == "Keyboard task"));

    ui.rebuild(app.view(size), size);
    ui.event(akari::Event::PointerDown(334., 132.));
    let pointer_messages = ui.event(akari::Event::PointerUp(334., 132.));
    assert!(
        pointer_messages
            .iter()
            .any(|message| matches!(message, Message::Add))
    );
    for message in pointer_messages {
        app.update(message);
    }

    ui.rebuild(app.view(size), size);
    ui.event(akari::Event::Scroll(180., 300., 72.));
    assert!(ui.states.get(&10).is_some_and(|state| state.scroll > 0.));
}

#[test]
fn tab_focus_recovers_after_deleting_the_focused_task_button() {
    let mut app = Tasks::default();
    let size = [780., 800.];
    let mut ui = akari::Ui::default();
    ui.rebuild(app.view(size), size);

    let delete = ui
        .semantics()
        .into_iter()
        .find(|semantic| semantic.key == 1_014)
        .expect("first task delete button");
    let x = delete.bounds.x + delete.bounds.w * 0.5;
    let y = delete.bounds.y + delete.bounds.h * 0.5;
    ui.event(akari::Event::PointerDown(x, y));
    let messages = ui.event(akari::Event::PointerUp(x, y));
    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::Delete(1)))
    );
    for message in messages {
        app.update(message);
    }
    ui.rebuild(app.view(size), size);
    assert_eq!(
        ui.focused,
        Some(1_021),
        "deleting a focused control should move focus to the next task"
    );

    ui.event(akari::Event::Tab(false));
    let first_focus = ui.focused;
    assert!(
        first_focus.is_some(),
        "Tab should restore focus after deletion"
    );
    ui.event(akari::Event::Tab(false));
    assert_ne!(ui.focused, first_focus, "the next Tab should advance focus");
    ui.event(akari::Event::Tab(true));
    assert_eq!(
        ui.focused, first_focus,
        "Shift+Tab should return to prior focus"
    );

    let last_delete = ui
        .semantics()
        .into_iter()
        .find(|semantic| semantic.key == 1_074)
        .expect("last task delete button");
    let x = last_delete.bounds.x + last_delete.bounds.w * 0.5;
    let y = last_delete.bounds.y + last_delete.bounds.h * 0.5;
    ui.event(akari::Event::PointerDown(x, y));
    let messages = ui.event(akari::Event::PointerUp(x, y));
    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::Delete(7)))
    );
    for message in messages {
        app.update(message);
    }
    ui.rebuild(app.view(size), size);
    assert_eq!(
        ui.focused,
        Some(71),
        "deleting the final task should move focus to the first task filter"
    );
    ui.event(akari::Event::Tab(false));
    assert_eq!(
        ui.focused,
        Some(72),
        "Tab should advance across task filters"
    );
    ui.event(akari::Event::Tab(true));
    assert_eq!(ui.focused, Some(71), "Shift+Tab should reverse tab focus");
}
