use std::path::Path;

pub fn extract_db_file_path(database_url: &str) -> std::path::PathBuf {
    let db_path = database_url.trim_start_matches("sqlite://");
    Path::new(db_path).to_path_buf()
}