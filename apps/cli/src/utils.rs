use std::path::Path;

/// Extracts the project name from the project path
pub fn extract_project_name(project_path: &Path) -> String {
    project_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "UnnamedProject".to_string())
}
