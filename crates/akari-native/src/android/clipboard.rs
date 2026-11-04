use crate::clipboard::ClipboardError;

pub(crate) fn get_text() -> Result<String, ClipboardError> {
    Err(ClipboardError(
        "Android clipboard host bridge is not connected".into(),
    ))
}

pub(crate) fn set_text(_: String) -> Result<(), ClipboardError> {
    Err(ClipboardError(
        "Android clipboard host bridge is not connected".into(),
    ))
}
