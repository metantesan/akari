pub(crate) mod clipboard;
pub(crate) mod font;
pub(crate) mod locale;
pub(crate) mod theme;

pub struct Platform;

impl crate::WindowInsetsProvider for Platform {
    fn window_insets(&self, _: &winit::window::Window) -> crate::WindowInsets {
        crate::WindowInsets::default()
    }
}

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

impl crate::font::SystemFontProvider for Platform {
    fn system_fonts(&self) -> crate::fonts::SystemFonts {
        font::system_fonts()
    }
}

impl crate::PlatformDirectories for Platform {
    fn config_dir(&self, app: &str) -> Result<std::path::PathBuf, crate::DirectoryError> {
        Ok(base("APPDATA")?.join(crate::app_segment(app)))
    }

    fn data_dir(&self, app: &str) -> Result<std::path::PathBuf, crate::DirectoryError> {
        Ok(base("LOCALAPPDATA")?.join(crate::app_segment(app)))
    }

    fn cache_dir(&self, app: &str) -> Result<std::path::PathBuf, crate::DirectoryError> {
        Ok(base("LOCALAPPDATA")?
            .join(crate::app_segment(app))
            .join("cache"))
    }
}

fn base(variable: &str) -> Result<std::path::PathBuf, crate::DirectoryError> {
    std::env::var_os(variable)
        .map(Into::into)
        .ok_or_else(|| crate::DirectoryError(format!("{variable} is not set")))
}
