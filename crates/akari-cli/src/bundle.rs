use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(super) fn bundle(
    project: Option<PathBuf>,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let project = super::build::project_dir(project)?;
    match env::consts::OS {
        "linux" => bundle_appimage(&project, output),
        "macos" => bundle_pkg(&project, output),
        os => Err(format!("bundling is not configured for host OS `{os}`").into()),
    }
}

fn command_exists(command: &str) -> bool {
    Command::new(command).arg("--help").output().is_ok()
}

fn bundle_appimage(project: &Path, output: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let metadata = fs::read_to_string(project.join("Cargo.toml"))?;
    let name = metadata
        .lines()
        .find_map(|line| {
            line.strip_prefix("name = \"")
                .and_then(|v| v.strip_suffix('\"'))
        })
        .ok_or("cannot read Cargo package name")?;
    super::build::run_status(
        Command::new("cargo")
            .current_dir(project)
            .args(["build", "--release"]),
    )?;
    let binary = project.join("target/release").join(name);
    let appdir = project
        .join("target/appimage")
        .join(format!("{name}.AppDir"));
    if appdir.exists() {
        fs::remove_dir_all(&appdir)?;
    }
    fs::create_dir_all(appdir.join("usr/bin"))?;
    fs::create_dir_all(appdir.join("usr/share/applications"))?;
    fs::create_dir_all(appdir.join("usr/share/icons/hicolor/scalable/apps"))?;
    fs::copy(binary, appdir.join("usr/bin").join(name))?;
    fs::copy(
        project.join("bundle/icon.svg"),
        appdir.join(format!("usr/share/icons/hicolor/scalable/apps/{name}.svg")),
    )?;
    fs::copy(
        project.join(format!("bundle/{name}.desktop")),
        appdir.join(format!("usr/share/applications/{name}.desktop")),
    )?;
    fs::copy(
        project.join(format!("bundle/{name}.desktop")),
        appdir.join(format!("{name}.desktop")),
    )?;
    fs::write(
        appdir.join("AppRun"),
        format!(
            "#!/bin/sh\nHERE=\"$(dirname \"$(readlink -f \"$0\")\")\"\nexec \"$HERE/usr/bin/{name}\" \"$@\"\n"
        ),
    )?;
    super::util::set_executable(&appdir.join("AppRun"))?;
    let output = output.unwrap_or_else(|| project.join(format!("{name}-x86_64.AppImage")));
    let tool = if command_exists("linuxdeploy") {
        "linuxdeploy"
    } else if command_exists("appimagetool") {
        "appimagetool"
    } else {
        return Err("install `linuxdeploy` or `appimagetool` to create the final AppImage; the AppDir is ready under target/appimage".into());
    };
    if tool == "linuxdeploy" {
        super::build::run_status(
            Command::new(tool)
                .arg("--appdir")
                .arg(&appdir)
                .arg("--output")
                .arg("appimage")
                .env("ARCH", "x86_64"),
        )?;
    } else {
        super::build::run_status(
            Command::new(tool)
                .arg(&appdir)
                .arg(&output)
                .env("ARCH", "x86_64"),
        )?;
    }
    Ok(())
}

fn bundle_pkg(project: &Path, output: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let metadata = fs::read_to_string(project.join("Cargo.toml"))?;
    let name = metadata
        .lines()
        .find_map(|line| {
            line.strip_prefix("name = \"")
                .and_then(|v| v.strip_suffix('\"'))
        })
        .ok_or("cannot read Cargo package name")?;
    let identifier = metadata
        .lines()
        .find_map(|line| {
            line.strip_prefix("application-id = \"")
                .and_then(|value| value.strip_suffix('\"'))
        })
        .ok_or("cannot read [package.metadata.akari] application-id")?;
    super::build::run_status(
        Command::new("cargo")
            .current_dir(project)
            .args(["build", "--release"]),
    )?;
    let payload = project
        .join("target/pkg-root/Applications")
        .join(format!("{name}.app"));
    fs::create_dir_all(payload.join("Contents/MacOS"))?;
    fs::copy(
        project.join("target/release").join(name),
        payload.join("Contents/MacOS").join(name),
    )?;
    fs::copy(
        project.join("bundle/Info.plist"),
        payload.join("Contents/Info.plist"),
    )?;
    let output = output.unwrap_or_else(|| project.join(format!("{name}.pkg")));
    super::build::run_status(
        Command::new("pkgbuild")
            .arg("--root")
            .arg(project.join("target/pkg-root"))
            .arg("--identifier")
            .arg(identifier)
            .arg("--version")
            .arg("0.1.0")
            .arg(output),
    )
}
