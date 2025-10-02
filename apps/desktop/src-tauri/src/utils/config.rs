use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::{watch, RwLock};

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct TrackedApp {
    pub name: String,
    pub bundle_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub theme: Theme,
    pub afk_timeout: u64,
    pub flush_interval: u64,
    pub sync_interval: u64,
    pub global_shortcut: String,
    pub tracked_apps: Vec<TrackedApp>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            theme: Theme::System,
            afk_timeout: 120,
            flush_interval: 120,
            sync_interval: 180,
            global_shortcut: String::from("CommandOrControl+S"),
            tracked_apps: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[serde(alias = "Light")]
    Light,
    #[serde(alias = "Dark")]
    Dark,
    #[serde(alias = "System")]
    System,
}

fn get_config_name() -> String {
    if cfg!(debug_assertions) {
        String::from("config_test.json")
    } else {
        String::from("config.json")
    }
}

#[derive(Clone)]
pub struct ConfigStore {
    inner: Arc<RwLock<AppConfig>>,
    pub afk_timeout: watch::Sender<u64>,
    pub flush_interval: watch::Sender<u64>,
    pub sync_interval: watch::Sender<u64>,
    pub tracked_apps: watch::Sender<Vec<TrackedApp>>,
}

impl ConfigStore {
    pub async fn new<R: Runtime>(handle: &AppHandle<R>) -> io::Result<Self> {
        let path = Self::get_config_path(handle);
        let config = if path.exists() {
            let contents = fs::read_to_string(path)?;
            serde_json::from_str(&contents)?
        } else {
            let default = AppConfig::default();
            default.save(handle)?;
            default
        };

        let (afk_tx, _) = watch::channel(config.afk_timeout);
        let (flush_tx, _) = watch::channel(config.flush_interval);
        let (sync_tx, _) = watch::channel(config.sync_interval);
        let (tracked_tx, _) = watch::channel(config.tracked_apps.clone());

        Ok(Self {
            inner: Arc::new(RwLock::new(config)),
            afk_timeout: afk_tx,
            flush_interval: flush_tx,
            sync_interval: sync_tx,
            tracked_apps: tracked_tx,
        })
    }

    pub fn get_config_path<R: Runtime>(handle: &AppHandle<R>) -> PathBuf {
        handle
            .path()
            .app_config_dir()
            .unwrap_or_else(|_| std::env::temp_dir())
            .join(get_config_name())
    }

    pub async fn get(&self) -> AppConfig {
        self.inner.read().await.clone()
    }

    pub async fn update<R: Runtime, F>(&self, handle: &AppHandle<R>, updater: F) -> io::Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut guard = self.inner.write().await;
        updater(&mut guard);
        guard.save(handle)?;

        let _ = self.afk_timeout.send(guard.afk_timeout);
        let _ = self.flush_interval.send(guard.flush_interval);
        let _ = self.sync_interval.send(guard.sync_interval);
        let _ = self.tracked_apps.send(guard.tracked_apps.clone());

        Ok(())
    }

    pub fn subscribe_afk_timeout(&self) -> watch::Receiver<u64> {
        self.afk_timeout.subscribe()
    }

    pub fn subscribe_flush_interval(&self) -> watch::Receiver<u64> {
        self.flush_interval.subscribe()
    }

    pub fn subscribe_sync_interval(&self) -> watch::Receiver<u64> {
        self.sync_interval.subscribe()
    }

    pub fn subscribe_tracked_apps(&self) -> watch::Receiver<Vec<TrackedApp>> {
        self.tracked_apps.subscribe()
    }
}

impl AppConfig {
    pub fn save<R: Runtime>(&self, handle: &AppHandle<R>) -> io::Result<()> {
        let path = ConfigStore::get_config_path(handle);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(&self)?;
        let mut file = fs::File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_config<R: Runtime>(app: AppHandle<R>) -> AppConfig {
    let config_store = app.state::<ConfigStore>();
    config_store.get().await
}

#[tauri::command]
#[specta::specta]
pub async fn set_theme<R: Runtime>(theme: Theme, app: AppHandle<R>) -> Result<(), String> {
    let config_store = app.state::<ConfigStore>();
    config_store
        .update(&app, |config| {
            config.theme = theme;
        })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn set_afk_timeout<R: Runtime>(timeout: u64, app: AppHandle<R>) -> Result<(), String> {
    let config_store = app.state::<ConfigStore>();
    config_store
        .update(&app, |config| {
            config.afk_timeout = timeout;
        })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn set_global_shortcut<R: Runtime>(
    shortcut: String,
    app: AppHandle<R>,
) -> Result<(), String> {
    let config_store = app.state::<ConfigStore>();
    config_store
        .update(&app, |config| {
            config.global_shortcut = shortcut;
        })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn set_tracked_apps<R: Runtime>(
    apps: Vec<TrackedApp>,
    app: AppHandle<R>,
) -> Result<(), String> {
    let config_store = app.state::<ConfigStore>();
    config_store
        .update(&app, |config| {
            config.tracked_apps = apps;
        })
        .await
        .map_err(|e| e.to_string())
}
