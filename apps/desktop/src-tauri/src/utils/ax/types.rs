use thiserror::Error;
use url::ParseError;

use crate::trackers::window_tracker::Window;

#[derive(Debug, Error)]
pub enum AxError {
    #[error("Accessibility permission not granted")]
    AccessibilityNotGranted,

    #[error("No focused window")]
    NoFocusedWindow,

    #[error("Unsupported application for this query")]
    UnsupportedApp,

    #[error("Requested info not available")]
    NotAvailable,

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveApp {
    pub name: String,
    pub bundle_id: String,
    pub pid: i32,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BrowserInfo {
    pub domain: String,
    pub url: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XcodeInfo {
    pub entity_path: String,
    pub project_path: Option<String>,
    pub project_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AxSnapshot {
    pub app: Option<ActiveApp>,
    pub window_title: Option<String>,
    pub browser: Option<BrowserInfo>,
    pub xcode: Option<XcodeInfo>,
}

impl From<&Window> for ActiveApp {
    fn from(window: &Window) -> Self {
        ActiveApp {
            name: window.app_name.to_string(),
            bundle_id: window.bundle_id.to_string(),
            pid: window.pid,
            path: window.path.to_string(),
        }
    }
}
