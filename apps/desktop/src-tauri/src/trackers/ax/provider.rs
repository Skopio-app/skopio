use objc2::{msg_send, rc::Retained, runtime::AnyClass};
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_foundation::{NSString, NSURL};
use url::{Position, Url};

use crate::{
    monitored_app::{MonitoredApp, BROWSER_APPS},
    trackers::ax::{
        ffi::{ax_app, ax_document, ax_find_descendant, ax_focused_window, ax_title, ax_url},
        types::*,
        util::{derive_xcode_project_name, infer_xcode_root, normalize_file},
    },
};

pub trait AxProvider: Send + Sync {
    fn frontmost_app(&self) -> Result<ActiveApp, AxError>;
    fn focused_window_title(&self, pid: i32) -> Result<String, AxError>;
    fn browser_info(&self, bundle_id: &str, pid: i32) -> Result<BrowserInfo, AxError>;
    fn xcode_info(&self, pid: i32) -> Result<XcodeInfo, AxError>;
}

/// System implementation (macOS)
pub struct SystemAxProvider;

impl AxProvider for SystemAxProvider {
    fn frontmost_app(&self) -> Result<ActiveApp, AxError> {
        unsafe {
            let cls = AnyClass::get(c"NSWorkspace").ok_or(AxError::Unknown)?;
            let ws: Retained<NSWorkspace> = msg_send![cls, sharedWorkspace];

            let app: Option<Retained<NSRunningApplication>> = msg_send![&*ws, frontmostApplication];
            let app = app.ok_or(AxError::NoFrontmostApplication)?;

            // name
            let name: String = {
                let s: Option<Retained<NSString>> = msg_send![&*app, localizedName];
                s.map(|s| s.to_string()).unwrap_or_else(|| "unknown".into())
            };

            // bundle id
            let bundle_id: String = {
                let s: Option<Retained<NSString>> = msg_send![&*app, bundleIdentifier];
                s.map(|s| s.to_string()).unwrap_or_else(|| "unknown".into())
            };

            let pid: i32 = msg_send![&*app, processIdentifier];

            // path
            let path: String = {
                let url: Option<Retained<NSURL>> = msg_send![&*app, executableURL];
                if let Some(url) = url {
                    let p: Option<Retained<NSString>> = msg_send![&*url, path];
                    p.map(|p| p.to_string()).unwrap_or_else(|| "unknown".into())
                } else {
                    "unknown".into()
                }
            };

            Ok(ActiveApp {
                name,
                bundle_id,
                pid,
                path,
            })
        }
    }

    fn focused_window_title(&self, pid: i32) -> Result<String, AxError> {
        unsafe {
            let app_el = ax_app(pid);
            if app_el.is_null() {
                return Err(AxError::AccessibilityNotGranted);
            }
            let win = ax_focused_window(app_el).ok_or(AxError::NoFocusedWindow)?;
            let title = ax_title(win).ok_or(AxError::AxTraversalFailed("AXTitle"))?;
            Ok(title)
        }
    }

    fn browser_info(&self, bundle_id: &str, pid: i32) -> Result<BrowserInfo, AxError> {
        let monitored_browser = bundle_id
            .parse::<MonitoredApp>()
            .unwrap_or(MonitoredApp::Unknown);
        if !BROWSER_APPS.contains(&monitored_browser) {
            return Err(AxError::UnsupportedApp);
        }

        unsafe {
            let app_el = ax_app(pid);
            if app_el.is_null() {
                return Err(AxError::AccessibilityNotGranted);
            }
            let win = ax_focused_window(app_el).ok_or(AxError::NoFocusedWindow)?;
            let web_area = ax_find_descendant(win, "AXWebArea", 12);

            let url = match web_area {
                Some(wa) => ax_url(wa).unwrap_or_default(),
                None => ax_url(win).unwrap_or_default(),
            }
            .trim()
            .to_string();

            if url.is_empty() {
                return Err(AxError::NotAvailable);
            }

            let parsed = Url::parse(&url).map_err(AxError::ParseError)?;

            let domain = parsed
                .domain()
                .map(str::to_string)
                .or_else(|| parsed.host_str().map(str::to_string))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| parsed.scheme().to_string());

            let path = &parsed[Position::BeforePath..];

            Ok(BrowserInfo {
                domain,
                url,
                path: path.to_string(),
            })
        }
    }

    fn xcode_info(&self, pid: i32) -> Result<XcodeInfo, AxError> {
        unsafe {
            let app = self.frontmost_app()?;
            if app
                .bundle_id
                .parse::<MonitoredApp>()
                .unwrap_or(MonitoredApp::Unknown)
                != MonitoredApp::Xcode
            {
                return Err(AxError::UnsupportedApp);
            }

            let app_el = ax_app(pid);
            if app_el.is_null() {
                return Err(AxError::AccessibilityNotGranted);
            }
            let win = ax_focused_window(app_el).ok_or(AxError::NoFocusedWindow)?;
            let raw = ax_document(win).unwrap_or_default();

            let entity_path = normalize_file(&raw).unwrap_or_else(|| "unknown".into());
            let project_path = if entity_path != "unknown" {
                infer_xcode_root(&entity_path).map(|p| p.to_string_lossy().to_string())
            } else {
                None
            };

            let project_name = project_path.as_ref().and_then(derive_xcode_project_name);

            Ok(XcodeInfo {
                entity_path,
                project_path,
                project_name,
            })
        }
    }
}
