use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let template = manifest.join("template/project");
    println!("cargo:rerun-if-changed={}", template.display());

    let mut files = Vec::new();
    collect(&template, &template, &mut files);
    files.sort();

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("output dir"));
    let mut generated =
        fs::File::create(out.join("template.rs")).expect("create embedded template");
    writeln!(generated, "const TEMPLATE: &[TemplateFile] = &[").expect("value must be present");
    for (relative, absolute) in files {
        println!("cargo:rerun-if-changed={}", absolute.display());
        let relative = relative
            .to_str()
            .expect("template path is UTF-8")
            .replace('\\', "/");
        writeln!(
            generated,
            "TemplateFile {{ path: {:?}, contents: include_bytes!({:?}) }},",
            relative,
            absolute.to_str().expect("template path is UTF-8"),
        )
        .expect("value must be present");
    }
    writeln!(generated, "]; ").expect("value must be present");
}

fn collect(root: &Path, directory: &Path, output: &mut Vec<(PathBuf, PathBuf)>) {
    let mut entries = fs::read_dir(directory)
        .expect("read template directory")
        .map(|entry| entry.expect("read template entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect(root, &path, output);
        } else if path.is_file() {
            output.push((
                path.strip_prefix(root)
                    .expect("value must be present")
                    .to_owned(),
                path,
            ));
        }
    }
}
