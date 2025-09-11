use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;
use std::{io, path::PathBuf};

use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::{fs as tokiofs, io::AsyncWriteExt};

#[derive(Error, Debug)]
pub enum ServerCtlError {
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
}

#[derive(Debug, Serialize)]
pub enum ServerReadyState {
    AlreadyRunning,
    Started,
    InstalledAndStarted,
}

const PLIST_LABEL: &str = "com.samwahome.skopio.server";
const BIN_NAME: &str = "skopio-server";
const MANIFEST_URL: &str =
    "https://github.com/Skopio-app/server-releases/releases/latest/download/latest.json";
static SERVER_INIT_GUARD: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

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
    #[allow(dead_code)]
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

fn server_root(app: &AppHandle) -> Result<PathBuf, ServerCtlError> {
    let base = app.path().app_data_dir()?;
    Ok(base.join("com.samwahome.skopio").join("server"))
}

fn server_bin_path(app: &AppHandle) -> Result<PathBuf, ServerCtlError> {
    Ok(server_root(app)?.join("bin").join(BIN_NAME))
}

fn plist_path(app: &AppHandle) -> Result<PathBuf, ServerCtlError> {
    let home = app.path().home_dir()?;
    Ok(home
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{PLIST_LABEL}.plist")))
}

async fn download_to_temp(
    app: &AppHandle,
    client: &Client,
    url: &str,
) -> Result<PathBuf, ServerCtlError> {
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(ServerCtlError::BadStatus(resp.status().to_string()));
    }
    let tmp_dir = server_root(app)?.join("tmp");
    tokiofs::create_dir_all(&tmp_dir).await?;
    let tmp_file = tmp_dir.join("download.bin.part");

    let mut file = tokiofs::File::create(&tmp_file).await?;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        file.write_all(&bytes).await?;
    }
    file.flush().await?;
    Ok(tmp_file)
}

fn sha256_file(path: &Path) -> Result<String, ServerCtlError> {
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

fn install_binary(app: &AppHandle, tmp_file: &Path) -> Result<PathBuf, ServerCtlError> {
    let bin_path = server_bin_path(app)?;
    if let Some(dir) = bin_path.parent() {
        fs::create_dir_all(dir)?;
    }
    let final_path = bin_path;
    if final_path.exists() {
        fs::remove_file(&final_path)?;
    }
    fs::rename(tmp_file, &final_path)?;
    let mut perms = fs::metadata(&final_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&final_path, perms)?;
    Ok(final_path)
}

fn write_plist(app: &AppHandle, bin: &Path) -> Result<PathBuf, ServerCtlError> {
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

fn launchctl(args: &[&str]) -> Result<(), ServerCtlError> {
    let out = Command::new("launchctl").args(args).output()?;
    if !out.status.success() {
        return Err(ServerCtlError::Launchctl(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    Ok(())
}

fn gui_domain() -> Result<String, ServerCtlError> {
    let uid = current_uid();
    Ok(format!("gui/{}", uid))
}

#[cfg(unix)]
fn current_uid() -> u32 {
    unsafe { libc::getuid() }
}

async fn fetch_manifest(client: &Client) -> Result<Manifest, ServerCtlError> {
    let url = MANIFEST_URL;
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(ServerCtlError::BadStatus(resp.status().to_string()));
    }
    let text = resp.text().await?;
    let m: Manifest = serde_json::from_str(&text)?;
    Ok(m)
}

fn pick_asset(m: &Manifest) -> Result<&Asset, ServerCtlError> {
    match std::env::consts::ARCH {
        "aarch64" => {
            m.assets.darwin_aarch64.as_ref().ok_or_else(|| {
                ServerCtlError::Manifest("missing darwin-aarch64 in manifest".into())
            })
        }
        "x86_64" => m
            .assets
            .darwin_x86_64
            .as_ref()
            .ok_or_else(|| ServerCtlError::Manifest("missing darwin-x86_64 in manifest".into())),
        other => Err(ServerCtlError::Manifest(format!(
            "Unsupported arch: {other}"
        ))),
    }
}

pub async fn download_and_install(app: &AppHandle) -> Result<PathBuf, ServerCtlError> {
    let client = Client::new();
    let manifest = fetch_manifest(&client).await?;
    let asset = pick_asset(&manifest)?;

    let tmp = download_to_temp(app, &client, &asset.url).await?;

    let processed = sha256_file(&tmp)?;
    let fetched = asset.sha256.to_ascii_lowercase();
    if processed != fetched {
        return Err(ServerCtlError::DigestMismatch);
    }

    install_binary(app, &tmp)
}

pub fn start(app: &AppHandle) -> Result<(), ServerCtlError> {
    let bin = server_bin_path(app)?;
    if !bin.exists() {
        return Err(ServerCtlError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "server binary missing",
        )));
    }
    let plist = write_plist(app, &bin)?;
    launchctl(&["bootstrap", &gui_domain()?, plist.to_str().unwrap()])?;
    let label = PLIST_LABEL;
    launchctl(&["kickstart", "-k", &format!("{}/{}", gui_domain()?, label)])?;
    Ok(())
}

// pub fn stop() -> Result<(), ServerCtlError> {
//     let label = PLIST_LABEL;
//     launchctl(&["bootout", &format!("{}/{}", gui_domain()?, label)])?;
//     Ok(())
// }

pub fn status() -> Result<(), ServerCtlError> {
    let label = PLIST_LABEL;
    launchctl(&["print", &format!("{}/{}", gui_domain()?, label)])
}

pub async fn ensure_server_ready(app: &AppHandle) -> Result<ServerReadyState, String> {
    let _guard = SERVER_INIT_GUARD.lock().await;
    if status().is_ok() {
        return Ok(ServerReadyState::AlreadyRunning);
    }

    let bin = server_bin_path(&app).map_err(err_str)?;
    if !bin.exists() {
        app.emit("server:progress", "Downloading server...").ok();
        download_and_install(&app).await.map_err(err_str)?;
        app.emit("server:progress", "Installed. Starting server...")
            .ok();
        start(&app).map_err(err_str)?;
        return Ok(ServerReadyState::InstalledAndStarted);
    }

    app.emit("server:progress", "Starting server...").ok();
    start(&app).map_err(err_str)?;
    return Ok(ServerReadyState::Started);
}

fn err_str<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}
