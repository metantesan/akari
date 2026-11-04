use akari_core::*;
#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Click(u64),
    Text(String),
}
fn form(text: &str) -> Element<Msg> {
    Element::column(
        0,
        vec![
            Element::text_field(1, text, "Title", Msg::Text),
            Element::button(2, "Add", Msg::Click(2)),
            Element::button(3, "Remove", Msg::Click(3)),
        ],
    )
}
#[test]
fn constraints_and_clipped_hit_testing() {
    assert_eq!(
        Constraints {
            min: [10., 20.],
            max: [100., 80.]
        }
        .constrain([200., 0.]),
        [100., 20.]
    );
    let mut ui = Ui::default();
    ui.rebuild(form(""), [200., 60.]);
    assert_eq!(ui.hit_test(20., 20.), Some(1));
    assert_eq!(ui.hit_test(20., 58.), Some(2));
    assert_eq!(ui.hit_test(20., 70.), None);
    assert_eq!(ui.hit_test(200., 20.), None);
}

#[test]
fn centered_row_children_fit_inside_the_padded_cross_axis() {
    let mut ui = Ui::default();
    ui.rebuild(
        Element::column(
            0,
            vec![
                Element::row(1, vec![Element::button(2, "x", Msg::Click(2)).width(44.)])
                    .height(60.)
                    .padding(8.)
                    .align(Align::Center),
            ],
        ),
        [120., 100.],
    );
    let bounds = ui
        .semantics()
        .into_iter()
        .find(|semantic| semantic.key == 2)
        .expect("value must be present")
        .bounds;
    assert_eq!(bounds, Rect::new(8., 8., 44., 44.));
    assert_eq!(bounds.y + bounds.h / 2., 30.);
}

