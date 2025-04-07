use cocoa::base::nil;
use cocoa::foundation::NSAutoreleasePool;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use log::warn;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use std::os::raw::c_void;
use std::ptr;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

// use crate::helpers::app::run_osascript;

/// Represents an actively tracked app window.
#[derive(Clone, PartialEq, Debug)]
pub struct Window {
    pub app_name: String,
    pub title: String,
    pub bundle_id: String,
    /// Refers to the executable path of a binary
    pub path: String,
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

        tokio::spawn(async move {
            sleep(Duration::from_millis(500)).await;

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
            let front_app_ptr: *mut AnyObject = msg_send![&*workspace, frontmostApplication];
            if front_app_ptr.is_null() {
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

            let app_path: Option<String> = front_app.and_then(|app| {
                let url: *mut AnyObject = msg_send![app, executableURL];
                if url.is_null() {
                    return None;
                }
                let path_obj: *const AnyObject = msg_send![url, path];
                if path_obj.is_null() {
                    return None;
                }
                let c_str: *const i8 = msg_send![path_obj, UTF8String];
                if c_str.is_null() {
                    return None;
                }
                Some(
                    std::ffi::CStr::from_ptr(c_str)
                        .to_string_lossy()
                        .into_owned(),
                )
            });
            let pid: i32 = msg_send![front_app_ptr, processIdentifier];

            let app_name_str = app_name.unwrap_or_else(|| "unknown".to_string());
            let bundle_id_str = bundle_id.unwrap_or_else(|| "unknown".to_string());
            let app_path_str = app_path.unwrap_or_else(|| "unknown".to_string());

            let window_title_str = Self::get_active_window_title_ax(pid);
            pool.drain();

            let window = Window {
                app_name: app_name_str,
                title: window_title_str,
                bundle_id: bundle_id_str,
                path: app_path_str,
            };

            Some(window)
        }
    }

    fn get_active_window_title_ax(pid: i32) -> String {
        #[link(name = "ApplicationServices", kind = "framework")]
        extern "C" {
            fn AXUIElementCopyAttributeValue(
                element: *const c_void,
                attribute: *const c_void,
                value: *mut *const c_void,
            ) -> i32;

            fn AXUIElementCreateApplication(pid: i32) -> *const c_void;
        }

        unsafe {
            let app_element = AXUIElementCreateApplication(pid);
            if app_element.is_null() {
                return "unknown".to_string();
            }

            let mut focused_window: *const c_void = ptr::null();
            let err = AXUIElementCopyAttributeValue(
                app_element,
                CFString::new("AXFocusedWindow").as_concrete_TypeRef() as *const _,
                &mut focused_window,
            );

            if err != 0 || focused_window.is_null() {
                return "unknown".to_string();
            }

            let mut title_value: *const c_void = ptr::null();
            let err = AXUIElementCopyAttributeValue(
                focused_window,
                CFString::new("AXTitle").as_concrete_TypeRef() as *const _,
                &mut title_value,
            );

            if err != 0 || title_value.is_null() {
                return "unknown".to_string();
            }

            let cf_title: CFString = CFString::wrap_under_create_rule(
                title_value as *mut core_foundation::string::__CFString,
            );
            let title = cf_title.to_string();

            if title.trim().is_empty() {
                return "unknown".to_string();
            }

            title
        }
    }

    // fn get_active_window_title(app_name: &str) -> String {
    //     let script = format!(
    //         r#"
    //         tell application "System Events"
    //             tell process "{}"
    //                 if exists (attribute "AXFocusedUIElement") then
    //                     try
    //                         return value of attribute "AXTitle" of AXFocusedUIElement
    //                     on error
    //                         return "missing"
    //                     end try
    //                 end if
    //             end tell
    //         end tell
    //         "#,
    //         app_name
    //     );

    //     let title = run_osascript(&script);
    //     if !title.is_empty() && title != "missing" {
    //         return title;
    //     }

    //     let fallback_script = format!(
    //         r#"
    //         tell application "System Events"
    //             tell process "{}"
    //                 repeat with win in every window
    //                     if value of attribute "AXMain" of win is true then
    //                         try
    //                             return value of attribute "AXTitle" of win
    //                         on error
    //                             return "missing"
    //                         end try
    //                     end if
    //                 end repeat
    //             end tell
    //         end tell
    //         "#,
    //         app_name
    //     );

    //     let fallback_title = run_osascript(&fallback_script);
    //     if !fallback_title.is_empty() && fallback_title != "missing" {
    //         return fallback_title;
    //     }

    //     "unknown".to_string()
    // }
}
