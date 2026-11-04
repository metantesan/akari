use std::{collections::BTreeMap, path::Path};

pub(super) fn module_name(icon_name: &str) -> String {
    let mut name = icon_name.replace('-', "_");
    if name.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        name.insert(0, '_');
    }
    const KEYWORDS: &[&str] = &[
        "as", "async", "await", "become", "box", "break", "const", "continue", "crate", "do",
        "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in",
        "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
        "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
        "union", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
    ];
    if KEYWORDS.contains(&name.as_str()) {
        name.push('_');
    }
    name
}

pub(super) fn parse_svg(svg: &str, file: &Path) -> (Vec<String>, Vec<(f32, f32, f32)>) {
    let mut paths = Vec::new();
    let mut circles = Vec::new();
    let mut rest = svg;
    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];
        let end = tag_end(rest).unwrap_or_else(|| panic!("unterminated tag in {}", file.display()));
        let tag = rest[..end].trim();
        rest = &rest[end + 1..];
        if tag.is_empty() || tag.starts_with(['/', '!', '?']) {
            continue;
        }
        let (name, attrs) = attributes(tag, file);
        match name.as_str() {
            "svg" => {}
            "path" => paths.push(required(&attrs, "d", file).to_owned()),
            "circle" => circles.push((
                optional_number(&attrs, "cx", 0., file),
                optional_number(&attrs, "cy", 0., file),
                required_number(&attrs, "r", file),
            )),
            "line" => paths.push(format!(
                "M{} {}L{} {}",
                number(required_number(&attrs, "x1", file)),
                number(required_number(&attrs, "y1", file)),
                number(required_number(&attrs, "x2", file)),
                number(required_number(&attrs, "y2", file))
            )),
            "polyline" | "polygon" => {
                paths.push(points_path(
                    required(&attrs, "points", file),
                    name == "polygon",
                    file,
                ));
            }
            "rect" => paths.push(rect_path(&attrs, file)),
            "ellipse" => {
                let cx = required_number(&attrs, "cx", file);
                let cy = required_number(&attrs, "cy", file);
                let rx = required_number(&attrs, "rx", file);
                let ry = required_number(&attrs, "ry", file);
                if (rx - ry).abs() < f32::EPSILON {
                    circles.push((cx, cy, rx));
                } else {
                    paths.push(format!(
                        "M{} {}A{} {} 0 1 0 {} {}A{} {} 0 1 0 {} {}",
                        number(cx - rx),
                        number(cy),
                        number(rx),
                        number(ry),
                        number(cx + rx),
                        number(cy),
                        number(rx),
                        number(ry),
                        number(cx - rx),
                        number(cy)
                    ));
                }
            }
            other => panic!("unsupported SVG element <{other}> in {}", file.display()),
        }
    }
    (paths, circles)
}

fn tag_end(tag: &str) -> Option<usize> {
    let mut quote = None;
    for (index, character) in tag.char_indices() {
        if quote == Some(character) {
            quote = None;
        } else if quote.is_none() && matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if quote.is_none() && character == '>' {
            return Some(index);
        }
    }
    None
}

fn attributes(tag: &str, file: &Path) -> (String, BTreeMap<String, String>) {
    let bytes = tag.as_bytes();
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    let name_start = index;
    while index < bytes.len() && !bytes[index].is_ascii_whitespace() && bytes[index] != b'/' {
        index += 1;
    }
    let name = tag[name_start..index].to_owned();
    let mut attrs = BTreeMap::new();
    while index < bytes.len() {
        while index < bytes.len() && (bytes[index].is_ascii_whitespace() || bytes[index] == b'/') {
            index += 1;
        }
        if index == bytes.len() {
            break;
        }
        let key_start = index;
        while index < bytes.len() && bytes[index] != b'=' && !bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let key = &tag[key_start..index];
        while index < bytes.len() && (bytes[index].is_ascii_whitespace() || bytes[index] == b'=') {
            index += 1;
        }
        if index == bytes.len() || !matches!(bytes[index], b'\'' | b'"') {
            panic!("malformed attribute in {}: {tag}", file.display());
        }
        let quote = bytes[index];
        index += 1;
        let value_start = index;
        while index < bytes.len() && bytes[index] != quote {
            index += 1;
        }
        if index == bytes.len() {
            panic!("unterminated attribute in {}: {tag}", file.display());
        }
        attrs.insert(key.to_owned(), tag[value_start..index].to_owned());
        index += 1;
    }
    (name, attrs)
}