#[test]
fn text_field_draw_is_clipped_to_its_own_bounds() {
    let mut ui = Ui::default();
    ui.rebuild(
        Element::row(
            0,
            vec![
                Element::text_field(1, "a long title", "Title", Msg::Text),
                Element::button(2, "Delete", Msg::Click(2)).width(64.),
            ],
        ),
        [240., 48.],
    );
    let field = ui
        .draw()
        .into_iter()
        .find_map(|draw| match draw {
            Draw::TextField { rect, clip, .. } => Some((rect, clip)),
            _ => None,
        })
        .expect("value must be present");
    let semantics = ui.semantics();
    let field_bounds = semantics
        .iter()
        .find(|semantic| semantic.key == 1)
        .expect("value must be present")
        .bounds;
    let delete = semantics
        .into_iter()
        .find(|semantic| semantic.key == 2)
        .expect("value must be present")
        .bounds;
    assert_eq!(field.0.x - field_bounds.x, 12.);
    assert_eq!(field_bounds.x + field_bounds.w - field.0.x - field.0.w, 12.);
    assert!(field.1.x >= field_bounds.x);
    assert!(field.1.x + field.1.w <= field_bounds.x + field_bounds.w);
    assert!(field.1.x + field.1.w <= delete.x);
}
#[test]
fn keys_preserve_state_and_deleted_keys_are_removed() {
    let mut ui = Ui::default();
    ui.rebuild(form("abc"), [300., 200.]);
    ui.focused = Some(1);
    ui.states.get_mut(&1).expect("value must be present").cursor = 2;
    ui.rebuild(
        Element::column(
            0,
            vec![
                Element::button(3, "Remove", Msg::Click(3)),
                Element::text_field(1, "abc", "Title", Msg::Text),
            ],
        ),
        [300., 200.],
    );
    assert_eq!(ui.states[&1].cursor, 2);
    assert_eq!(ui.focused, Some(1));
    ui.rebuild(Element::text(0, "Empty", 16.), [300., 200.]);
    assert!(!ui.states.contains_key(&1));
    assert_eq!(ui.focused, None);
}
#[test]
fn keyboard_focus_and_activation() {
    let mut ui = Ui::default();
    ui.rebuild(form(""), [300., 200.]);
    ui.event(Event::Tab(false));
    assert_eq!(ui.focused, Some(1));
    ui.event(Event::Tab(true));
    assert_eq!(ui.focused, Some(3));
    assert_eq!(ui.event(Event::Activate), vec![Msg::Click(3)]);
    ui.event(Event::Tab(false));
    ui.event(Event::Tab(false));
    assert_eq!(ui.focused, Some(2));
}
#[test]
fn pointer_release_requires_same_button_and_cancel_prevents_activation() {
    let mut ui = Ui::default();
    ui.rebuild(form(""), [300., 200.]);
    ui.event(Event::PointerDown(20., 65.));
    assert!(ui.event(Event::PointerUp(20., 120.)).is_empty());
    ui.event(Event::PointerDown(20., 65.));
    ui.event(Event::Cancel);
    assert!(ui.event(Event::PointerUp(20., 65.)).is_empty());
    ui.event(Event::PointerDown(20., 65.));
    assert_eq!(ui.event(Event::PointerUp(20., 65.)), vec![Msg::Click(2)]);
}
#[test]
fn unicode_graphemes_selection_and_ime() {
    for text in ["سَلام", "a\u{301}", "👩‍👩‍👧‍👦", "سلام Rust"] {
        let mut ui = Ui::default();
        ui.rebuild(form(text), [300., 200.]);
        ui.event(Event::PointerDown(10., 10.));
        ui.event(Event::Edit(EditKey::SelectAll, false));
        ui.event(Event::Composition("世界".into()));
        assert_eq!(ui.states[&1].composition, "世界");
        assert_eq!(
            ui.event(Event::Text("世界".into())),
            vec![Msg::Text("世界".into())]
        );
        assert!(ui.states[&1].composition.is_empty());
    }
    let mut ui = Ui::default();
    ui.rebuild(form("a\u{301}"), [300., 200.]);
    ui.event(Event::PointerDown(10., 10.));
    assert_eq!(
        ui.event(Event::Edit(EditKey::Backspace, false)),
        vec![Msg::Text(String::new())]
    );
    ui.rebuild(form("👩‍👩‍👧‍👦x"), [300., 200.]);
    ui.states.get_mut(&1).expect("value must be present").cursor = 0;
    ui.states.get_mut(&1).expect("value must be present").anchor = 0;
    assert_eq!(
        ui.event(Event::Edit(EditKey::Delete, false)),
        vec![Msg::Text("x".into())]
    );
}
#[test]
fn scrolling_is_bounded_and_tab_reveals_focus() {
    let mut ui = Ui::default();
    let root = || {
        Element::scroll(
            0,
            (1..=20)
                .map(|i| Element::button(i, "Task", Msg::Click(i)))
                .collect(),
        )
    };
    ui.rebuild(root(), [300., 100.]);
    ui.event(Event::Scroll(10., 10., 100000.));
    let max = ui.states[&0].scroll_max;
    assert!(max > 0.);
    assert_eq!(ui.states[&0].scroll, max);
    ui.rebuild(root(), [300., 100.]);
    ui.event(Event::Tab(false));
    ui.rebuild(root(), [300., 100.]);
    let bounds = ui.focused_bounds().expect("value must be present");
    assert!(bounds.y >= 0. && bounds.y + bounds.h <= 100.);
    ui.event(Event::Scroll(10., 10., -100000.));
    assert_eq!(ui.states[&0].scroll, 0.);
}
#[test]
fn theme_and_semantics_resolve() {
    let mut ui = Ui::default();
    ui.theme = Theme::dark();
    ui.rebuild(form(""), [300., 200.]);
    ui.event(Event::Tab(false));
    let nodes = ui.semantics();
    let field = nodes
        .iter()
        .find(|n| n.key == 1)
        .expect("value must be present");
    assert_eq!(field.role, Role::TextField);
    assert_eq!(field.label, "Title");
    assert!(field.focused);
    assert!(field.actions.contains(&"set_value"));
    assert!(
        ui.draw()
            .iter()
            .any(|d| matches!(d,Draw::Text{color,..} if *color==ui.theme.text))
    );
    assert_ne!(Theme::light().text, Theme::dark().text);
}
#[test]
fn task_results_become_application_messages() {
    struct App(u64);
    impl Application for App {
        type Message = u64;
        fn update(&mut self, m: u64) -> Vec<Task<u64>> {
            self.0 += m;
            vec![]
        }
        fn view(&self, _: [f32; 2]) -> Element<u64> {
            Element::text(0, self.0.to_string(), 16.)
        }
    }
    let task = Task::perform(|| 42);
    let result = std::thread::spawn(move || (task.0)())
        .join()
        .expect("value must be present");
    let mut app = App(0);
    app.update(result);
    assert_eq!(app.0, 42);
}
#[test]
#[should_panic(expected = "widget keys must be unique")]
fn duplicate_keys_are_rejected() {
    let mut ui = Ui::default();
    ui.rebuild(
        Element::row(
            0,
            vec![
                Element::button(1, "A", Msg::Click(1)),
                Element::button(1, "B", Msg::Click(1)),
            ],
        ),
        [300., 200.],
    );
}
