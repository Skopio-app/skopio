use moka::sync::Cache;
use percent_encoding::percent_decode_str;
use std::{path::PathBuf, sync::LazyLock, time::Duration};

use tokei::{Config, LanguageType, Languages};

static LANGUAGE_CACHE: LazyLock<Cache<String, String>> = LazyLock::new(|| {
    Cache::builder()
        .max_capacity(300)
        .time_to_live(Duration::from_secs(30 * 60)) // 30 minutes
        .time_to_idle(Duration::from_secs(10 * 60)) // 10 minutes
        .build()
});

fn decode_path(encoded_path: &str) -> PathBuf {
    let decoded = percent_decode_str(encoded_path).decode_utf8_lossy();
    PathBuf::from(decoded.into_owned())
}

pub fn detect_language(file_path: &str) -> Option<String> {
    if let Some(lang) = LANGUAGE_CACHE.get(file_path) {
        return Some(lang);
    }

    let decoded_path = decode_path(file_path);

    if let Some(ext) = decoded_path.extension().and_then(|e| e.to_str()) {
        if let Some(lang) = LanguageType::from_file_extension(ext) {
            let lang_name = lang.name().to_string();
            LANGUAGE_CACHE.insert(file_path.to_string(), lang_name.clone());
            return Some(lang_name);
        }
    }

    let mut languages = Languages::new();
    let config = Config::default();

    languages.get_statistics(&[decoded_path.as_path()], &[], &config);

    let detected_language = languages
        .iter()
        .max_by_key(|(_, stats)| stats.code)
        .map(|(lang, _)| lang.name().to_string());

    if let Some(ref name) = detected_language {
        LANGUAGE_CACHE.insert(file_path.to_string(), name.clone())
    }

    detected_language
}
