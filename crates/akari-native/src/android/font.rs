use std::{fs, path::PathBuf};

pub(crate) fn system_fonts() -> crate::font::SystemFonts {
    let path = configured_sans_serif();
    crate::font::SystemFonts::from_path(path.as_deref(), None)
}

fn configured_sans_serif() -> Option<PathBuf> {
    let config = [
        "/system/etc/fonts.xml",
        "/product/etc/fonts_customization.xml",
        "/system/etc/fonts_customization.xml",
    ]
    .into_iter()
    .find_map(|path| fs::read_to_string(path).ok())?;

    let family = config.split("<family").skip(1).find(|block| {
        block
            .split('>')
            .next()
            .is_some_and(|attributes| attributes.contains("name=\"sans-serif\""))
    })?;
    let family = family.split_once("</family>")?.0;
    let font_name = family
        .split("<font")
        .skip(1)
        .filter_map(|font| {
            let (attributes, content) = font.split_once('>')?;
            let weight = attributes
                .split("weight=\"")
                .nth(1)
                .and_then(|value| value.split('"').next())
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(400);
            let filename = content.split('<').next()?.trim().to_owned();
            (!filename.is_empty()).then_some((weight.abs_diff(400), filename))
        })
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, filename)| filename)?;
    let path = PathBuf::from("/system/fonts").join(font_name);
    path.is_file().then_some(path)
}
