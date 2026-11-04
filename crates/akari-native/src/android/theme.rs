use crate::theme::SystemTheme;
use std::sync::atomic::{AtomicU8, Ordering};

static SYSTEM_THEME: AtomicU8 = AtomicU8::new(0);

pub(crate) fn resolve(is_dark: Option<bool>) -> SystemTheme {
    match is_dark {
        Some(true) => SystemTheme::Dark,
        Some(false) => SystemTheme::Light,
        None => match SYSTEM_THEME.load(Ordering::Relaxed) {
            1 => SystemTheme::Light,
            2 => SystemTheme::Dark,
            _ => SystemTheme::Unknown,
        },
    }
}

pub(crate) fn resolve_configuration(
    configuration: &android_activity::ConfigurationRef,
) -> SystemTheme {
    use ndk::configuration::UiModeNight;

    let theme = match configuration.ui_mode_night() {
        UiModeNight::Yes => SystemTheme::Dark,
        UiModeNight::No => SystemTheme::Light,
        _ => SystemTheme::Unknown,
    };
    SYSTEM_THEME.store(
        match theme {
            SystemTheme::Light => 1,
            SystemTheme::Dark => 2,
            SystemTheme::Unknown => 0,
        },
        Ordering::Relaxed,
    );
    theme
}
