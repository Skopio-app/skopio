use std::path::Path;

use tokei::{Config, Languages};

pub fn detect_language(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);

    let mut languages = Languages::new();
    let config = Config::default();

    languages.get_statistics(&[path], &[], &config);

    let detected_language = languages
        .iter()
        .max_by_key(|(_, stats)| stats.code)
        .map(|(lang, _)| *lang);

    detected_language.map(|lang| lang.name().to_string())
}
