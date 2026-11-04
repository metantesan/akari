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
        Ok(home()?.join("files/config").join(crate::app_segment(app)))
    }

    fn data_dir(&self, app: &str) -> Result<std::path::PathBuf, crate::DirectoryError> {
        Ok(home()?.join("files").join(crate::app_segment(app)))
    }

    fn cache_dir(&self, app: &str) -> Result<std::path::PathBuf, crate::DirectoryError> {
        Ok(home()?.join("cache").join(crate::app_segment(app)))
    }
}

fn home() -> Result<std::path::PathBuf, crate::DirectoryError> {
    std::env::var_os("HOME")
        .map(Into::into)
        .ok_or_else(|| crate::DirectoryError("Android app-private HOME is unavailable".into()))
}
