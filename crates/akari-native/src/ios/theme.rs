use crate::theme::SystemTheme;

pub(crate) fn resolve(is_dark: Option<bool>) -> SystemTheme {
    match is_dark {
        Some(true) => SystemTheme::Dark,
        Some(false) => SystemTheme::Light,
        None => SystemTheme::Unknown,
    }
}
