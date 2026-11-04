use crate::clipboard::ClipboardError;

pub(crate) fn get_text() -> Result<String, ClipboardError> {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.get_text())
        .map_err(|error| ClipboardError(error.to_string()))
}

pub(crate) fn set_text(text: String) -> Result<(), ClipboardError> {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(text))
        .map_err(|error| ClipboardError(error.to_string()))
}
