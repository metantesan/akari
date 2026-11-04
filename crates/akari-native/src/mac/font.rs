pub(crate) fn system_fonts() -> crate::font::SystemFonts {
    let (family, bytes) = crate::font::apple_system_font();
    crate::font::SystemFonts::from_bytes(bytes, family)
}
