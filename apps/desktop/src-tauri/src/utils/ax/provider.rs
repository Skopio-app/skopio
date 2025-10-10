use url::{Position, Url};

use crate::{
    monitored_app::{MonitoredApp, BROWSER_APPS},
    utils::ax::{
        ffi::AxElement,
        types::{AxError, BrowserInfo, XcodeInfo},
        util::{derive_xcode_project_name, infer_xcode_root, normalize_file},
    },
};

pub trait AxProvider: Send + Sync {
    fn browser_info(&self, bundle_id: &str, pid: i32) -> Result<BrowserInfo, AxError>;
    fn xcode_info(&self, pid: i32) -> Result<XcodeInfo, AxError>;
}

/// System implementation (macOS)
pub struct SystemAxProvider;

impl AxProvider for SystemAxProvider {
    fn browser_info(&self, bundle_id: &str, pid: i32) -> Result<BrowserInfo, AxError> {
        let monitored_browser = bundle_id
            .parse::<MonitoredApp>()
            .unwrap_or(MonitoredApp::Unknown);
        if !BROWSER_APPS.contains(&monitored_browser) {
            return Err(AxError::UnsupportedApp);
        }

        unsafe {
            let app = AxElement::app(pid).ok_or(AxError::AccessibilityNotGranted)?;
            let win = app.focused_window().ok_or(AxError::NoFocusedWindow)?;
            let web_area = win.find_descendants("AXWebArea", 12);

            let url = match web_area {
                Some(wa) => wa.url(),
                None => win.url(),
            }
            .unwrap_or_default()
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
            let app = AxElement::app(pid).ok_or(AxError::AccessibilityNotGranted)?;
            let win = app.focused_window().ok_or(AxError::NoFocusedWindow)?;
            let raw = win.document().unwrap_or_default();

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
