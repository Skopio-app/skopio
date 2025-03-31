use std::process::Command;

pub fn get_git_branch(project: &str) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(project)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    output.unwrap_or_else(|| "unknown".to_string())
}
