use std::{panic, process::Command};

use tracing::error;

pub fn run_osascript(script: &str) -> String {
    let result = panic::catch_unwind(|| {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .ok()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

        output.unwrap_or_else(|| "unknown".to_string())
    });

    result.unwrap_or_else(|_| {
        error!("Failed to execute AppleScript: {}", script);
        "unknown".to_string()
    })
}
