use cocoa::base::nil;
use cocoa::foundation::NSAutoreleasePool;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2::{msg_send, sel};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{panic, thread};

#[derive(Clone, PartialEq, Debug)]
pub struct Window {
    pub app_name: String,
    pub title: String,
    pub bundle_id: String,
    pub active_process: String,
}

pub struct WindowTracker {
    active_window: Arc<Mutex<Option<Window>>>,
}

impl WindowTracker {
    pub fn new() -> Self {
        Self {
            active_window: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_tracking(self: Arc<Self>, event_callback: Arc<dyn Fn(Window) + Send + Sync>) {
        let active_window = Arc::clone(&self.active_window);

        thread::spawn(move || {
            // let mut last_window: Option<Window> = None;

            loop {
                thread::sleep(Duration::from_millis(500)); // Poll every 2 seconds

                let new_window = WindowTracker::get_active_window();
                if let Some(new_window) = new_window {
                    let mut active_window_lock = active_window.lock().unwrap();

                    if *active_window_lock != Some(new_window.clone()) {
                        *active_window_lock = Some(new_window.clone());
                        event_callback(new_window); // Trigger callback on window change
                    }
                }
            }
        });
    }

    pub fn get_active_window() -> Option<Window> {
        log::info!("Entering get_active_window...");

        let result = panic::catch_unwind(|| unsafe {
            let pool = NSAutoreleasePool::new(nil);

            log::info!("Getting NSWorkspace...");
            let class_name = c"NSWorkspace";
            let workspace_class = AnyClass::get(class_name);
            if workspace_class.is_none() {
                log::error!("Failed to get NSWorkspace class.");
                return None;
            }
            let workspace: Retained<AnyObject> =
                msg_send![workspace_class.unwrap(), sharedWorkspace];

            log::info!("Getting frontmost application...");
            let front_app: Option<&AnyObject> = msg_send![&*workspace, frontmostApplication];

            if front_app.is_none() {
                log::warn!("No active application found.");
                return None;
            }

            log::info!("Getting localized app name...");
            let app_name: Option<String> = front_app.and_then(|app| {
                let app_name: Option<&AnyObject> = msg_send![app, localizedName];
                app_name.map(|app_name| {
                    let s: *const i8 = msg_send![app_name, UTF8String];
                    std::ffi::CStr::from_ptr(s).to_string_lossy().into_owned()
                })
            });

            log::info!("Getting bundle ID...");
            let bundle_id: Option<String> = {
                let bundle_id: Option<&AnyObject> =
                    front_app.map(|app| msg_send![app, bundleIdentifier]);
                if let Some(bundle_id) = bundle_id {
                    let s: *const i8 = msg_send![bundle_id, UTF8String];
                    Some(std::ffi::CStr::from_ptr(s).to_string_lossy().into_owned())
                } else {
                    None
                }
            };

            let app_name_str = app_name.unwrap_or_else(|| "unknown".to_string());
            let bundle_id_str = bundle_id.unwrap_or_else(|| "unknown".to_string());

            log::info!("App name: {}", app_name_str);
            log::info!("Bundle ID: {}", bundle_id_str);

            let mut window_title_str = Self::get_window_title();
            let mut active_process = "unknown".to_string();

            if app_name_str == "Terminal" {
                window_title_str = Self::get_terminal_directory();
                active_process = Self::get_terminal_process();
            } else if app_name_str.contains("Google Chrome")
                || app_name_str.contains("Safari")
                || app_name_str.contains("Firefox")
            {
                window_title_str = Self::get_browser_active_tab(&bundle_id_str);
            }

            log::info!("Finalizing active window details...");
            pool.drain();

            Some(Window {
                app_name: app_name_str,
                title: window_title_str,
                bundle_id: bundle_id_str,
                active_process,
            })
        });

        match result {
            Ok(value) => {
                log::info!("Active window retrieved: {:?}", value);
                value
            }
            Err(_) => {
                log::error!("Failed to get active window (macOS API exception)");
                None
            }
        }
    }

    fn get_window_title() -> String {
        log::info!("Getting window title...");

        let result = panic::catch_unwind(|| unsafe {
            let pool = NSAutoreleasePool::new(nil);

            log::info!("Getting current application...");
            let class_name = c"NSRunningApplication";
            let window_info_class = AnyClass::get(class_name);
            if window_info_class.is_none() {
                log::error!("Failed to get NSRunningApplication class.");
                return "unknown".to_string();
            }

            let active_app: Option<Retained<AnyObject>> =
                msg_send![window_info_class.unwrap(), currentApplication];
            if active_app.is_none() {
                log::warn!("No active application found.");
                return "unknown".to_string();
            }

            log::info!("Fetching window title using `performSelector`...");
            let selector = sel!(localizedName);
            let window_title: Option<&AnyObject> = active_app
                .as_ref()
                .map(|app| msg_send![app, performSelector: selector]);

            if let Some(window_title) = window_title {
                let s: *const i8 = msg_send![window_title, UTF8String];
                let title_str = std::ffi::CStr::from_ptr(s).to_string_lossy().into_owned();
                log::info!("Window title: {}", title_str);
                pool.drain();
                return title_str;
            }

            log::warn!("Window title is nil.");
            pool.drain();
            "unknown".to_string()
        });

        result.unwrap_or_else(|_| {
            log::error!("Failed to get window title (macOS API exception)");
            "unknown".to_string()
        })
    }

    pub fn get_terminal_directory() -> String {
        let script = r#"
        tell application "Terminal"
            try
                if (count of windows) > 0 then
                    set frontWindow to front window
                    if (count of tabs of frontWindow) > 0 then
                        set frontTab to selected tab of frontWindow
                        return frontTab's current settings's title
                    end if
                end if
            end try
        end tell
        "#;

        let output = Self::run_osascript(script);

        if output == "unknown" || output.is_empty() {
            // Fallback: Use `lsof` to get the working directory of the frontmost Terminal process
            let pid_output = Command::new("pgrep")
                .arg("-n")
                .arg("Terminal")
                .output()
                .ok()
                .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string());

            if let Some(pid) = pid_output {
                let lsof_output = Command::new("lsof")
                    .arg("-a")
                    .arg("-p")
                    .arg(&pid)
                    .arg("-d")
                    .arg("cwd")
                    .arg("-Fn")
                    .output()
                    .ok()
                    .map(|out| {
                        out.stdout
                            .split(|&b| b == b'\n')
                            .find(|line| line.starts_with(b"n"))
                            .map(|line| String::from_utf8_lossy(&line[1..]).trim().to_string())
                    });

                return lsof_output.flatten().unwrap_or("unknown".to_string());
            }
        }

        output
    }

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

        let window_title = Self::run_osascript(script);

        if !window_title.is_empty() && window_title != "No Active Terminal" {
            window_title
        } else {
            "unknown".to_string()
        }
    }

    pub fn get_browser_active_tab(bundle_id: &str) -> String {
        let script = match bundle_id {
            "com.google.Chrome" => {
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
            "org.mozilla.firefox" => {
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
            "com.apple.Safari" => {
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

        let output = Self::run_osascript(script);
        if output == "No active tab" || output.is_empty() {
            log::warn!("No active tab detected for {}", bundle_id);
            return "unknown".to_string();
        }

        log::info!("Active tab URL detected: {}", output);
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
            log::error!("Failed to execute AppleScript: {}", script);
            "unknown".to_string()
        })
    }
}
