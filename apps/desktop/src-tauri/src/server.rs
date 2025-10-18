use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use std::{io, path::PathBuf};

use async_trait::async_trait;
use common::models::outputs::HealthStatus;
use futures_util::StreamExt;
use reqwest::Client;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;
use thiserror::Error;
use tokio::time::timeout;
use tokio::{fs as tokiofs, io::AsyncWriteExt};
use tracing::{debug, error};
use zip::result::ZipError;
use zip::ZipArchive;

use crate::network;

#[derive(Error, Debug)]
pub enum ServerManagerError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Bad response status: {0}")]
    BadStatus(String),

    #[error("Digest mismatch")]
    DigestMismatch,

    #[error("Launchctl failed: {0}")]
    Launchctl(String),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("Manifest error: {0}")]
    Manifest(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Zip error: {0}")]
    Zip(#[from] ZipError),

    #[error("Semver error: {0}")]
    Semver(#[from] semver::Error),
}

#[derive(Debug, Clone, Serialize, Type, Event)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum ServerStatus {
    Offline,
    Checking,
    Downloading {
        received: u64,
        total: Option<u64>,
        percent: Option<u8>,
    },
    Installing,
    Starting,
    Running,
    Updating,
    Error {
        message: String,
    },
}

const PLIST_LABEL: &str = "com.samwahome.skopio.server";
const BIN_NAME: &str = "skopio-server";
const MANIFEST_URL: &str =
    "https://github.com/Skopio-app/server-releases/releases/latest/download/latest.json";

#[derive(Debug, Deserialize)]
struct AssetSig {
    #[allow(dead_code)]
    #[serde(default)]
    r#type: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    signature_url: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    certificate_url: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    bundle_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    url: String,
    sha256: String,
    #[allow(dead_code)]
    #[serde(default)]
    size: Option<u64>,
    #[allow(dead_code)]
    #[serde(default)]
    sig: Option<AssetSig>,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    version: String,
    #[allow(dead_code)]
    #[serde(default)]
    released_at: Option<String>,
    assets: ManifestAssets,
}

#[derive(Debug, Deserialize)]
struct ManifestAssets {
    #[serde(rename = "darwin-aarch64")]
    #[allow(dead_code)]
    darwin_aarch64: Option<Asset>,
    #[serde(rename = "darwin-x86_64")]
    #[allow(dead_code)]
    darwin_x86_64: Option<Asset>,
}

#[async_trait]
pub trait ServerManagerExt {
    fn start_server(&self) -> Result<(), ServerManagerError>;
    fn set_server_status(&self, status: ServerStatus);
    async fn check_server_update(&self) -> Result<bool, ServerManagerError>;
    async fn ensure_server_ready(&self) -> Result<(), ServerManagerError>;
}

#[async_trait]
impl<R: Runtime> ServerManagerExt for AppHandle<R> {
    fn start_server(&self) -> Result<(), ServerManagerError> {
        let bin = server_bin_path(self)?;
        if !bin.exists() {
            return Err(ServerManagerError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "server binary missing",
            )));
        }
        let plist = write_plist(self, &bin)?;
        launchctl(&["bootstrap", &gui_domain()?, plist.to_str().unwrap()])?;
        let label = PLIST_LABEL;
        launchctl(&["kickstart", "-k", &format!("{}/{}", gui_domain()?, label)])?;
        Ok(())
    }

    fn set_server_status(&self, status: ServerStatus) {
        status.emit(self).unwrap_or_default();
    }

    async fn check_server_update(&self) -> Result<bool, ServerManagerError> {
        self.set_server_status(ServerStatus::Checking);

        let client = Client::new();
        let manifest = fetch_manifest(&client).await?;
        let asset = pick_asset(&manifest)?;

        let latest = Version::parse(&manifest.version)?;
        let current_str = read_installed_version(self).unwrap_or_default();
        let current = Version::parse(&current_str)?;

        let needs_update = latest > current;

        if needs_update {
            self.set_server_status(ServerStatus::Updating);
            let _ = status().and_then(|_| stop());
            self.set_server_status(ServerStatus::Installing);
            let _ = download_and_install(self, &manifest, asset).await?;
            self.set_server_status(ServerStatus::Starting);
            self.start_server()?;
            self.set_server_status(ServerStatus::Running);
            return Ok(true);
        }

        if read_installed_version(self).as_deref() != Some(&manifest.version) {
            let _ = write_installed_version(self, &manifest.version);
        }

        Ok(false)
    }

    async fn ensure_server_ready(&self) -> Result<(), ServerManagerError> {
        self.set_server_status(ServerStatus::Checking);

        let is_running = status().is_ok();

        match self.check_server_update().await {
            Ok(true) => {
                self.set_server_status(ServerStatus::Starting);
                check_server_ready(MAX_WAIT).await?;
                self.set_server_status(ServerStatus::Running);
                Ok(())
            }
            Ok(false) => {
                self.set_server_status(ServerStatus::Starting);
                if !is_running {
                    self.start_server()?;
                }
                check_server_ready(MAX_WAIT).await?;
                self.set_server_status(ServerStatus::Running);
                Ok(())
            }
            Err(e) => {
                self.set_server_status(ServerStatus::Error {
                    message: e.to_string(),
                });
                Err(e)
            }
        }
    }
}

