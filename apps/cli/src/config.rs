use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::utils::CliError;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    db_paths: HashMap<String, String>,
}

fn get_config_path() -> Result<PathBuf, CliError> {
    let config_dir = dirs::config_dir().expect("Failed to get config directory");
    let cli_config_dir = config_dir.join("com.samwahome.skopio");

    fs::create_dir_all(&cli_config_dir)?;

    if cfg!(debug_assertions) {
        Ok(cli_config_dir.join("cli_test_config.json"))
    } else {
        Ok(cli_config_dir.join("cli_config.json"))
    }
}

fn db_filename(app: &str) -> String {
    if cfg!(debug_assertions) {
        format!("skopio-{app}-test-data.db")
    } else {
        format!("skopio-{app}-data.db")
    }
}

pub fn get_or_store_db_path(db_dir: Option<String>, app: &str) -> Result<String, CliError> {
    let config_path = get_config_path()?;
    let mut config = fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| serde_json::from_str::<Config>(&s).ok())
        .unwrap_or_default();

    if let Some(dir) = db_dir {
        let full_db_path = resolve_db_path(&dir, app);
        config
            .db_paths
            .insert(app.to_string(), full_db_path.to_str().unwrap().to_string());
        write_config(&config_path, &config)?;
        return Ok(full_db_path.to_str().unwrap().to_string());
    }

    if let Some(db_path) = config.db_paths.get(app) {
        return Ok(db_path.clone());
    }

    let default_config_dir = config_path
        .parent()
        .expect("Config path has no parent")
        .to_path_buf();
    fs::create_dir_all(&default_config_dir)?;
    let default_db_path = default_config_dir.join(db_filename(app));

    config.db_paths.insert(
        app.to_string(),
        default_db_path.to_str().unwrap().to_string(),
    );
    write_config(&config_path, &config)?;
    Ok(default_db_path.to_str().unwrap().to_string())
}

fn resolve_db_path(dir: &str, app: &str) -> PathBuf {
    let db_path = Path::new(dir);

    if db_path.is_dir() {
        db_path.join(db_filename(app))
    } else {
        db_path.to_path_buf()
    }
}

fn write_config(config_path: &Path, config: &Config) -> Result<(), CliError> {
    let mut file = File::create(config_path)?;
    let serialized = serde_json::to_string_pretty(config)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn stores_and_returns_cli_path() {
        let dir = tempdir().unwrap();
        let app = "android studio";

        let result =
            get_or_store_db_path(Some(dir.path().to_str().unwrap().to_string()), app).unwrap();
        let expected = dir
            .path()
            .join(db_filename(app))
            .to_str()
            .unwrap()
            .to_string();
        assert_eq!(result, expected);

        let reread = get_or_store_db_path(None, app).unwrap();
        assert_eq!(reread, expected);
    }
}
