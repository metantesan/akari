use crate::theme::SystemTheme;

pub(crate) fn resolve(is_dark: Option<bool>) -> SystemTheme {
    match is_dark {
        Some(true) => SystemTheme::Dark,
        Some(false) => SystemTheme::Light,
        None => SystemTheme::Unknown,
    }
}

pub(crate) fn resolve_configuration(
    configuration: &android_activity::ConfigurationRef,
) -> SystemTheme {
    use ndk::configuration::UiModeNight;

    match configuration.ui_mode_night() {
        UiModeNight::Yes => SystemTheme::Dark,
        UiModeNight::No => SystemTheme::Light,
        _ => SystemTheme::Unknown,
    }
}