fn required<'a>(attrs: &'a BTreeMap<String, String>, key: &str, file: &Path) -> &'a str {
    attrs
        .get(key)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("missing {key} attribute in {}", file.display()))
}

fn number_value(value: &str, file: &Path) -> f32 {
    let parsed = value.parse::<f32>().unwrap_or_else(|error| {
        panic!(
            "invalid numeric SVG value {value:?} in {}: {error}",
            file.display()
        )
    });
    assert!(
        parsed.is_finite(),
        "non-finite SVG number in {}",
        file.display()
    );
    parsed
}

fn required_number(attrs: &BTreeMap<String, String>, key: &str, file: &Path) -> f32 {
    number_value(required(attrs, key, file), file)
}

fn optional_number(attrs: &BTreeMap<String, String>, key: &str, default: f32, file: &Path) -> f32 {
    attrs
        .get(key)
        .map(|value| number_value(value, file))
        .unwrap_or(default)
}

pub(super) fn number(value: f32) -> String {
    if value == 0. {
        return "0.0".to_owned();
    }
    let value = value.to_string();
    if value.contains(['.', 'e', 'E']) {
        value
    } else {
        format!("{value}.0")
    }
}

fn points_path(points: &str, close: bool, file: &Path) -> String {
    let values = points
        .split(|character: char| character.is_ascii_whitespace() || character == ',')
        .filter(|part| !part.is_empty())
        .map(|part| number_value(part, file))
        .collect::<Vec<_>>();
    assert!(
        values.len() >= 4 && values.len() % 2 == 0,
        "invalid points in {}",
        file.display()
    );
    let mut path = format!("M{} {}", number(values[0]), number(values[1]));
    for pair in values[2..].chunks_exact(2) {
        path.push_str(&format!("L{} {}", number(pair[0]), number(pair[1])));
    }
    if close {
        path.push('Z');
    }
    path
}

fn rect_path(attrs: &BTreeMap<String, String>, file: &Path) -> String {
    let x = optional_number(attrs, "x", 0., file);
    let y = optional_number(attrs, "y", 0., file);
    let width = required_number(attrs, "width", file);
    let height = required_number(attrs, "height", file);
    let mut rx = optional_number(attrs, "rx", 0., file);
    let mut ry = optional_number(attrs, "ry", rx, file);
    if rx == 0. || ry == 0. {
        rx = 0.;
        ry = 0.;
    }
    rx = rx.min(width * 0.5);
    ry = ry.min(height * 0.5);
    if rx == 0. {
        format!(
            "M{} {}H{}V{}H{}Z",
            number(x),
            number(y),
            number(x + width),
            number(y + height),
            number(x)
        )
    } else {
        format!(
            "M{} {}H{}A{} {} 0 0 1 {} {}V{}A{} {} 0 0 1 {} {}H{}A{} {} 0 0 1 {} {}V{}A{} {} 0 0 1 {} {}Z",
            number(x + rx),
            number(y),
            number(x + width - rx),
            number(rx),
            number(ry),
            number(x + width),
            number(y + ry),
            number(y + height - ry),
            number(rx),
            number(ry),
            number(x + width - rx),
            number(y + height),
            number(x + rx),
            number(rx),
            number(ry),
            number(x),
            number(y + height - ry),
            number(y + ry),
            number(rx),
            number(ry),
            number(x + rx),
            number(y)
        )
    }
}
