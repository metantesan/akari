use std::{env, error::Error, fs, path::PathBuf, process::Command};

pub(super) fn project_dir(project: Option<PathBuf>) -> Result<PathBuf, Box<dyn Error>> {
    let project = match project {
        Some(project) => project,
        None => env::current_dir()?,
    };
    if project.is_absolute() {
        Ok(project)
    } else {
        Ok(env::current_dir()?.join(project))
    }
}

pub(super) fn run_project_command(
    project: Option<PathBuf>,
    target: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let project = project_dir(project)?;
    let mut cmd = if target.as_deref() == Some("android") {
        let mut command = Command::new("./gradlew");
        command
            .current_dir(project.join("android"))
            .arg(":app:assembleDebug");
        command
    } else {
        let mut command = Command::new("cargo");
        command.current_dir(&project).args(["build", "--release"]);
        if let Some(target) = target {
            command.arg("--target").arg(target);
        }
        command
    };
    run_status(&mut cmd)
}

pub(super) fn run_project(
    project: Option<PathBuf>,
    target: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let project = project_dir(project)?;
    if target.as_deref() == Some("android") {
        let mut gradle = Command::new("./gradlew");
        gradle
            .current_dir(project.join("android"))
            .arg(":app:installDebug");
        run_status(&mut gradle)?;

        let manifest = fs::read_to_string(project.join("Cargo.toml"))?;
        let application_id = manifest
            .lines()
            .find_map(|line| {
                line.trim()
                    .strip_prefix("application-id = \"")
                    .and_then(|value| value.strip_suffix('\"'))
                    .map(str::to_owned)
            })
            .ok_or("cannot read [package.metadata.akari] application-id")?;
        let activity = format!("{application_id}/.MainActivity");
        let mut adb = Command::new("adb");
        adb.args(["shell", "am", "start", "-n"]).arg(activity);
        run_status(&mut adb)
    } else {
        let mut command = Command::new("cargo");
        command.current_dir(&project).arg("run");
        if let Some(target) = target {
            command.arg("--target").arg(target);
        }
        run_status(&mut command)
    }
}

pub(super) fn run_status(command: &mut Command) -> Result<(), Box<dyn Error>> {
    let status = command.status()?;
    if !status.success() {
        return Err(format!("command failed with status {status}: {command:?}").into());
    }
    Ok(())
}
