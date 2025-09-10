use anyhow::{Context, Result};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    os::unix::net::UnixListener as StdUnixListener,
    path::{Path, PathBuf},
};
use tokio::net::UnixListener;

#[allow(dead_code)]
pub fn mac_run_dir() -> Result<PathBuf> {
    let data = dirs::data_dir().context("mac data dir not found")?;
    Ok(data.join("com.samwahome.skopio/run"))
}

#[allow(dead_code)]
pub fn ensure_dir_mode(dir: &Path, mode: u32) -> Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    fs::set_permissions(dir, fs::Permissions::from_mode(mode))?;
    Ok(())
}

#[allow(dead_code)]
pub fn bind_uds(path: &Path, mode: u32) -> Result<UnixListener> {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    let std = StdUnixListener::bind(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    std.set_nonblocking(true)?;
    Ok(UnixListener::from_std(std)?)
}
