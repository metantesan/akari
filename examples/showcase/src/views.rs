use super::*;

impl Showcase {
    pub(super) fn controls(&self, compact: bool) -> Element<Message> {
        let button_body = Element::column(
            120,
            vec![
                Element::row(
                    121,
                    vec![
                        Button::new(122, "Press me", Message::PressButton).build(),
                        Button::new(123, "Card action", Message::PressButton)
                            .style(ButtonStyle::Card)
                            .build(),
                    ],
                )
                .gap(10.)
                .height(48.),
                label(124, format!("Pressed {} times", self.press_count)),
                Element::row(
                    125,
                    vec![
                        Chip::new(126, "Design", Message::ToggleChip)
                            .selected(self.chip_selected)
                            .build(),
                        Chip::new(127, "Rust", Message::ToggleChip)
                            .selected(!self.chip_selected)
                            .build(),
                    ],
                )
                .gap(8.)
                .height(38.),
            ],
        )
        .gap(8.);
        let buttons = panel(
            110,
            "Actions",
            "Buttons and compact filter chips",
            button_body,
            245.,
        );

        let preference_rows = if compact {
            vec![
                Element::row(
                    222,
                    vec![
                        Checkbox::new(223, "Accept updates", self.checked, Message::ToggleCheck)
                            .build(),
                        label(224, "Accept updates"),
                    ],
                )
                .height(48.)
                .align(Align::Center),
                Element::row(
                    225,
                    vec![
                        Switch::new(
                            226,
                            "Notifications",
                            self.switch_enabled,
                            Message::ToggleSwitch,
                        )
                        .build(),
                        label(227, "Notifications"),
                    ],
                )
                .height(48.)
                .align(Align::Center),
            ]
        } else {
            vec![
                Element::row(
                    222,
                    vec![
                        Checkbox::new(223, "Accept updates", self.checked, Message::ToggleCheck)
                            .build(),
                        label(224, "Accept updates"),
                        Switch::new(
                            225,
                            "Notifications",
                            self.switch_enabled,
                            Message::ToggleSwitch,
                        )
                        .build(),
                        label(226, "Notifications"),
                    ],
                )
                .gap(6.)
                .height(48.)
                .align(Align::Center),
            ]
        };
        let mut choice_children = preference_rows;
        choice_children.extend([
            Element::row(
                227,
                vec![
                    RadioItem::new(
                        228,
                        "Comfortable density",
                        self.radio == 0,
                        Message::ChooseRadio(0),
                    )
                    .build(),
                    label(229, "Comfortable"),
                    RadioItem::new(
                        230,
                        "Compact density",
                        self.radio == 1,
                        Message::ChooseRadio(1),
                    )
                    .build(),
                    label(231, "Compact"),
                ],
            )
            .gap(6.)
            .height(48.)
            .align(Align::Center),
            SegmentedButton::new(
                232,
                self.density,
                vec![
                    Segment::new("List", lucide::list, Message::ChooseDensity(0)),
                    Segment::new("Grid", lucide::layout_grid, Message::ChooseDensity(1)),
                    Segment::new("Board", lucide::columns_3, Message::ChooseDensity(2)),
                ],
            )
            .label("View density")
            .build(),
        ]);
        let choice_rows = Element::column(221, choice_children).gap(7.);
        let forms = panel(
            210,
            "Choices",
            "Controlled selection with accessible labels",
            choice_rows,
            if compact { 365. } else { 330. },
        );

        let select_body = Element::row(
            321,
            vec![
                Input::new(
                    322,
                    self.input_value.clone(),
                    "Type something",
                    Message::InputChanged,
                )
                .label("Demo text input")
                .build(),
                Select::new(
                    323,
                    ["System", "Light", "Dark"][self.select_choice],
                    Message::ActivateSelect,
                )
                .label("Color preference")
                .build(),
            ],
        )
        .gap(10.)
        .height(48.);
        let field_panel = panel(
            310,
            "Text and selection",
            "Input editing and a controlled select trigger",
            select_body,
            170.,
        );

        let intro = panel(
            100,
            "Controls",
            "Real widgets, live state, and the same message loop used by apps.",
            Element::row(
                104,
                vec![
                    Element::icon(105, lucide::mouse_pointer_2),
                    label(106, "Click, type, tab, and change the controls below."),
                ],
            )
            .height(40.)
            .align(Align::Center),
            160.,
        );
        Element::column(90, vec![intro, buttons, forms, field_panel])
            .gap(12.)
            .height(if compact { 985. } else { 950. })
    }

