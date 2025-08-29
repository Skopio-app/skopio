use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime};

pub fn get_db_name() -> String {
    if cfg!(debug_assertions) {
        String::from("skopio_desktop_test.db")
    } else {
        String::from("skopio_desktop.db")
    }
}

pub fn get_db_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(get_db_name())
}
