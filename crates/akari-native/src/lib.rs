//! Native operating-system services shared by Akari applications.
//!
//! Each supported operating system has a native adapter directory. The
//! adapter is determined at compile time, keeping platform APIs out of widgets.

#[cfg(target_os = "android")]
#[path = "android/mod.rs"]
mod native;
#[cfg(target_os = "ios")]
#[path = "ios/mod.rs"]
mod native;
#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod native;
#[cfg(target_os = "macos")]
#[path = "mac/mod.rs"]
mod native;
#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod native;

#[cfg(not(any(
    target_os = "android",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "windows"
)))]
compile_error!("Akari native services do not support this target yet");

pub mod clipboard {
    use std::{error::Error, fmt};

    #[derive(Debug)]
    pub struct ClipboardError(pub(crate) String);

    impl fmt::Display for ClipboardError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl Error for ClipboardError {}

    pub fn get_text() -> Result<String, ClipboardError> {
        crate::Clipboard::get_text(&mut crate::current())
    }

    pub fn set_text(text: impl Into<String>) -> Result<(), ClipboardError> {
        crate::Clipboard::set_text(&mut crate::current(), &text.into())
    }
}

pub mod localization;

mod font;

/// Native system-font lookup and framework-owned fallback faces.
pub mod fonts {
    pub use crate::font::{SystemFontProvider, SystemFonts, from_event_loop, system_fonts};
}

/// Returns system-preferred BCP 47 language tags, most preferred first.
pub fn system_locales() -> Vec<String> {
    LocaleProvider::system_locales(&current())
}

pub mod theme {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub enum SystemTheme {
        Light,
        Dark,
        #[default]
        Unknown,
    }

    pub fn from_dark(is_dark: Option<bool>) -> SystemTheme {
        crate::native::theme::resolve(is_dark)
    }

    pub fn from_event_loop(event_loop: &winit::event_loop::ActiveEventLoop) -> SystemTheme {
        crate::ThemeProvider::system_theme_from_event_loop(&crate::current(), event_loop)
    }

    pub fn update_from_window(window: &winit::window::Window, current: SystemTheme) -> SystemTheme {
        crate::ThemeProvider::system_theme_from_window(&crate::current(), window, current)
    }
}

/// Safe-area insets for a native window, expressed in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WindowInsets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Native window inset lookup.
pub mod window_insets {
    use crate::WindowInsets;
    use winit::{event_loop::ActiveEventLoop, window::Window};

    pub fn for_window(window: &Window) -> WindowInsets {
        crate::WindowInsetsProvider::window_insets(&crate::current(), window)
    }

    pub fn from_event_loop(event_loop: &ActiveEventLoop, window: &Window) -> WindowInsets {
        crate::WindowInsetsProvider::window_insets_from_event_loop(
            &crate::current(),
            event_loop,
            window,
        )
    }
}

/// Provides the visible content bounds for a native window.
pub trait WindowInsetsProvider {
    fn window_insets(&self, window: &winit::window::Window) -> WindowInsets;

    fn window_insets_from_event_loop(
        &self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        window: &winit::window::Window,
    ) -> WindowInsets {
        self.window_insets(window)
    }
}

/// Native plain-text clipboard access.
pub trait Clipboard {
    fn get_text(&mut self) -> Result<String, clipboard::ClipboardError>;
    fn set_text(&mut self, text: &str) -> Result<(), clipboard::ClipboardError>;
}

/// Platform appearance information, normalized to Akari's light/dark palette.
pub trait ThemeProvider {
    fn system_theme(&self, hint: Option<bool>) -> theme::SystemTheme;

    fn system_theme_from_event_loop(
        &self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> theme::SystemTheme {
        self.system_theme(
            event_loop
                .system_theme()
                .map(|theme| theme == winit::window::Theme::Dark),
        )
    }

    fn system_theme_from_window(
        &self,
        _window: &winit::window::Window,
        current: theme::SystemTheme,
    ) -> theme::SystemTheme {
        current
    }
}

/// Native ordered locale preferences, returned as BCP 47 tags.
pub trait LocaleProvider {
    fn system_locales(&self) -> Vec<String>;
}

/// Standard per-application locations for persistent, configuration, and
/// temporary files. `application_id` should be stable and app-specific.
pub trait PlatformDirectories {
    fn config_dir(&self, application_id: &str) -> Result<std::path::PathBuf, DirectoryError>;
    fn data_dir(&self, application_id: &str) -> Result<std::path::PathBuf, DirectoryError>;
    fn cache_dir(&self, application_id: &str) -> Result<std::path::PathBuf, DirectoryError>;
}

#[derive(Debug)]
pub struct DirectoryError(pub String);

impl std::fmt::Display for DirectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DirectoryError {}

pub use native::Platform as CurrentPlatform;

pub fn current() -> CurrentPlatform {
    CurrentPlatform
}

fn app_segment(application_id: &str) -> String {
    let segment: String = application_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if segment.is_empty() || segment == "." || segment == ".." {
        "app".to_owned()
    } else {
        segment
    }
}

#[cfg(test)]
mod tests {
    use super::{PlatformDirectories, current};

    #[test]
    fn app_directory_names_remain_inside_the_platform_root() {
        let platform = current();
        let path = platform
            .data_dir("../My App")
            .expect("platform data directory");
        assert!(path.ends_with(".._My_App"));
    }
}
