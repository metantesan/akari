pub(crate) mod clipboard;
pub(crate) mod font;
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

    fn system_theme_from_event_loop(
        &self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> crate::theme::SystemTheme {
        use winit::platform::android::ActiveEventLoopExtAndroid;
        theme::resolve_configuration(&event_loop.android_app().config())
    }

    fn system_theme_from_window(
        &self,
        window: &winit::window::Window,
        _current: crate::theme::SystemTheme,
    ) -> crate::theme::SystemTheme {
        use winit::platform::android::WindowExtAndroid;
        theme::resolve_configuration(&window.config())
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

    fn system_fonts_from_event_loop(
        &self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> crate::fonts::SystemFonts {
        use winit::platform::android::ActiveEventLoopExtAndroid;
        let config = event_loop.android_app().config();
        if config.sdk_version() >= 29 {
            match font::system_fonts_from_ndk() {
                Some(fonts) => fonts,
                None => font::system_fonts(),
            }
        } else {
            font::system_fonts()
        }
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
