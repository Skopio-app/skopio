use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime};

const DB_NAME: &str = "skopio_desktop.db";

pub fn get_db_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(DB_NAME)
}
