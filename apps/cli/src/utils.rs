use std::path::{Path, PathBuf};
use git2::Repository;

/// **Finds the Git project root from a given path, falling back to the path itself**
pub fn find_git_project_root(from_path: &Path) -> PathBuf {
    if let Ok(repo) = Repository::discover(from_path) {
        if let Some(workdir) = repo.workdir() {
            return workdir.to_path_buf();
        }
    }

    from_path.to_path_buf()
}

/// **Extracts the project name from the project path**
pub fn extract_project_name(project_path: &Path) -> String {
    project_path.file_name()
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