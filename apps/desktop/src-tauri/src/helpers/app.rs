use std::{panic, process::Command};

use log::{error, warn};
use url::Url;

use crate::{helpers::language::detect_language, monitored_app::MonitoredApp};

/// Returns (domain, url, tab title) from the active tab of a supported browser
pub fn get_browser_active_tab(bundle_id: &MonitoredApp) -> (String, String, String) {
    let script = match bundle_id {
        MonitoredApp::Chrome => {
            r#"
            tell application "Google Chrome"
                if (count of windows) > 0 and (count of tabs of front window) > 0 then
                    set theTab to active tab of front window
                    set theURL to URL of the theTab
                    set theTitle to title of theTab
                    return theURL & "||" & theTitle
                else
                    return "No active tab"
                end if
            end tell
        "#
        }
        MonitoredApp::Firefox => {
            r#"
            tell application "Firefox"
                activate
                delay 0.5
                tell application "System Events"
                    keystroke "l" using command down
                    delay 0.2
                    keystroke "c" using command down
                end tell
                delay 0.2
                set clipboard_content to the clipboard
                return clipboard_content & "||unknown"
            end tell
        "#
        }
        MonitoredApp::Safari => {
            r#"
            tell application "Safari"
                if (count of windows) > 0 and (count of tabs of front window) > 0 then
                    set theTab to current tab of front window
                    set theURL to URL of theTab
                    set theTitle to name of theTab
                    return theURL & "||" & theTitle
                else
                    return "No active tab"
                end if
            end tell
        "#
        }
        _ => {
            return (
                "unknown".to_string(),
                "unknown".to_string(),
                "unknown".to_string(),
            )
        }
    };

    let output = run_osascript(script);
    if output == "No active tab" || output.is_empty() {
        warn!("No active tab detected for {}", bundle_id);
        return (
            "unknown".to_string(),
            "unknown".to_string(),
            "unknown".to_string(),
        );
    }

    let parts: Vec<&str> = output.split("||").collect();
    let url = parts.first().unwrap_or(&"unknown").trim();
    let title = parts.get(1).unwrap_or(&"unknown").trim();

    let domain = Url::parse(url)
        .ok()
        .and_then(|u| u.domain().map(|d| d.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    (domain, url.to_string(), title.to_string())
}

pub fn get_xcode_project_details() -> (Option<String>, Option<String>, String, Option<String>) {
    let project_path_script =
        r#"tell application "Xcode" to get path of active workspace document"#;
    let project_path = run_osascript(project_path_script);
    let project_path = if project_path != "unknown" && !project_path.is_empty() {
        Some(project_path)
    } else {
        None
    };
    let project_name = project_path
        .as_ref()
        .map(|p| p.split("/").last().unwrap_or("Unknown").to_string());

    let active_file_script = r#"
        tell application "System Events"
            tell process "Xcode"
                try
                    return value of attribute "AXDocument" of window 1
                on error
                    return "No active document"
                end try
            end tell
        end tell
    "#;
    let active_file_path = run_osascript(active_file_script);

    let entity_name = if active_file_path.starts_with("file://") {
        active_file_path.trim_start_matches("file://").to_string()
    } else if active_file_path != "No active document" && active_file_path != "unknown" {
        active_file_path
    } else {
        warn!("No active document detected in Xcode.");
        "unknown".to_string()
    };

    let language_name = detect_language(&entity_name);

    (project_name, project_path, entity_name, language_name)
}

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
