use cocoa::base::nil;
use cocoa::foundation::NSAutoreleasePool;
use log::warn;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::helpers::app::{get_browser_active_tab, get_terminal_process, run_osascript};
use crate::monitored_app::MonitoredApp;

#[derive(Clone, PartialEq, Debug)]
pub struct Window {
    pub app_name: String,
    pub title: String,
    pub bundle_id: String,
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

        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(500));

            let new_window = WindowTracker::get_active_window();
            if let Some(new_window) = new_window {
                let mut active_window_lock = active_window.lock().unwrap();

                if *active_window_lock != Some(new_window.clone()) {
                    *active_window_lock = Some(new_window.clone());
                    event_callback(new_window);
                }
            }
        });
    }

    pub fn get_active_window() -> Option<Window> {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            let class_name = c"NSWorkspace";
            let workspace_class = AnyClass::get(class_name)?;
            let workspace: Retained<AnyObject> = msg_send![workspace_class, sharedWorkspace];

            let front_app: Option<&AnyObject> = msg_send![&*workspace, frontmostApplication];
            if front_app.is_none() {
                warn!("No active application found.");
                return None;
            }
            let app_name: Option<String> = front_app.and_then(|app| {
                let app_name: Option<&AnyObject> = msg_send![app, localizedName];
                app_name.map(|app_name| {
                    let s: *const i8 = msg_send![app_name, UTF8String];
                    std::ffi::CStr::from_ptr(s).to_string_lossy().into_owned()
                })
            });

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

            let window_title_str = Self::get_active_window_title(&app_name_str, &bundle_id_str);
            pool.drain();

            let window = Window {
                app_name: app_name_str,
                title: window_title_str,
                bundle_id: bundle_id_str,
            };

            Some(window)
        }
    }

    fn get_active_window_title(app_name: &str, bundle_id: &str) -> String {
        if app_name == "Terminal" {
            return get_terminal_process();
        } else if ["Google Chrome", "Safari", "Firefox"].contains(&app_name) {
            return get_browser_active_tab(
                &bundle_id
                    .parse::<MonitoredApp>()
                    .unwrap_or(MonitoredApp::Unknown),
            );
        }

        let script = format!(
            r#"
            tell application "System Events"
                tell process "{}"
                    if exists (attribute "AXFocusedUIElement") then
                        try
                            return value of attribute "AXTitle" of AXFocusedUIElement
                        on error
                            return "missing"
                        end try
                    end if
                end tell
            end tell
            "#,
            app_name
        );

        let title = run_osascript(&script);
        if !title.is_empty() && title != "missing" {
            return title;
        }

        let fallback_script = format!(
            r#"
            tell application "System Events"
                tell process "{}"
                    repeat with win in every window
                        if value of attribute "AXMain" of win is true then
                            try
                                return value of attribute "AXTitle" of win
                            on error
                                return "missing"
                            end try
                        end if
                    end repeat
                end tell
            end tell
            "#,
            app_name
        );

        let fallback_title = run_osascript(&fallback_script);
        if !fallback_title.is_empty() && fallback_title != "missing" {
            return fallback_title;
        }

        "unknown".to_string()
    }
}
