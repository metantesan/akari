use std::{ffi::c_void, mem, path::PathBuf};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, SystemParametersInfoW,
};

pub(crate) fn system_fonts() -> crate::font::SystemFonts {
    let family = system_message_font();
    let bytes = family.as_deref().and_then(font_bytes_for_family);
    crate::font::SystemFonts::from_bytes(bytes, family)
}

fn system_message_font() -> Option<String> {
    let mut metrics: NONCLIENTMETRICSW = unsafe { mem::zeroed() };
    metrics.cbSize = mem::size_of::<NONCLIENTMETRICSW>() as u32;
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            (&mut metrics as *mut NONCLIENTMETRICSW).cast::<c_void>(),
            0,
        )
    };
    if ok == 0 {
        return None;
    }
    let name = &metrics.lfMessageFont.lfFaceName;
    let length = name
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(name.len());
    Some(String::from_utf16_lossy(&name[..length]))
}

fn font_bytes_for_family(family: &str) -> Option<Vec<u8>> {
    let fonts = std::env::var_os("WINDIR")?.into_string().ok()?;
    let directory = PathBuf::from(fonts).join("Fonts");
    let normalized = family
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    let entries = std::fs::read_dir(directory).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
        if !matches!(extension.as_str(), "ttf" | "otf" | "ttc") {
            continue;
        }
        let stem = path.file_stem()?.to_string_lossy();
        let normalized_stem = stem
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>();
        if normalized_stem.starts_with(&normalized)
            && let Ok(bytes) = std::fs::read(path)
        {
            return Some(bytes);
        }
    }
    None
}
