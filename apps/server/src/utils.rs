use std::path::PathBuf;
use dirs::data_dir;

pub fn get_application_support_path() -> PathBuf {
    let base_dir = data_dir().unwrap_or_else(|| PathBuf::from("."));
    let db_path = base_dir.join("Timestack").join("timestack.db");

    db_path
}