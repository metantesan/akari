#[path = "build_support/svg.rs"]
mod svg;

use svg::{module_name, number, parse_svg};

use handlebars::Handlebars;
use serde_json::json;
use std::{collections::HashSet, env, fs, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let icon_dir = manifest.join("icons");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    let template_path = manifest.join("templates/lucide.rs.hbs");
    println!("cargo:rerun-if-changed={}", icon_dir.display());
    println!("cargo:rerun-if-changed={}", template_path.display());

    let mut files = fs::read_dir(&icon_dir)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", icon_dir.display()))
        .map(|entry| entry.expect("icon directory entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "svg"))
        .collect::<Vec<_>>();
    files.sort();
    assert!(!files.is_empty(), "Lucide SVG source directory is empty");

    let mut modules = HashSet::new();
    let mut icons = Vec::with_capacity(files.len());
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
        let icon_name = file
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("UTF-8 Lucide icon filename");
        let module = module_name(icon_name);
        assert!(
            modules.insert(module.clone()),
            "multiple Lucide SVG names map to Rust module {module}"
        );
        let svg = fs::read_to_string(file)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", file.display()));
        let (paths, circles) = parse_svg(&svg, file);
        assert!(
            !paths.is_empty() || !circles.is_empty(),
            "{} contains no drawable shapes",
            file.display()
        );
        icons.push(json!({
            "name": module,
            "svg_name": format!("lucide/{icon_name}"),
            "paths": paths.iter().map(|path| format!("{path:?}")).collect::<Vec<_>>(),
            "circles": circles.iter().map(|(x, y, radius)| {
                format!("({}, {}, {})", number(*x), number(*y), number(*radius))
            }).collect::<Vec<_>>(),
        }));
    }
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    let template = fs::read_to_string(&template_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", template_path.display()));
    let generated = handlebars
        .render_template(&template, &json!({ "icons": icons, "count": files.len() }))
        .expect("render Lucide Rust template");
    fs::write(output.join("lucide.rs"), generated).expect("write generated Lucide modules");
}
