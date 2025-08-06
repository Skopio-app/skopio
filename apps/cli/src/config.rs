use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    db_paths: HashMap<String, String>,
}

fn get_config_path() -> String {
    let config_dir = dirs::config_dir().expect("Failed to get home directory");
    format!(
        "{}/com.samwahome.skopio/cli_config.json",
        config_dir.display()
    )
}

pub fn get_or_store_db_path(cli_db_path: Option<String>, app: &str) -> String {
    let config_path = get_config_path();
    let config_dir = Path::new(&config_path).parent().unwrap();

    fs::create_dir_all(config_dir).expect("Failed to create config directory");

    let mut config = if Path::new(&config_path).exists() {
        fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Config>(&s).ok())
            .unwrap_or_default()
    } else {
        Config::default()
    };

    if let Some(db) = cli_db_path {
        config.db_paths.insert(app.to_string(), db.clone());
        fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
            .expect("Failed to write config");
    }

    if let Some(db) = config.db_paths.get(app) {
        return db.clone();
    }

    format!("skopio-{}-data.db", app)
}
