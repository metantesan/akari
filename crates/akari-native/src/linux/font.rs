use std::process::Command;

pub(crate) fn system_fonts() -> crate::font::SystemFonts {
    let (family, path) = system_sans_serif();
    crate::font::SystemFonts::from_path(path.as_deref(), family)
}

fn system_sans_serif() -> (Option<String>, Option<std::path::PathBuf>) {
    let Ok(output) = Command::new("fc-match")
        .args(["--format=%{family}\n%{file}\n", "sans-serif"])
        .output()
    else {
        return (None, None);
    };
    if !output.status.success() {
        return (None, None);
    }
    let output = String::from_utf8_lossy(&output.stdout);
    let mut lines = output.lines();
    let family = lines
        .next()
        .map(|family| family.split(',').next().unwrap_or(family).trim().to_owned())
        .filter(|family| !family.is_empty());
    let path = lines
        .next()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(std::path::PathBuf::from);
    (family, path)
}
