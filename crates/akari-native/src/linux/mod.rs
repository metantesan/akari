pub(crate) mod clipboard;
pub(crate) mod locale;
pub(crate) mod theme;

pub struct Platform;

impl crate::Clipboard for Platform {
    fn get_text(&mut self) -> Result<String, crate::clipboard::ClipboardError> {
        self::clipboard::get_text()
    }

    fn set_text(&mut self, text: &str) -> Result<(), crate::clipboard::ClipboardError> {
        self::clipboard::set_text(text.to_owned())
    }
}

impl crate::ThemeProvider for Platform {
    fn system_theme(&self, hint: Option<bool>) -> crate::theme::SystemTheme {
        theme::resolve(hint)
    }
}

impl crate::LocaleProvider for Platform {
    fn system_locales(&self) -> Vec<String> {
        locale::system_locales()
    }
}

impl crate::PlatformDirectories for Platform {
    fn config_dir(&self, app: &str) -> Result<std::path::PathBuf, crate::DirectoryError> {
        base("XDG_CONFIG_HOME", ".config").map(|path| path.join(crate::app_segment(app)))
    }

    fn data_dir(&self, app: &str) -> Result<std::path::PathBuf, crate::DirectoryError> {
        base("XDG_DATA_HOME", ".local/share").map(|path| path.join(crate::app_segment(app)))
    }

    fn cache_dir(&self, app: &str) -> Result<std::path::PathBuf, crate::DirectoryError> {
        base("XDG_CACHE_HOME", ".cache").map(|path| path.join(crate::app_segment(app)))
    }
}

fn base(variable: &str, home_suffix: &str) -> Result<std::path::PathBuf, crate::DirectoryError> {
    if let Some(path) = std::env::var_os(variable).filter(|value| !value.is_empty()) {
        return Ok(path.into());
    }
    let home =
        std::env::var_os("HOME").ok_or_else(|| crate::DirectoryError("HOME is not set".into()))?;
    Ok(std::path::PathBuf::from(home).join(home_suffix))
}
