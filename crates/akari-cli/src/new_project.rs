use super::args::NewArgs;
use std::{
    env,
    error::Error,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

struct TemplateFile {
    path: &'static str,
    contents: &'static [u8],
}
include!(concat!(env!("OUT_DIR"), "/template.rs"));

#[derive(Debug)]
pub(super) struct NewOptions {
    pub(super) display_name: String,
    pub(super) package: String,
    pub(super) identifier: String,
    pub(super) output: PathBuf,
    pub(super) local_akari_path: Option<PathBuf>,
}

pub(super) fn options(args: NewArgs) -> Result<NewOptions, Box<dyn Error>> {
    let display_name = match args.name {
        Some(name) => name,
        None => {
            print!("App name: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_owned()
        }
    };
    if display_name.trim().is_empty() {
        return Err("app name cannot be empty".into());
    }
    let package = args.package.unwrap_or_else(|| slug(&display_name));
    validate_package(&package)?;
    let identifier = args
        .identifier
        .unwrap_or_else(|| format!("com.example.{}", package.replace('-', "_")));
    validate_identifier(&identifier)?;
    let local_akari_path = args
        .akari_path
        .map(|path| resolve_akari_path(&path))
        .transpose()?;
    let output = args.output.unwrap_or(env::current_dir()?.join(&package));
    let output = if output.is_absolute() {
        output
    } else {
        env::current_dir()?.join(output)
    };
    Ok(NewOptions {
        display_name,
        package,
        identifier,
        output,
        local_akari_path,
    })
}

fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut separator = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            if separator && !out.is_empty() {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
            separator = false;
        } else {
            separator = true;
        }
    }
    out.trim_matches('-').to_owned()
}

fn validate_package(name: &str) -> Result<(), Box<dyn Error>> {
    let mut chars = name.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_alphanumeric())
        || !chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("package name must start with an ASCII letter or digit and contain only letters, digits, `-`, or `_`".into());
    }
    Ok(())
}

fn validate_identifier(identifier: &str) -> Result<(), Box<dyn Error>> {
    if identifier.split('.').count() < 2
        || identifier.split('.').any(|part| {
            part.is_empty()
                || !part
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                || !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
    {
        return Err("application identifier must be dot-separated ASCII identifiers, for example `com.example.notes`".into());
    }
    Ok(())
}

fn resolve_akari_path(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let path = path.canonicalize()?;
    if path.join("Cargo.toml").is_file()
        && fs::read_to_string(path.join("Cargo.toml"))?
            .lines()
            .any(|l| l.trim() == "name = \"akari\"")
    {
        return Ok(path);
    }
    let package_path = path.join("crates/akari");
    if package_path.join("Cargo.toml").is_file() {
        return Ok(package_path);
    }
    Err(format!(
        "{} is not the Akari package or workspace root",
        path.display()
    )
    .into())
}

pub(super) fn scaffold(options: &NewOptions) -> Result<(), Box<dyn Error>> {
    if options.output.exists() {
        return Err(format!(
            "{} already exists; choose another output directory",
            options.output.display()
        )
        .into());
    }
    let parent = options
        .output
        .parent()
        .ok_or("output directory has no parent")?;
    fs::create_dir_all(parent)?;
    let staging = parent.join(format!(".akari-new-{}", std::process::id()));
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    fs::create_dir(&staging)?;
    let result = write_template(&staging, options).and_then(|()| {
        fs::rename(&staging, &options.output)?;
        Ok(())
    });
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn write_template(root: &Path, options: &NewOptions) -> Result<(), Box<dyn Error>> {
    let crate_name = options.package.replace('-', "_");
    let java_package_path = options.identifier.replace('.', "/");
    let mut replacements = vec![
        ("{{package_name}}", options.package.clone()),
        ("{{crate_name}}", crate_name),
        ("{{display_name}}", options.display_name.clone()),
        ("{{identifier}}", options.identifier.clone()),
        ("{{java_package_path}}", java_package_path),
    ];
    if let Some(path) = &options.local_akari_path {
        replacements.push((
            "{{akari_dependency}}",
            format!(
                "akari = {{ package = \"akari-gui\", path = \"{}\", optional = true }}",
                toml_string_path(path)
            ),
        ));
    } else {
        replacements.push((
            "{{akari_dependency}}",
            "akari = { git = \"https://github.com/metantesan/akari\", package = \"akari-gui\", optional = true }".to_owned(),
        ));
    }
    for file in TEMPLATE {
        let template_path = file.path.strip_suffix(".hbs").unwrap_or(file.path);
        let output_path = replace_all(template_path, &replacements);
        let destination = root.join(output_path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Ok(source) = std::str::from_utf8(file.contents) {
            fs::write(&destination, replace_all(source, &replacements))?;
        } else {
            fs::write(&destination, file.contents)?;
        }
        if template_path == "android/gradlew" {
            super::util::set_executable(&destination)?;
        }
    }
    Ok(())
}

fn replace_all(source: &str, replacements: &[(&'static str, String)]) -> String {
    replacements
        .iter()
        .fold(source.to_owned(), |text, (key, value)| {
            text.replace(key, value)
        })
}
fn toml_string_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_name_derives_normal_package_slug() {
        assert_eq!(slug("Daily Notes!"), "daily-notes");
    }
    #[test]
    fn validates_user_supplied_identifiers() {
        assert!(validate_identifier("com.example.notes").is_ok());
        assert!(validate_identifier("dev.akari.bad-id").is_err());
    }
    #[test]
    fn replaces_tokens() {
        assert_eq!(
            replace_all(
                "{{identifier}}/{{package_name}}",
                &[
                    ("{{identifier}}", "com.my.app".into()),
                    ("{{package_name}}", "my-app".into())
                ]
            ),
            "com.my.app/my-app"
        );
    }
}