    pub(super) fn layout(&self) -> Element<Message> {
        let density = if self.radio == 0 { 62. } else { 46. };
        let tiles = Element::row(
            421,
            vec![
                Tile::new(422, label(423, "Left tile · keyed content"))
                    .build()
                    .height(density),
                Tile::new(424, label(425, "Right tile · independent"))
                    .build()
                    .height(density),
            ],
        )
        .gap(10.)
        .height(density);
        let tile_panel = panel(
            410,
            "Tiles",
            "Small surfaces for compact collections",
            tiles,
            180.,
        );

        let accordion_content = vec![
            label(444, "Disclosure content stays in the same retained tree."),
            label(445, "The open state belongs to application data."),
        ];
        let accordion = Accordion::new(
            441,
            if self.accordion_open {
                "Hide implementation notes"
            } else {
                "Show implementation notes"
            },
            self.accordion_open,
            Message::ToggleAccordion,
            accordion_content,
        )
        .build()
        .height(if self.accordion_open { 118. } else { 48. });
        let accordion_panel = panel(
            430,
            "Accordion",
            "A controlled disclosure section",
            accordion,
            240.,
        );

        let table = Table::new(
            461,
            vec![
                vec![
                    label(462, "Component"),
                    label(463, "Group"),
                    label(464, "Status"),
                ],
                vec![
                    label(465, "TabBar"),
                    label(466, "Navigation"),
                    label(467, "Ready"),
                ],
                vec![
                    label(468, "Card"),
                    label(469, "Layout"),
                    label(476, "Ready"),
                ],
            ],
        )
        .build();
        let table_panel = panel(450, "Table", "Simple rows for small data sets", table, 225.);

        let scroll_items = (0..7)
            .map(|index| {
                Element::container(
                    500 + index,
                    label(490 + index, format!("Scrollable row {}", index + 1)),
                )
                .height(34.)
                .padding(7.)
                .card()
            })
            .collect();
        let scroll_content = Element::column(480, scroll_items).height(262.).gap(4.);
        let inner_scroll = ScrollView::new(481, vec![scroll_content])
            .build()
            .height(128.)
            .card()
            .padding(10.);
        let scroll_panel = panel(
            470,
            "Scroll view",
            "Nested scroll area with its own retained offset",
            inner_scroll,
            245.,
        );

        Element::column(
            400,
            vec![tile_panel, accordion_panel, scroll_panel, table_panel],
        )
        .gap(12.)
        .height(930.)
    }

    pub(super) fn media(&self) -> Element<Message> {
        let progress_body = Element::column(
            521,
            vec![
                label(522, format!("{:.0}% complete", self.progress * 100.)),
                ProgressBar::<Message>::new(523, self.progress)
                    .label("Showcase progress")
                    .build()
                    .height(10.),
                Button::new(524, "Advance progress", Message::AdvanceProgress).build(),
            ],
        )
        .gap(8.);
        let progress_panel = panel(
            510,
            "Progress",
            "Status feedback with a semantic value",
            progress_body,
            220.,
        );

        let image = Image::<Message>::new(541, sample_bitmap()).build();
        let image_body = Element::row(
            542,
            vec![
                image.width(180.).height(108.),
                Element::column(
                    543,
                    vec![
                        heading(544, "Raster image"),
                        label(545, "Decoded pixels are supplied to Akari's own renderer."),
                    ],
                )
                .gap(6.),
            ],
        )
        .gap(16.)
        .height(112.)
        .align(Align::Center);
        let image_panel = panel(530, "Image", "Local bitmap content", image_body, 230.);

        let icon_values = [
            ("Search", lucide::search),
            ("Calendar", lucide::calendar_days),
            ("Sparkles", lucide::sparkles),
            ("Layers", lucide::layers_2),
            ("Cloud", lucide::cloud),
            ("Check", lucide::circle_check),
        ];
        let icon_rows = icon_values
            .into_iter()
            .enumerate()
            .map(|(index, (title, icon))| icon_label(561 + index as u64 * 4, icon, title))
            .collect();
        let icons = Element::column(560, icon_rows).gap(5.);
        let icon_panel = panel(
            550,
            "Lucide icons",
            "Only referenced icon values enter the app",
            icons,
            370.,
        );

        Element::column(500, vec![progress_panel, image_panel, icon_panel])
            .gap(12.)
            .height(860.)
    }
}
