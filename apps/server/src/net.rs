#![cfg(target_os = "macos")]

use anyhow::Result;
use axum::Router;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::{net::UnixListener, task::JoinSet};
use tracing::{debug, error};
use std::{fs, os::unix::{fs::PermissionsExt, net::UnixListener as StdUnixListener}, path::{Path, PathBuf}};

pub fn mac_run_dir() -> Result<PathBuf> {
    let data = dirs::data_dir()?;
    Ok(data.join("com.samwahome.skopio/run"))
}

pub fn ensure_dir_mode(dir: &Path, mode: u32) -> Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    fs::set_permissions(dir, fs::Permissions::from_mode(mode))?;
    Ok(())
}

pub fn bind_uds(path: &Path, mode: u32) -> Result<UnixListener> {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    let std = StdUnixListener::bind(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    std.set_nonblocking(true)?;
    Ok(UnixListener::from_std(std)?)
}

pub async fn serve_unix(
    listener: UnixListener,
    app: Router,
    shutdown: impl std::future::Future<Output = ()>,
) -> Result<()> {
    let mut joinset: JoinSet<()> = JoinSet::new();

    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            biased;

            _ = &mut shutdown => {
                debug!("UDS: shutdown requested");
                break;
            }

            accept = listener.accept() => {
                let (stream, _addr) = match accept {
                    Ok(s) => s,
                    Err(e) => {
                        error!("UDS accept error: {e}");
                        continue;
                    }
                };

                let svc = app.clone().into_service();

                joinset.spawn(async move {
                    let io = TokioIo::new(stream);
                    if let Err(err) = http1::Builder::new()
                        .preserve_header_case(false)
                        .title_case_headers(false)
                        .serve_connection(io, svc)
                        .await
                    {
                        debug!("UDS conn error: {err}");
                    }
                });
            }
        }
    }

    while let Some(_res) = joinset.join_next().await {}
    Ok(())
}
