use std::{error::Error, fs, path::Path};

#[cfg(unix)]
pub(super) fn set_executable(path: &Path) -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}
#[cfg(not(unix))]
pub(super) fn set_executable(_: &Path) -> Result<(), Box<dyn Error>> {
    Ok(())
}
