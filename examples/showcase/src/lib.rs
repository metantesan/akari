mod views;

use akari::{
    Align, Element, Image as RasterImage, RendererInfo, SimpleComponent, Task, Theme,
    ThemePreference,
    components::{
        Accordion, Button, ButtonStyle, Card, Checkbox, Chip, Image, Input, ProgressBar, RadioItem,
        ScrollView, Segment, SegmentedButton, Select, Switch, TabBar, TabItem, Table, Tile,
        heading, label,
    },
    icons::lucide,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Controls,
    Layout,
    Media,
}

impl Page {
    const ALL: [Self; 3] = [Self::Controls, Self::Layout, Self::Media];

    const fn index(self) -> usize {
        match self {
            Self::Controls => 0,
            Self::Layout => 1,
            Self::Media => 2,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Navigate(Page),
    PressButton,
    ToggleChip,
    InputChanged(String),
    ToggleCheck,
    ToggleSwitch,
    ChooseDensity(usize),
    ChooseRadio(usize),
    ActivateSelect,
    ToggleAccordion,
    AdvanceProgress,
    ToggleTheme,
}

pub struct Showcase {
    page: Page,
    press_count: u32,
    chip_selected: bool,
    input_value: String,
    checked: bool,
    switch_enabled: bool,
    density: usize,
    radio: usize,
    select_choice: usize,
    accordion_open: bool,
    progress: f32,
    forced_dark: Option<bool>,
    renderer_description: String,
}

impl Default for Showcase {
    fn default() -> Self {
        Self {
            page: Page::Controls,
            press_count: 0,
            chip_selected: true,
            input_value: "Edit this text".into(),
            checked: true,
            switch_enabled: true,
            density: 0,
            radio: 0,
            select_choice: 0,
            accordion_open: true,
            progress: 0.68,
            forced_dark: None,
            renderer_description: "Waiting for the renderer".into(),
        }
    }
}

fn system_is_dark() -> bool {
    akari::native::ThemeProvider::system_theme(&akari::native::current(), None)
        == akari::native::theme::SystemTheme::Dark
}

fn panel<M: Clone>(
    key: u64,
    title: &str,
    description: &str,
    body: Element<M>,
    height: f32,
) -> Element<M> {
    let content = Element::column(
        key + 1,
        vec![heading(key + 2, title), label(key + 3, description), body],
    )
    .gap(7.);
    Card::new(key, content).padding(18.).height(height).build()
}

fn icon_label<M: Clone>(key: u64, icon: akari::Icon, title: &str) -> Element<M> {
    Element::row(
        key,
        vec![Element::icon(key + 1, icon), label(key + 2, title)],
    )
    .gap(10.)
    .height(38.)
    .align(Align::Center)
}

fn sample_bitmap() -> RasterImage {
    let (width, height) = (144_u32, 88_u32);
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let horizon = 46 + (x as i32 / 14 % 4) * 3;
            let color = if (y as i32) < horizon {
                [39, 91 + (x % 3) as u8 * 12, 133, 255]
            } else if y > 68 {
                [39, 68 + (x % 4) as u8 * 8, 75, 255]
            } else {
                [78, 123 + (x % 3) as u8 * 8, 119, 255]
            };
            rgba.extend_from_slice(&color);
        }
    }
    RasterImage {
        width,
        height,
        rgba: Arc::from(rgba),
    }
}

impl SimpleComponent for Showcase {
    type Input = Message;

    fn title(&self) -> &str {
        "Akari Component Showcase"
    }

    fn renderer_info(&mut self, info: RendererInfo) {
        self.renderer_description = format!("{} · {}", info.backend, info.device_type);
    }

    fn update(&mut self, message: Message) -> Vec<Task<Message>> {
        match message {
            Message::Navigate(page) => self.page = page,
            Message::PressButton => self.press_count += 1,
            Message::ToggleChip => self.chip_selected = !self.chip_selected,
            Message::InputChanged(value) => self.input_value = value,
            Message::ToggleCheck => self.checked = !self.checked,
            Message::ToggleSwitch => self.switch_enabled = !self.switch_enabled,
            Message::ChooseDensity(index) => self.density = index.min(2),
            Message::ChooseRadio(index) => self.radio = index.min(1),
            Message::ActivateSelect => self.select_choice = (self.select_choice + 1) % 3,
            Message::ToggleAccordion => self.accordion_open = !self.accordion_open,
            Message::AdvanceProgress => {
                self.progress += 0.16;
                if self.progress > 1.0 {
                    self.progress = 0.0;
                }
            }
            Message::ToggleTheme => {
                let current = self.forced_dark.unwrap_or_else(system_is_dark);
                self.forced_dark = Some(!current);
            }
        }
        Vec::new()
    }

    fn view(&self, size: [f32; 2]) -> Element<Message> {
        let compact = size[0] < 620.;
        let heading_row = Element::row(
            1,
            vec![
                Element::column(
                    2,
                    vec![
                        heading(3, "Akari components"),
                        label(4, "A live field guide to the widgets in your toolkit."),
                        Element::text(6, format!("Renderer: {}", self.renderer_description), 13.),
                    ],
                )
                .gap(2.),
                Element::icon_button(5, lucide::sun_moon, Message::ToggleTheme).label(
                    if self.forced_dark.unwrap_or_else(system_is_dark) {
                        "Switch to light theme"
                    } else {
                        "Switch to dark theme"
                    },
                ),
            ],
        )
        .gap(12.)
        .height(104.)
        .align(Align::Center);
        let navigation = TabBar::new(
            10,
            self.page.index(),
            Page::ALL
                .into_iter()
                .map(|page| {
                    let (name, icon) = match page {
                        Page::Controls => ("Controls", lucide::sliders_horizontal),
                        Page::Layout => ("Layout", lucide::panels_top_left),
                        Page::Media => ("Media", lucide::image),
                    };
                    TabItem::new(name, icon, Message::Navigate(page))
                })
                .collect(),
        )
        .label("Showcase sections")
        .height(62.)
        .solid()
        .build();
        let content = match self.page {
            Page::Controls => self.controls(compact),
            Page::Layout => self.layout(),
            Page::Media => self.media(),
        };
        let mut scroll = ScrollView::new(20, vec![content]).build().fill_height();
        scroll.height = akari::Length::Fill;
        Element::column(0, vec![heading_row, navigation, scroll])
            .padding(if compact { 14. } else { 24. })
            .gap(12.)
            .direction(akari::Direction::LeftToRight)
    }

    fn theme(&self) -> Theme {
        if self.forced_dark.unwrap_or_else(system_is_dark) {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    fn theme_preference(&self) -> ThemePreference {
        if self.forced_dark.is_some() {
            ThemePreference::Custom
        } else {
            ThemePreference::System
        }
    }
}
