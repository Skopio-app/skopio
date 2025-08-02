use std::path::Path;

use git2::Repository;

pub fn find_git_branch<T: AsRef<Path>>(from_path: T) -> Option<String> {
    if let Ok(repo) = Repository::discover(from_path) {
        if let Ok(head) = repo.head() {
            if let Some(branch) = head.shorthand() {
                return Some(branch.to_string());
            }
        }
    }

    None
}
