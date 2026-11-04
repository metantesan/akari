//! Fluent resource loading, locale negotiation, and translation fallback.

use std::collections::HashSet;

mod bundle;
mod error;

use bundle::format_in_bundle;
pub use error::LocalizationError;
use error::parse_locale;

pub use fluent::{FluentArgs, FluentValue, fluent_args};
use fluent::{FluentBundle, FluentResource};
pub use unic_langid::LanguageIdentifier;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

struct Bundle {
    locale: LanguageIdentifier,
    bundle: FluentBundle<FluentResource>,
}

/// Fluent-backed application localization with system locale negotiation.
///
/// Load one or more `.ftl` resources per locale. Formatting searches the
/// preferred system locales, then the configured fallback locale, and finally
/// any remaining loaded bundles.
pub struct Localization {
    preferred: Vec<LanguageIdentifier>,
    fallback: LanguageIdentifier,
    bundles: Vec<Bundle>,
}

impl Localization {
    /// Creates a localizer from the native system language list.
    pub fn from_system(fallback: &str) -> Result<Self, LocalizationError> {
        Self::new(crate::system_locales(), fallback)
    }

    /// Creates a localizer from explicitly ordered BCP 47 tags.
    pub fn new<I, S>(preferred: I, fallback: &str) -> Result<Self, LocalizationError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let fallback = parse_locale(fallback)?;
        let mut locales = Vec::new();
        let mut seen = HashSet::new();
        for locale in preferred {
            if let Ok(locale) = locale.as_ref().parse::<LanguageIdentifier>()
                && seen.insert(locale.clone())
            {
                locales.push(locale);
            }
        }
        if seen.insert(fallback.clone()) {
            locales.push(fallback.clone());
        }
        Ok(Self {
            preferred: locales,
            fallback,
            bundles: Vec::new(),
        })
    }

    /// Parses and adds an FTL resource to the locale's Fluent bundle.
    pub fn add_resource(&mut self, locale: &str, source: &str) -> Result<(), LocalizationError> {
        let locale = parse_locale(locale)?;
        let resource = FluentResource::try_new(source.to_owned()).map_err(|(_, errors)| {
            LocalizationError(format!("invalid Fluent resource: {errors:?}"))
        })?;
        if let Some(bundle) = self.bundles.iter_mut().find(|entry| entry.locale == locale) {
            bundle.bundle.add_resource(resource).map_err(|errors| {
                LocalizationError(format!("duplicate Fluent resource: {errors:?}"))
            })?;
        } else {
            let mut bundle = FluentBundle::new(vec![locale.clone()]);
            bundle.add_resource(resource).map_err(|errors| {
                LocalizationError(format!("invalid Fluent resource: {errors:?}"))
            })?;
            self.bundles.push(Bundle { locale, bundle });
        }
        Ok(())
    }

    /// The best loaded locale matching system preferences, or the configured
    /// fallback if it has not been loaded yet.
    pub fn locale(&self) -> &LanguageIdentifier {
        self.preferred
            .iter()
            .find_map(|preferred| self.find_bundle(preferred))
            .map_or(&self.fallback, |bundle| &bundle.locale)
    }

    /// Returns the writing direction implied by the selected locale's script
    /// or language.
    pub fn direction(&self) -> TextDirection {
        let locale = self.locale();
        if locale.script.as_ref().is_some_and(|script| {
            matches!(
                script.as_str(),
                "Arab" | "Hebr" | "Thaa" | "Nkoo" | "Adlm" | "Syrc"
            )
        }) || matches!(
            locale.language.as_str(),
            "ar" | "fa" | "he" | "ur" | "ps" | "sd" | "ug" | "yi" | "ku" | "dv" | "ks"
        ) {
            TextDirection::RightToLeft
        } else {
            TextDirection::LeftToRight
        }
    }

    /// Formats a message, searching preferred locales and then fallback
    /// bundles. Returns `None` if no loaded bundle contains the message.
    pub fn format(&self, id: &str, args: Option<&FluentArgs<'_>>) -> Option<String> {
        self.format_pattern(id, None, args)
    }

    /// Formats a Fluent message attribute, such as `.aria-label` or `.title`.
    pub fn attribute(
        &self,
        id: &str,
        attribute: &str,
        args: Option<&FluentArgs<'_>>,
    ) -> Option<String> {
        self.format_pattern(id, Some(attribute), args)
    }

    /// Formats a simple message without arguments.
    pub fn text(&self, id: &str) -> Option<String> {
        self.format(id, None)
    }

    fn format_pattern(
        &self,
        id: &str,
        attribute: Option<&str>,
        args: Option<&FluentArgs<'_>>,
    ) -> Option<String> {
        for preferred in &self.preferred {
            if let Some(bundle) = self.find_bundle(preferred)
                && let Some(value) = format_in_bundle(bundle, id, attribute, args)
            {
                return Some(value);
            }
        }
        for bundle in &self.bundles {
            if let Some(value) = format_in_bundle(bundle, id, attribute, args) {
                return Some(value);
            }
        }
        None
    }

    fn find_bundle(&self, locale: &LanguageIdentifier) -> Option<&Bundle> {
        self.bundles
            .iter()
            .find(|bundle| &bundle.locale == locale)
            .or_else(|| {
                self.bundles
                    .iter()
                    .find(|bundle| bundle.locale.language == locale.language)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{Localization, fluent_args};

    #[test]
    fn negotiates_system_locale_and_falls_back_per_message() {
        let mut localization =
            Localization::new(["fa-IR", "en-US"], "en-US").expect("valid locales");
        localization
            .add_resource("fa", "hello = سلام، { $name }\n")
            .expect("valid Persian FTL");
        localization
            .add_resource("en-US", "hello = Hello, { $name }\nbye = Goodbye\n")
            .expect("valid English FTL");

        assert_eq!(localization.locale().to_string(), "fa");
        let args = fluent_args!["name" => "دوست"];
        assert_eq!(
            localization.format("hello", Some(&args)).as_deref(),
            Some("سلام، \u{2068}دوست\u{2069}")
        );
        assert_eq!(localization.text("bye").as_deref(), Some("Goodbye"));
    }

    #[test]
    fn resolves_plural_selectors_using_fluent_rules() {
        let mut localization = Localization::new(["en-US"], "en-US").expect("valid locale");
        localization
            .add_resource(
                "en-US",
                "unread = { $count ->\n    [one] One unread message\n   *[other] { $count } unread messages\n}\n",
            )
            .expect("valid plural resource");
        let args = fluent_args!["count" => 3];
        assert_eq!(
            localization.format("unread", Some(&args)).as_deref(),
            Some("\u{2068}3\u{2069} unread messages")
        );
    }
}
