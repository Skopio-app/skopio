use dirs::data_dir;
use std::path::PathBuf;

fn get_db_name() -> String {
    if cfg!(debug_assertions) {
        return String::from("skopio_server_test.db");
    } else {
        return String::from("skopio_server.db");
    }
}

pub fn get_db_path() -> PathBuf {
    let base_dir = data_dir().unwrap_or_else(|| PathBuf::from("."));
    base_dir
        .join("com.samwahome.skopio")
        .join("server")
        .join(get_db_name())
}
