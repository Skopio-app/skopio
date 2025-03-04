use git2::Repository;
use std::path::Path;

/// Extracts the project name from the project path
pub fn extract_project_name(project_path: &Path) -> String {
    project_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "UnnamedProject".to_string())
}

pub fn find_git_branch(from_path: &Path) -> String {
    if let Ok(repo) = Repository::discover(from_path) {
        if let Ok(head) = repo.head() {
            if let Some(branch) = head.shorthand() {
                return branch.to_string();
            }
        }
    }

    "unknown".to_string()
}
