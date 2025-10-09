use thiserror::Error;
use url::ParseError;

#[derive(Debug, Error)]
pub enum AxError {
    #[error("Accessibility permission not granted")]
    AccessibilityNotGranted,

    #[error("No frontmost application")]
    NoFrontmostApplication,

    #[error("No focused window")]
    NoFocusedWindow,

    #[error("AX traversal failed: {0}")]
    AxTraversalFailed(&'static str),

    #[error("Unsupported application for this query")]
    UnsupportedApp,

    #[error("Requested info not available")]
    NotAvailable,

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Unknown AX error")]
    Unknown,
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
