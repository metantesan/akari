use super::{Bundle, FluentArgs};

pub(super) fn format_in_bundle(
    bundle: &Bundle,
    id: &str,
    attribute: Option<&str>,
    args: Option<&FluentArgs<'_>>,
) -> Option<String> {
    let message = bundle.bundle.get_message(id)?;
    let pattern = if let Some(attribute) = attribute {
        message
            .attributes()
            .find(|item| item.id() == attribute)?
            .value()
    } else {
        message.value()?
    };
    let mut errors = Vec::new();
    Some(
        bundle
            .bundle
            .format_pattern(pattern, args, &mut errors)
            .into_owned(),
    )
}