const MAX_WAIT: Duration = Duration::from_secs(120);

fn server_root<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, ServerManagerError> {
    let base = app.path().app_data_dir()?;
    Ok(base.join("server"))
}

fn server_bin_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, ServerManagerError> {
    Ok(server_root(app)?.join("bin").join(BIN_NAME))
}

#[cfg(target_os = "macos")]
fn plist_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, ServerManagerError> {
    let home = app.path().home_dir()?;
    Ok(home
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{PLIST_LABEL}.plist")))
}

async fn download_to_temp<R: Runtime>(
    app: &AppHandle<R>,
    client: &Client,
    url: &str,
) -> Result<PathBuf, ServerManagerError> {
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(ServerManagerError::BadStatus(resp.status().to_string()));
    }
    let total = resp.content_length();

    let tmp_dir = server_root(app)?.join("tmp");
    tokiofs::create_dir_all(&tmp_dir).await?;
    let tmp_file = tmp_dir.join("download.bin.part");

    let mut file = tokiofs::File::create(&tmp_file).await?;
    let mut stream = resp.bytes_stream();
    let mut received: u64 = 0;
    let mut last_emit = 0u8;

    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        received += bytes.len() as u64;
        file.write_all(&bytes).await?;

        let percent = total.map(|t| ((received.saturating_mul(100)) / t) as u8);
        if percent.unwrap_or(0) != last_emit {
            app.set_server_status(ServerStatus::Downloading {
                received,
                total,
                percent,
            });
            last_emit = percent.unwrap_or(0)
        }
    }
    file.flush().await?;
    Ok(tmp_file)
}

fn compute_sha256(path: &Path) -> Result<String, ServerManagerError> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for b in digest {
        use std::fmt::Write as _;
        write!(&mut hex, "{:02x}", b).unwrap();
    }
    Ok(hex)
}

fn unzip_binary(zip_path: &Path, out_path: &Path) -> Result<(), ServerManagerError> {
    let zip_file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(zip_file)?;

    let mut pick_idx: Option<usize> = None;
    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        if entry.is_file() {
            let name = entry.name();
            if name.ends_with(BIN_NAME) || name == BIN_NAME {
                pick_idx = Some(i);
                break;
            }
            if pick_idx.is_none()
                && Path::new(name).file_name() == Some(std::ffi::OsStr::new(BIN_NAME))
            {
                pick_idx = Some(i);
            }
        }
    }
    if pick_idx.is_none() && archive.len() == 1 && archive.by_index(0)?.is_file() {
        pick_idx = Some(0);
    }

    let idx = pick_idx.ok_or_else(|| {
        ServerManagerError::Manifest("Zip does not contain expected binary".into())
    })?;
    let mut entry = archive.by_index(idx)?;
    if let Some(dir) = out_path.parent() {
        fs::create_dir_all(dir)?;
    }
    let part = out_path.with_extension("part");
    {
        let mut out = fs::File::create(&part)?;
        std::io::copy(&mut entry, &mut out)?;
        out.flush()?;
    }
    let mut perms = fs::metadata(&part)?.permissions();
    #[cfg(unix)]
    {
        perms.set_mode(0o755);
        fs::set_permissions(&part, perms)?;
    }
    if out_path.exists() {
        fs::remove_file(out_path)?;
    }
    fs::rename(part, out_path)?;
    Ok(())
}

fn write_plist<R: Runtime>(app: &AppHandle<R>, bin: &Path) -> Result<PathBuf, ServerManagerError> {
    let plist_dst = plist_path(app)?;
    if let Some(dir) = plist_dst.parent() {
        fs::create_dir_all(dir)?;
    }

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
            <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
                "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
                <plist version="1.0"><dict>
                <key>Label</key><string>{label}</string>
                <key>ProgramArguments</key>
                <array>
                <string>{bin}</string>
                </array>
                <key>RunAtLoad</key><true/>
                <key>KeepAlive</key><true/>
                <key>ProcessType</key><string>Background</string>
                </dict></plist>
            "#,
        label = PLIST_LABEL,
        bin = bin.display(),
    );

    let mut f = fs::File::create(&plist_dst)?;
    f.write_all(plist.as_bytes())?;
    f.flush()?;
    Ok(plist_dst)
}

