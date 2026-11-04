use crate::theme::SystemTheme;

pub(crate) fn resolve(is_dark: Option<bool>) -> SystemTheme {
    detect_linux_theme()
        .or_else(|| from_hint(is_dark))
        .unwrap_or_default()
}

fn detect_linux_theme() -> Option<SystemTheme> {
    if is_kde_desktop()
        && let Some(theme) = detect_kde_theme()
    {
        return Some(theme);
    }

    if let Some(value) = command_output(
        "gsettings",
        &["get", "org.gnome.desktop.interface", "color-scheme"],
    ) && let Some(theme) = parse_preference(&value)
    {
        return Some(theme);
    }

    if let Some(value) = command_output(
        "gsettings",
        &["get", "org.gnome.desktop.interface", "gtk-theme"],
    ) && let Some(theme) = parse_theme_name(&value)
    {
        return Some(theme);
    }

    if let Some(value) = std::env::var_os("GTK_THEME")
        && let Some(theme) = parse_theme_name(&value.to_string_lossy())
    {
        return Some(theme);
    }

    detect_kde_theme()
}

fn is_kde_desktop() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .is_some_and(|desktop| {
            desktop
                .split(':')
                .any(|name| name.to_ascii_lowercase().contains("kde"))
        })
}

fn detect_kde_theme() -> Option<SystemTheme> {
    for command in ["kreadconfig6", "kreadconfig5"] {
        if let Some(background) = command_output(
            command,
            &[
                "--file",
                "kdeglobals",
                "--group",
                "Colors:Window",
                "--key",
                "BackgroundNormal",
            ],
        ) && let Some(theme) = parse_kde_background(&background)
        {
            return Some(theme);
        }

        if let Some(value) = command_output(
            command,
            &[
                "--file",
                "kdeglobals",
                "--group",
                "General",
                "--key",
                "ColorScheme",
            ],
        ) && let Some(theme) = parse_theme_name(&value)
        {
            return Some(theme);
        }
    }
    None
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn parse_preference(value: &str) -> Option<SystemTheme> {
    let value = value
        .trim_matches(|ch: char| ch == '\'' || ch == '"')
        .to_ascii_lowercase();
    match value.as_str() {
        "prefer-dark" => Some(SystemTheme::Dark),
        "prefer-light" => Some(SystemTheme::Light),
        _ => None,
    }
}

fn parse_theme_name(value: &str) -> Option<SystemTheme> {
    let value = value.to_ascii_lowercase();
    if value.contains("dark") || value.contains("night") {
        Some(SystemTheme::Dark)
    } else if value.contains("light") {
        Some(SystemTheme::Light)
    } else {
        None
    }
}

fn parse_kde_background(value: &str) -> Option<SystemTheme> {
    let rgb = value
        .split(',')
        .map(|component| component.trim().parse::<u8>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if rgb.len() != 3 {
        return None;
    }
    let luminance =
        (2126 * u32::from(rgb[0]) + 7152 * u32::from(rgb[1]) + 722 * u32::from(rgb[2])) / 10_000;
    Some(if luminance < 128 {
        SystemTheme::Dark
    } else {
        SystemTheme::Light
    })
}

fn from_hint(is_dark: Option<bool>) -> Option<SystemTheme> {
    is_dark.map(|dark| {
        if dark {
            SystemTheme::Dark
        } else {
            SystemTheme::Light
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{SystemTheme, command_output, parse_preference, parse_theme_name, resolve};

    #[test]
    fn parses_gnome_color_scheme_preference() {
        assert_eq!(parse_preference("'prefer-dark'"), Some(SystemTheme::Dark));
        assert_eq!(parse_preference("'prefer-light'"), Some(SystemTheme::Light));
        assert_eq!(parse_preference("'default'"), None);
    }

    #[test]
    fn parses_gtk_and_kde_theme_names() {
        assert_eq!(parse_theme_name("Adwaita-dark"), Some(SystemTheme::Dark));
        assert_eq!(parse_theme_name("Breeze Light"), Some(SystemTheme::Light));
    }

    #[test]
    fn desktop_preference_overrides_winit_light_fallback() {
        if let Some(preference) = command_output(
            "gsettings",
            &["get", "org.gnome.desktop.interface", "color-scheme"],
        ) && let Some(expected) = parse_preference(&preference)
        {
            assert_eq!(resolve(Some(false)), expected);
        }
    }
}
