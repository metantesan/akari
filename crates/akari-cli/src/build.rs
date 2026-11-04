use std::{env, error::Error, path::PathBuf, process::Command};

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

pub(super) fn run_status(command: &mut Command) -> Result<(), Box<dyn Error>> {
    let status = command.status()?;
    if !status.success() {
        return Err(format!("command failed with status {status}: {command:?}").into());
    }
    Ok(())
}
