#![cfg(target_os = "macos")]

use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::AnyObject;
use objc2::{msg_send, sel};
use objc2_app_kit::{NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace};
use objc2_foundation::{NSArray, NSString};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::watch;
use tokio::time::Duration;
use tracing::{debug, error, info};

use crate::trackers::TrackerLifecycle;
use crate::utils::ax::ffi::AxElement;
use crate::utils::config::TrackedApp;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Represents an actively tracked app window.
#[derive(Clone, PartialEq, Debug)]
pub struct Window {
    pub app_name: Arc<str>,
    pub title: Arc<str>,
    pub bundle_id: Arc<str>,
    /// Refers to the executable path of the app binary
    pub path: Arc<str>,
    pub pid: i32,
}

#[derive(Clone)]
pub struct WindowTracker {
    rx: watch::Receiver<Option<Window>>,
    tx: watch::Sender<Option<Window>>,
    shutdown: Arc<AtomicBool>,
}

impl WindowTracker {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(None);
        Self {
            tx,
            rx,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_tracking(self: Arc<Self>) {
        self.shutdown.store(false, Ordering::Relaxed);
        let tx = self.tx.clone();
        let shutdown = Arc::clone(&self.shutdown);

        match std::thread::Builder::new()
            .name("skopio-window-tracker".into())
            .spawn(move || {
                let mut last_window: Option<Window> = None;

                loop {
                    if shutdown.load(Ordering::Relaxed) {
                        break;
                    }

                    let current_window = Self::get_active_window();

                    if current_window != last_window {
                        match &current_window {
                            Some(window) => {
                                debug!(
                                    "Window switched to {} ({}) with title {}",
                                    window.app_name, window.bundle_id, window.title
                                );
                            }
                            None => {
                                debug!("No active window");
                            }
                        }

                        if tx.send(current_window.clone()).is_err() {
                            debug!("No subscribers to receive active window update");
                        }

                        last_window = current_window;
                    }

                    std::thread::sleep(POLL_INTERVAL);
                }
            }) {
            Ok(_handle) => {}
            Err(error) => {
                error!("Failed to spawn window tracker thread: {error}")
            }
        }
    }

    fn get_active_window() -> Option<Window> {
        autoreleasepool(|_| unsafe {
            let workspace: Retained<NSWorkspace> = NSWorkspace::sharedWorkspace();

            let front_app: *mut AnyObject = msg_send![&*workspace, frontmostApplication];
            if front_app.is_null() {
                return None;
            }

            let app_name = nsobject_string_msg(front_app, sel!(localizedName))
                .unwrap_or_else(|| Arc::from("unknown"));

            let bundle_id = nsobject_string_msg(front_app, sel!(bundleIdentifier))
                .unwrap_or_else(|| Arc::from("unknown"));

            let path = executable_path(front_app).unwrap_or_else(|| Arc::from("unknown"));

            let pid: i32 = msg_send![front_app, processIdentifier];

            let title = Self::get_active_window_title(pid)
                .map(|title| Arc::from(title.into_boxed_str()))
                .unwrap_or_else(|| Arc::from("unknown"));

            let window = Window {
                app_name,
                title,
                bundle_id,
                path,
                pid,
            };

            Some(window)
        })
    }

    fn get_active_window_title(pid: i32) -> Option<String> {
        unsafe {
            let app = AxElement::app(pid)?;
            let win = app.focused_window()?;
            win.title()
        }
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
        info!("Window tracker stopped");
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
impl TrackerLifecycle for WindowTracker {
    type StartArgs = ();

    fn start_tracking(self: Arc<Self>, (): Self::StartArgs) {
        WindowTracker::start_tracking(self);
    }

    async fn shutdown(&self) {
        self.stop_tracking();
    }
}

impl Default for WindowTracker {
    fn default() -> Self {
        Self::new()
    }
}

unsafe fn nsstring_to_string(ns: Option<Retained<NSString>>) -> Option<String> {
    ns.map(|s| s.to_string())
}

unsafe fn nsobject_string_msg(
    object: *mut AnyObject,
    selector: objc2::runtime::Sel,
) -> Option<Arc<str>> {
    let value: *mut AnyObject = msg_send![object, performSelector: selector];

    unsafe { nsobject_to_arc_str(value) }
}

unsafe fn nsobject_to_arc_str(value: *mut AnyObject) -> Option<Arc<str>> {
    if value.is_null() {
        return None;
    }

    let c_str: *const i8 = msg_send![value, UTF8String];
    if c_str.is_null() {
        return None;
    }

    Some(Arc::from(unsafe {
        std::ffi::CStr::from_ptr(c_str)
            .to_string_lossy()
            .into_owned()
            .into_boxed_str()
    }))
}

unsafe fn executable_path(app: *mut AnyObject) -> Option<Arc<str>> {
    let url: *mut AnyObject = msg_send![app, executableURL];
    if url.is_null() {
        return None;
    }

    let path: *mut AnyObject = msg_send![url, path];
    unsafe { nsobject_to_arc_str(path) }
}
