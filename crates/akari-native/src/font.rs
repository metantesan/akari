use std::{path::Path, sync::Arc};

/// Font faces selected by the active operating system and framework fallback.
#[derive(Clone, Debug)]
pub struct SystemFonts {
    family: String,
    faces: Vec<Arc<Vec<u8>>>,
}

impl SystemFonts {
    #[cfg(any(target_os = "android", target_os = "linux"))]
    pub(crate) fn from_path(path: Option<&Path>, family_hint: Option<String>) -> Self {
        let primary = path.and_then(|path| std::fs::read(path).ok());
        Self::from_bytes(primary, family_hint)
    }

    pub(crate) fn from_bytes(primary: Option<Vec<u8>>, family_hint: Option<String>) -> Self {
        let fallback_latin = Arc::new(include_bytes!("../fonts/NotoSans-Regular.ttf").to_vec());
        let fallback_arabic =
            Arc::new(include_bytes!("../fonts/NotoSansArabic-Regular.ttf").to_vec());

        let family = primary
            .as_deref()
            .and_then(font_family)
            .or(family_hint)
            .unwrap_or_else(|| "Noto Sans".to_owned());

        let mut faces = Vec::with_capacity(3);
        if let Some(primary) = primary {
            faces.push(Arc::new(primary));
        }
        faces.push(fallback_latin);
        faces.push(fallback_arabic);
        Self { family, faces }
    }

    /// The operating-system interface family selected for Akari's sans-serif text.
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Font faces to register with the renderer before the first frame.
    pub fn faces(&self) -> &[Arc<Vec<u8>>] {
        &self.faces
    }
}

/// Load the fonts selected by this platform's native provider.
pub fn system_fonts() -> SystemFonts {
    crate::font::SystemFontProvider::system_fonts(&crate::current())
}

/// Load system fonts using the active platform context when one is required.
pub fn from_event_loop(event_loop: &winit::event_loop::ActiveEventLoop) -> SystemFonts {
    crate::font::SystemFontProvider::system_fonts_from_event_loop(&crate::current(), event_loop)
}

/// Supplies the selected system font and framework fallback faces.
pub trait SystemFontProvider {
    /// Resolve the current system UI font and return usable fallback faces.
    fn system_fonts(&self) -> SystemFonts;

    /// Resolve fonts with access to platform context; defaults to a context-free lookup.
    fn system_fonts_from_event_loop(
        &self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> SystemFonts {
        self.system_fonts()
    }
}

pub(crate) fn font_family(bytes: &[u8]) -> Option<String> {
    let mut database = fontdb::Database::new();
    database.load_font_data(bytes.to_vec());
    database
        .faces()
        .next()
        .and_then(|face| face.families.first().map(|(family, _)| family.clone()))
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) fn read_font(path: &Path) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) fn apple_system_font() -> (Option<String>, Option<Vec<u8>>) {
    use std::ffi::{c_char, c_void};

    type CfTypeRef = *const c_void;
    type CfIndex = isize;

    #[link(name = "CoreText", kind = "framework")]
    unsafe extern "C" {
        fn CTFontCreateUIFontForLanguage(ui_type: u32, size: f64, language: CfTypeRef)
        -> CfTypeRef;
        fn CTFontCopyFamilyName(font: CfTypeRef) -> CfTypeRef;
        fn CTFontCopyAttribute(font: CfTypeRef, attribute: CfTypeRef) -> CfTypeRef;
        static kCTFontURLAttribute: CfTypeRef;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFURLCopyFileSystemPath(url: CfTypeRef, path_style: u32) -> CfTypeRef;
        fn CFStringGetLength(value: CfTypeRef) -> CfIndex;
        fn CFStringGetMaximumSizeForEncoding(length: CfIndex, encoding: u32) -> CfIndex;
        fn CFStringGetCString(
            value: CfTypeRef,
            buffer: *mut c_char,
            buffer_size: CfIndex,
            encoding: u32,
        ) -> bool;
        fn CFRelease(value: CfTypeRef);
    }

    fn string(value: CfTypeRef) -> Option<String> {
        if value.is_null() {
            return None;
        }
        const UTF8: u32 = 0x0800_0100;
        let capacity = unsafe { CFStringGetMaximumSizeForEncoding(CFStringGetLength(value), UTF8) }
            .checked_add(1)? as usize;
        let mut buffer = vec![0_u8; capacity];
        let ok = unsafe {
            CFStringGetCString(value, buffer.as_mut_ptr().cast(), capacity as CfIndex, UTF8)
        };
        if !ok {
            return None;
        }
        let value = std::ffi::CStr::from_bytes_until_nul(&buffer).ok()?;
        value.to_str().ok().map(str::to_owned)
    }

    // Core Text's kCTFontUIFontSystem selector is 2 on macOS and iOS.
    let font = unsafe { CTFontCreateUIFontForLanguage(2, 0.0, std::ptr::null()) };
    if font.is_null() {
        return (None, None);
    }
    let family_ref = unsafe { CTFontCopyFamilyName(font) };
    let family = string(family_ref);
    if !family_ref.is_null() {
        unsafe { CFRelease(family_ref) };
    }
    let url = unsafe { CTFontCopyAttribute(font, kCTFontURLAttribute) };
    let path_ref = if url.is_null() {
        std::ptr::null()
    } else {
        unsafe { CFURLCopyFileSystemPath(url, 0) }
    };
    let path = string(path_ref).map(std::path::PathBuf::from);
    for reference in [path_ref, url, font] {
        if !reference.is_null() {
            unsafe { CFRelease(reference) };
        }
    }
    let bytes = path.as_deref().and_then(read_font);
    (family, bytes)
}
