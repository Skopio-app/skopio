use std::{panic, process::Command};

use log::{error, warn};

use crate::monitored_app::MonitoredApp;

pub fn get_terminal_process() -> String {
    let script = r#"
    tell application "Terminal"
        if (count of windows) > 0 then
            set frontWindow to front window
            return name of frontWindow
        else
            return "No Active Terminal"
        end if
    end tell
    "#;

    let window_title = run_osascript(script);

    if !window_title.is_empty() && window_title != "No Active Terminal" {
        window_title
    } else {
        "unknown".to_string()
    }
}

pub fn get_browser_active_tab(bundle_id: &MonitoredApp) -> String {
    let script = match bundle_id {
        MonitoredApp::Chrome => {
            r#"
            tell application "Google Chrome"
                if (count of windows) > 0 and (count of tabs of front window) > 0 then
                    return URL of active tab of front window
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
                return clipboard_content
            end tell
        "#
        }
        MonitoredApp::Safari => {
            r#"
            tell application "Safari"
                if (count of windows) > 0 and (count of tabs of front window) > 0 then
                    return URL of current tab of front window
                else
                    return "No active tab"
                end if
            end tell
        "#
        }
        _ => return "unknown".to_string(),
    };

    let output = run_osascript(script);
    if output == "No active tab" || output.is_empty() {
        warn!("No active tab detected for {}", bundle_id);
        return "unknown".to_string();
    }

    output
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
