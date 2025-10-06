use std::path::{Path, PathBuf};

use percent_encoding::percent_decode_str;

pub fn normalize_file(input: &str) -> Option<String> {
    if input.is_empty() {
        return None;
    }
    if let Some(rest) = input.strip_prefix("file://") {
        let decoded = percent_decode_str(rest).decode_utf8().ok()?.to_string();
        return Some(if decoded.starts_with('/') {
            decoded
        } else {
            format!("/{}", decoded)
        });
    }
    Some(input.to_string())
}

pub fn infer_xcode_root(entity: &str) -> Option<PathBuf> {
    let mut cur = Path::new(entity).parent()?;
    while let Some(dir) = Some(cur) {
        if let Ok(read) = std::fs::read_dir(dir) {
            for entry in read.flatten() {
                let p = entry.path();
                match p.extension().and_then(|e| e.to_str()) {
                    Some("xcworkspace") | Some("xcodeproj") => return Some(p),
                    _ => {}
                }
            }
        }
        cur = dir.parent()?;
    }
    None
}

pub fn derive_xcode_project_name<P: AsRef<Path>>(path: P) -> Option<String> {
    if let Some(stem) = path.as_ref().file_stem() {
        return Some(stem.to_string_lossy().to_string());
    }
    path.as_ref()
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
}
