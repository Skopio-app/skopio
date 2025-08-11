use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    db_paths: HashMap<String, String>,
}

fn get_config_path() -> String {
    let config_dir = dirs::config_dir().expect("Failed to get home directory");
    if cfg!(debug_assertions) {
        format!(
            "{}/com.samwahome.skopio/cli_test_config.json",
            config_dir.display()
        )
    } else {
        format!(
            "{}/com.samwahome.skopio/cli_config.json",
            config_dir.display()
        )
    }
}

pub fn get_or_store_db_path(cli_db_path: Option<String>, app: &str) -> String {
    let config_path = get_config_path();
    let config_dir = Path::new(&config_path);
    resolve_db_path(cli_db_path, app, config_dir)
}

fn resolve_db_path(cli_db_path: Option<String>, app: &str, config_path: &Path) -> String {
    let config_dir = config_path.parent().unwrap();

    fs::create_dir_all(config_dir).expect("Failed to create config directory");

    let mut config = if config_path.exists() {
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

    if cfg!(debug_assertions) {
        format!("skopio-{}-test-data.db", app)
    } else {
        format!("skopio-{}-data.db", app)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn returns_default_path_when_no_config_exists() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("cli_test_config_json");

        let result = resolve_db_path(None, "vscode", &config_path);
        assert_eq!(result, "skopio-vscode-data.db");
    }

    #[test]
    fn stores_and_returns_cli_path() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("cli_test_config.json");

        let db_path = "/tmp/db.sqlite".to_string();
        let result = resolve_db_path(Some(db_path.clone()), "android studio", &config_path);
        assert_eq!(result, db_path);

        let reread = resolve_db_path(None, "android studio", &config_path);
        assert_eq!(reread, db_path);
    }

    #[test]
    fn returns_existing_path_from_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("cli_test_config.json");

        let mut config = Config::default();
        config
            .db_paths
            .insert("webstorm".into(), "/data/webstorm.db".into());

        fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        let result = resolve_db_path(None, "webstorm", &config_path);
        assert_eq!(result, "/data/webstorm.db");
    }
}
