use super::LanguageIdentifier;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct LocalizationError(pub(super) String);

impl fmt::Display for LocalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for LocalizationError {}

pub(super) fn parse_locale(locale: &str) -> Result<LanguageIdentifier, LocalizationError> {
    locale
        .parse()
        .map_err(|error| LocalizationError(format!("invalid BCP 47 locale '{locale}': {error}")))
}
