#![cfg(target_os = "macos")]

use core_foundation::base::{CFRelease, TCFType};
use core_foundation::string::CFString;
use log::{debug, info, warn};
use objc2::msg_send;
use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::{AnyClass, AnyObject};
use objc2_app_kit::{NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace};
use objc2_foundation::{NSArray, NSString};
use std::os::raw::c_void;
use std::ptr;
use std::sync::Arc;
use tokio::sync::{watch, Notify};
use tokio::time::{interval, Duration};

use crate::utils::config::TrackedApp;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCopyAttributeValue(
        element: *const c_void,
        attribute: *const c_void,
        value: *mut *const c_void,
    ) -> i32;

    fn AXUIElementCreateApplication(pid: i32) -> *const c_void;
}

/// Represents an actively tracked app window.
#[derive(Clone, PartialEq, Debug)]
pub struct Window {
    pub app_name: Arc<str>,
    pub title: Arc<str>,
    pub bundle_id: Arc<str>,
    /// Refers to the executable path of the app binary
    pub path: Arc<str>,
}

#[derive(Clone)]
pub struct WindowTracker {
    rx: watch::Receiver<Option<Window>>,
    tx: watch::Sender<Option<Window>>,
    shutdown: Arc<Notify>,
}

impl WindowTracker {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(None);
        Self {
            tx,
            rx,
            shutdown: Arc::new(Notify::new()),
        }
    }

    pub fn start_tracking(self: Arc<Self>) {
        let tx = self.tx.clone();
        let shutdown = Arc::clone(&self.shutdown);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(500));
            let mut last_window: Option<Window> = None;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                if let Some(window) = Self::get_active_window() {
                    let should_send = match &last_window {
                        Some(prev) if prev != &window => true,
                        None => true,
                        _ => false,
                    };

                    if should_send {
                        debug!(
                            "Window switched to {} with title {}",
                            window.app_name, window.title
                        );
                    }
                    last_window = Some(window.clone());
                    if tx.send(Some(window)).is_err() {
                        warn!("No subscribers to receive active window update");
                    }
                }
                    }

                _ = shutdown.notified() => {
                    info!("Window tracker stopped");
                    break;
                }
                }
            }
        });
    }

    fn get_active_window() -> Option<Window> {
        autoreleasepool(|_| unsafe {
            let class_name = c"NSWorkspace";
            let workspace_class = AnyClass::get(class_name)?;
            let workspace: Retained<AnyObject> = msg_send![workspace_class, sharedWorkspace];

            let front_app_ptr: *mut AnyObject = msg_send![&*workspace, frontmostApplication];
            if front_app_ptr.is_null() {
                warn!("No active application found.");
                return None;
            }

            let front_app = &*front_app_ptr;

            let app_name: Arc<str> = {
                let name_obj: Option<&AnyObject> = msg_send![front_app, localizedName];
                name_obj.map_or_else(
                    || Arc::from("unknown"),
                    |nsstr| {
                        let c_str: *const i8 = msg_send![nsstr, UTF8String];
                        Arc::from(
                            std::ffi::CStr::from_ptr(c_str)
                                .to_string_lossy()
                                .into_owned()
                                .into_boxed_str(),
                        )
                    },
                )
            };

            let bundle_id: Arc<str> = {
                let id_obj: *mut AnyObject = msg_send![front_app, bundleIdentifier];
                if id_obj.is_null() {
                    Arc::from("unknown")
                } else {
                    let c_str: *const i8 = msg_send![id_obj, UTF8String];
                    if c_str.is_null() {
                        Arc::from("unknown")
                    } else {
                        Arc::from(
                            std::ffi::CStr::from_ptr(c_str)
                                .to_string_lossy()
                                .into_owned()
                                .into_boxed_str(),
                        )
                    }
                }
            };

            let path: Arc<str> = {
                let url: *mut AnyObject = msg_send![front_app, executableURL];
                if url.is_null() {
                    Arc::from("unknown")
                } else {
                    let path_obj: *const AnyObject = msg_send![url, path];
                    if path_obj.is_null() {
                        Arc::from("unknown")
                    } else {
                        let c_str: *const i8 = msg_send![path_obj, UTF8String];
                        if c_str.is_null() {
                            Arc::from("unknown")
                        } else {
                            Arc::from(
                                std::ffi::CStr::from_ptr(c_str)
                                    .to_string_lossy()
                                    .into_owned()
                                    .into_boxed_str(),
                            )
                        }
                    }
                }
            };

            let pid: i32 = msg_send![front_app, processIdentifier];

            let title = match Self::get_active_window_title(pid) {
                Some(title) => Arc::from(title.into_boxed_str()),
                None => Arc::from("unknown"),
            };

            let window = Window {
                app_name,
                title,
                bundle_id,
                path,
            };

            Some(window)
        })
    }

    fn get_active_window_title(pid: i32) -> Option<String> {
        autoreleasepool(|_| unsafe {
            let app_element = AXUIElementCreateApplication(pid);
            if app_element.is_null() {
                return None;
            }

            let mut focused_window: *const c_void = ptr::null_mut();
            let err = AXUIElementCopyAttributeValue(
                app_element,
                CFString::new("AXFocusedWindow").as_concrete_TypeRef() as *const _,
                &mut focused_window,
            );

            if err != 0 || focused_window.is_null() {
                CFRelease(app_element);
                return None;
            }

            let mut title_value: *const c_void = ptr::null_mut();
            let err = AXUIElementCopyAttributeValue(
                focused_window,
                CFString::new("AXTitle").as_concrete_TypeRef() as *const _,
                &mut title_value,
            );

            if err != 0 || title_value.is_null() {
                CFRelease(focused_window);
                CFRelease(app_element);
                return None;
            }

            let cf_title: CFString = CFString::wrap_under_create_rule(
                title_value as *mut core_foundation::string::__CFString,
            );

            let title = cf_title.to_string();

            CFRelease(focused_window);
            CFRelease(app_element);

            if title.trim().is_empty() {
                None
            } else {
                Some(title)
            }
        })
    }

    pub fn list_open_apps() -> Vec<TrackedApp> {
        autoreleasepool(|_| unsafe {
            let ws: Retained<NSWorkspace> = NSWorkspace::sharedWorkspace();

            let running: Retained<NSArray<NSRunningApplication>> = ws.runningApplications();
            let len = running.len();

            let mut out = Vec::with_capacity(len);

            for i in 0..len {
                let app: Retained<NSRunningApplication> = running.objectAtIndex(i);
                let policy = app.activationPolicy();
                if policy == NSApplicationActivationPolicy::Accessory
                    || policy == NSApplicationActivationPolicy::Prohibited
                {
                    continue;
                }

                let name =
                    nsstring_to_string(app.localizedName()).unwrap_or_else(|| "unknown".into());
                let bundle_id =
                    nsstring_to_string(app.bundleIdentifier()).unwrap_or_else(|| "unknown".into());

                out.push(TrackedApp { name, bundle_id });
            }
            out
        })
    }

    pub fn subscribe(&self) -> watch::Receiver<Option<Window>> {
        self.rx.clone()
    }

    pub fn stop_tracking(&self) {
        self.shutdown.notify_one();
    }
}

unsafe fn nsstring_to_string(ns: Option<Retained<NSString>>) -> Option<String> {
    ns.map(|s| s.to_string())
}
