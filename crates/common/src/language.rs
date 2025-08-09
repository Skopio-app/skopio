use std::path::PathBuf;

use percent_encoding::percent_decode_str;
use tokei::{Config, Languages};

fn decode_path(encoded_path: &str) -> PathBuf {
    let decoded = percent_decode_str(encoded_path).decode_utf8_lossy();
    PathBuf::from(decoded.into_owned())
}

pub fn detect_language(file_path: &str) -> Option<String> {
    let decoded_path = decode_path(file_path);

    let mut languages = Languages::new();
    let config = Config::default();

    languages.get_statistics(&[decoded_path.as_path()], &[], &config);

    let detected_language = languages
        .iter()
        .max_by_key(|(_, stats)| stats.code)
        .map(|(lang, _)| lang.name().to_string());

    detected_language
}
