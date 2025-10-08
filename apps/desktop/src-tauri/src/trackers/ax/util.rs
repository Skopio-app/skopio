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
    let markers = [
        "xcworkspace",     // Xcode workspace
        "xcodeproj",       // Xcode project
        "Package.swift",   // Swift Package
        "project.pbxproj", // Legacy/bare project file
        "playground",      // Xcode Playground
        "xcplayground",    // Xcode Playground (alternative extension)
    ];

    let mut cur = Path::new(entity).parent()?;

    loop {
        let found = if let Ok(read) = std::fs::read_dir(cur) {
            let mut hit: Option<PathBuf> = None;

            for entry in read.flatten() {
                let path = entry.path();

                let file_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();

                if markers.iter().any(|m| *m == file_name || *m == ext) {
                    let root = match file_name {
                        "Package.swift" | "project.pbxproj" => cur.to_path_buf(),
                        _ => path,
                    };
                    hit = Some(root);
                    break;
                }
            }
            hit
        } else {
            None
        };

        if let Some(root) = found {
            return Some(root);
        }

        cur = cur.parent()?;
    }
}

pub fn derive_xcode_project_name<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .file_stem()
        .or_else(|| path.as_ref().file_name())
        .map(|name| name.to_string_lossy().to_string())
}
