mod view;

use akari::{
    Align, Direction, Element, Glass, SimpleComponent, Task, Theme, ThemePreference,
    components::{Input, TabBar, TabItem},
    icons::lucide,
    native::localization::{FluentArgs, Localization, TextDirection, fluent_args},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    All,
    Open,
    Done,
}

impl Filter {
    const ALL: [Self; 3] = [Self::All, Self::Open, Self::Done];

    const fn index(self) -> usize {
        match self {
            Self::All => 0,
            Self::Open => 1,
            Self::Done => 2,
        }
    }

    const fn message_id(self) -> &'static str {
        match self {
            Self::All => "filter-all",
            Self::Open => "filter-open",
            Self::Done => "filter-done",
        }
    }

    const fn icon(self) -> akari::Icon {
        match self {
            Self::All => lucide::list_checks,
            Self::Open => lucide::circle,
            Self::Done => lucide::circle_check,
        }
    }

    fn includes(self, item: &Item) -> bool {
        match self {
            Self::All => true,
            Self::Open => !item.done,
            Self::Done => item.done,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Draft(String),
    Add,
    Toggle(u64),
    Delete(u64),
    TaskText(u64, String),
    CommitTask(u64),
    ToggleTheme,
    SelectFilter(Filter),
}

#[derive(Clone, Debug)]
struct Item {
    id: u64,
    title: String,
    done: bool,
}

pub struct Tasks {
    title: String,
    localization: Option<Localization>,
    items: Vec<Item>,
    draft: String,
    next_id: u64,
    dark: Option<bool>,
    filter: Filter,
}

impl Default for Tasks {
    fn default() -> Self {
        let mut localization = Localization::from_system("en-US").ok();
        if let Some(localization) = &mut localization {
            let _ = localization.add_resource("en-US", include_str!("../i18n/en-US.ftl"));
            let _ = localization.add_resource("fa", include_str!("../i18n/fa.ftl"));
        }
        let title = localization
            .as_ref()
            .and_then(|localization| localization.text("app-title"))
            .unwrap_or_else(|| "Tasks".to_owned());
        Self {
            title,
            localization,
            items: vec![
                Item {
                    id: 1,
                    title: "Plan the week".into(),
                    done: false,
                },
                Item {
                    id: 2,
                    title: "Review project notes".into(),
                    done: false,
                },
                Item {
                    id: 3,
                    title: "Take a short break".into(),
                    done: true,
                },
                Item {
                    id: 4,
                    title: "Send the update".into(),
                    done: false,
                },
                Item {
                    id: 5,
                    title: "Read for twenty minutes".into(),
                    done: false,
                },
                Item {
                    id: 6,
                    title: "Tidy up the desk".into(),
                    done: true,
                },
                Item {
                    id: 7,
                    title: "برنامه فردا — project update".into(),
                    done: false,
                },
            ],
            draft: String::new(),
            next_id: 8,
            dark: None,
            filter: Filter::All,
        }
    }
}

impl Tasks {
    fn text(&self, id: &str, fallback: &str) -> String {
        self.localization
            .as_ref()
            .and_then(|localization| localization.text(id))
            .unwrap_or_else(|| fallback.to_owned())
    }

    fn format(&self, id: &str, args: &FluentArgs<'_>, fallback: String) -> String {
        self.localization
            .as_ref()
            .and_then(|localization| localization.format(id, Some(args)))
            .unwrap_or(fallback)
    }

    fn attribute(
        &self,
        id: &str,
        attribute: &str,
        args: &FluentArgs<'_>,
        fallback: String,
    ) -> String {
        self.localization
            .as_ref()
            .and_then(|localization| localization.attribute(id, attribute, Some(args)))
            .unwrap_or(fallback)
    }

    fn system_dark() -> bool {
        akari::native::ThemeProvider::system_theme(&akari::native::current(), None)
            == akari::native::theme::SystemTheme::Dark
    }

    fn current_dark(&self) -> bool {
        self.dark.unwrap_or_else(Self::system_dark)
    }

    fn add(&mut self, title: String) {
        let title = title.trim();
        if !title.is_empty() {
            self.items.push(Item {
                id: self.next_id,
                title: title.to_owned(),
                done: false,
            });
            self.next_id += 1;
            self.draft.clear();
        }
    }
}

#[cfg(test)]
mod tests;
