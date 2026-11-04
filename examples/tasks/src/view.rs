use super::*;

impl Tasks {
    fn view_item(&self, item: &Item, compact: bool) -> Element<Message> {
        let id = item.id;
        let key = 1_000 + id * 10;
        let title = Input::new(
            key + 2,
            item.title.clone(),
            self.text("task-title-placeholder", "Task title"),
            move |text| Message::TaskText(id, text),
        )
        .on_submit(Message::CommitTask(id))
        .label(self.format(
            "task-title-label",
            &fluent_args!["title" => item.title.as_str()],
            format!("Task: {}", item.title),
        ))
        .build();
        let toggle = Element::icon_button(
            key + 1,
            if item.done {
                lucide::circle_check
            } else {
                lucide::circle
            },
            Message::Toggle(id),
        )
        .label(if item.done {
            self.text("mark-task-active", "Mark task active")
        } else {
            self.text("complete-task", "Complete task")
        });
        let delete_label = self.attribute(
            "delete-task",
            "aria-label",
            &fluent_args!["title" => item.title.as_str()],
            format!("Delete task {}", item.title),
        );
        let delete = Element::icon_button(key + 4, lucide::trash, Message::Delete(id))
            .label(delete_label.clone());
        let mut row = if compact {
            let title_row = Element::row(key + 5, vec![toggle, title])
                .gap(8.)
                .height(48.);
            let spacer = Element::text(key + 6, "", 12.);
            let action_items = vec![
                spacer,
                Element::icon_button(key + 4, lucide::trash, Message::Delete(id))
                    .height(34.)
                    .label(delete_label),
            ];
            let actions = Element::row(key + 7, action_items).gap(8.).height(38.);
            Element::column(key, vec![title_row, actions])
                .gap(2.)
                .height(96.)
                .padding(8.)
                .card()
        } else {
            let row_items = vec![toggle, title, delete];
            Element::row(key, row_items)
                .gap(8.)
                .height(58.)
                .padding(8.)
                .card()
        };
        row.align = Align::Center;
        row
    }
}

impl SimpleComponent for Tasks {
    type Input = Message;

    fn title(&self) -> &str {
        &self.title
    }

    fn update(&mut self, message: Message) -> Vec<Task<Message>> {
        match message {
            Message::Draft(value) => self.draft = value,
            Message::Add => self.add(self.draft.clone()),
            Message::Toggle(id) => {
                if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
                    item.done = !item.done;
                }
            }
            Message::Delete(id) => self.items.retain(|item| item.id != id),
            Message::TaskText(id, text) => {
                if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
                    item.title = text;
                }
            }
            Message::CommitTask(id) => {
                if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
                    item.title = item.title.trim().to_owned();
                }
            }
            Message::ToggleTheme => self.dark = Some(!self.current_dark()),
            Message::SelectFilter(filter) => self.filter = filter,
        }
        Vec::new()
    }

    fn view(&self, size: [f32; 2]) -> Element<Message> {
        let compact = size[0] < 560.;
        let completed = self.items.iter().filter(|item| item.done).count();
        let mut children = vec![
            Element::row(
                1,
                vec![
                    Element::column(
                        2,
                        vec![
                            Element::text(3, self.text("app-title", "Tasks"), 30.),
                            Element::text(
                                4,
                                self.format(
                                    "task-summary",
                                    &fluent_args!["open" => self.items.len() - completed, "done" => completed],
                                    format!("{} open · {completed} done", self.items.len() - completed),
                                ),
                                15.,
                            ),
                        ],
                    )
                    .gap(2.),
                    Element::icon_button(5, lucide::sun_moon, Message::ToggleTheme)
                        .label(self.text("switch-color-theme", "Switch color theme")),
                ],
            )
            .gap(12.)
            .height(72.),
            Element::row(
                6,
                vec![
                    Input::new(
                        7,
                        self.draft.clone(),
                        self.text("add-placeholder", "Add a task"),
                        Message::Draft,
                    )
                        .on_submit(Message::Add)
                        .label(self.text("new-task-label", "New task"))
                        .build(),
                    Element::icon_button(8, lucide::plus, Message::Add)
                        .label(self.text("add-task-label", "Add task")),
                ],
            )
            .gap(8.)
            .height(54.),
        ];
        let rows = self
            .items
            .iter()
            .filter(|item| self.filter.includes(item))
            .map(|item| self.view_item(item, compact))
            .collect();
        let item_height = if compact { 104. } else { 66. };
        let visible_count = self
            .items
            .iter()
            .filter(|item| self.filter.includes(item))
            .count();
        let mut list = Element::column(9, rows).gap(8.).padding(2.);
        list.height = akari::Length::Fixed(visible_count as f32 * item_height + 4.);
        let mut scroll = Element::scroll(10, vec![list]).fill_height();
        scroll.height = akari::Length::Fill;
        children.push(scroll);
        let tabs = Filter::ALL
            .into_iter()
            .map(|filter| {
                TabItem::new(
                    self.text(filter.message_id(), filter.message_id()),
                    filter.icon(),
                    Message::SelectFilter(filter),
                )
            })
            .collect();
        children.push(
            TabBar::new(70, self.filter.index(), tabs)
                .label(self.text("filter-tasks", "Filter tasks"))
                .height(64.)
                .liquid_glass(Glass::default())
                .build(),
        );
        let mut root = Element::column(11, children)
            .padding(if compact { 18. } else { 28. })
            .gap(16.)
            .direction(
                if self.localization.as_ref().is_some_and(|localization| {
                    localization.direction() == TextDirection::RightToLeft
                }) {
                    Direction::RightToLeft
                } else {
                    Direction::LeftToRight
                },
            );
        root.height = akari::Length::Fill;
        root
    }

    fn theme(&self) -> Theme {
        if self.current_dark() {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    fn theme_preference(&self) -> ThemePreference {
        if self.dark.is_some() {
            ThemePreference::Custom
        } else {
            ThemePreference::System
        }
    }
}
