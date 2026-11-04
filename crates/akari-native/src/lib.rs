//! Native operating-system services shared by Akari applications.
//!
//! Each supported operating system has an adapter directory. The selected
//! adapter is determined at compile time, keeping platform APIs out of widgets.

#[cfg(target_os = "android")]
#[path = "android/mod.rs"]
mod selected;
#[cfg(target_os = "ios")]
#[path = "ios/mod.rs"]
mod selected;
#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod selected;
#[cfg(target_os = "macos")]
#[path = "mac/mod.rs"]
mod selected;
#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod selected;

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
    pub use crate::font::{SystemFontProvider, SystemFonts, system_fonts};
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
        crate::selected::theme::resolve(is_dark)
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

pub use selected::Platform as CurrentPlatform;

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