#[cfg(target_os = "macos")]
fn launchctl(args: &[&str]) -> Result<(), ServerManagerError> {
    let out = Command::new("launchctl").args(args).output()?;
    if !out.status.success() {
        return Err(ServerManagerError::Launchctl(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    Ok(())
}

fn gui_domain() -> Result<String, ServerManagerError> {
    let uid = current_uid();
    Ok(format!("gui/{}", uid))
}

#[cfg(unix)]
fn current_uid() -> u32 {
    unsafe { libc::getuid() }
}

async fn fetch_manifest(client: &Client) -> Result<Manifest, ServerManagerError> {
    let url = MANIFEST_URL;
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(ServerManagerError::BadStatus(resp.status().to_string()));
    }
    let text = resp.text().await?;
    let m: Manifest = serde_json::from_str(&text)?;
    Ok(m)
}

fn pick_asset(m: &Manifest) -> Result<&Asset, ServerManagerError> {
    match std::env::consts::ARCH {
        "aarch64" => m.assets.darwin_aarch64.as_ref().ok_or_else(|| {
            ServerManagerError::Manifest("missing darwin-aarch64 in manifest".into())
        }),
        "x86_64" => m.assets.darwin_x86_64.as_ref().ok_or_else(|| {
            ServerManagerError::Manifest("missing darwin-x86_64 in manifest".into())
        }),
        other => Err(ServerManagerError::Manifest(format!(
            "Unsupported arch: {other}"
        ))),
    }
}

async fn download_and_install<R: Runtime>(
    app: &AppHandle<R>,
    manifest: &Manifest,
    asset: &Asset,
) -> Result<PathBuf, ServerManagerError> {
    let client = Client::new();

    let zip_path = download_to_temp(app, &client, &asset.url).await?;
    let processed = compute_sha256(&zip_path)?;
    let fetched = asset.sha256.to_ascii_lowercase();
    if processed != fetched {
        return Err(ServerManagerError::DigestMismatch);
    }

    let out = server_bin_path(app)?;
    unzip_binary(&zip_path, &out)?;
    write_installed_version(app, &manifest.version)?;
    Ok(out)
}

pub fn stop() -> Result<(), ServerManagerError> {
    let label = PLIST_LABEL;
    launchctl(&["bootout", &format!("{}/{}", gui_domain()?, label)])?;
    Ok(())
}

pub fn status() -> Result<(), ServerManagerError> {
    let label = PLIST_LABEL;
    launchctl(&["print", &format!("{}/{}", gui_domain()?, label)])
}

fn version_file<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, ServerManagerError> {
    Ok(server_root(app)?.join("bin").join("version.txt"))
}

fn read_installed_version<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    fs::read_to_string(version_file(app).ok()?)
        .ok()
        .map(|s| s.trim().to_string())
}

fn write_installed_version<R: Runtime>(
    app: &AppHandle<R>,
    ver: &str,
) -> Result<(), ServerManagerError> {
    let vf = version_file(app)?;
    if let Some(dir) = vf.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(vf, ver.as_bytes())?;
    Ok(())
}

async fn check_server_ready(max_wait: Duration) -> Result<(), ServerManagerError> {
    let start = Instant::now();
    let mut delay = Duration::from_millis(100);

    loop {
        if start.elapsed() >= max_wait {
            return Err(ServerManagerError::Io(io::Error::other(
                "Server readiness timed out",
            )));
        }

        let remaining = max_wait.saturating_sub(start.elapsed());
        let per_attempt = remaining.min(Duration::from_secs(5));

        let probe = timeout(
            per_attempt,
            network::req_json::<HealthStatus, ()>("/health", None),
        )
        .await;

        match probe {
            Err(_elapsed) => {
                debug!(
                    "Health probe timed out after {:?} (remaining budget {:?})",
                    per_attempt, remaining
                );
            }
            Ok(Err(e)) => {
                error!("Health probe failed: {e}");
            }
            Ok(Ok(h)) => {
                if h.status.eq_ignore_ascii_case("ok") {
                    return Ok(());
                }
            }
        }

        tokio::time::sleep(delay.min(remaining)).await;
        delay = (delay * 2).min(Duration::from_secs(1));
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_server_status<R: Runtime>(app: AppHandle<R>) -> Result<ServerStatus, String> {
    let bin = server_bin_path(&app).map_err(|e| e.to_string())?;
    if !bin.exists() {
        return Ok(ServerStatus::Offline);
    }
    match check_server_ready(MAX_WAIT).await {
        Ok(_) => Ok(ServerStatus::Running),
        Err(e) => Err(e.to_string()),
    }
}
