use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::LazyLock,
};

use common::language::detect_language;
use serde::Serialize;
use strum_macros::{Display, EnumString};

use crate::{
    trackers::window_tracker::WindowTracker,
    utils::{
        ax::{
            ffi::AxElement,
            types::{AxError, AxSnapshot},
        },
        config::TrackedApp,
    },
};

pub static BROWSER_APPS: LazyLock<HashSet<MonitoredApp>> = LazyLock::new(|| {
    HashSet::from([
        MonitoredApp::Brave,
        MonitoredApp::Chrome,
        MonitoredApp::ChromeBeta,
        MonitoredApp::ChromeCanary,
        MonitoredApp::Safari,
        MonitoredApp::SafariPreview,
        MonitoredApp::Firefox,
        MonitoredApp::ArcBrowser,
        MonitoredApp::Dia,
    ])
});

static CODE_REVIEW_URLS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["github.com", "gitlab.com", "bitbucket.org"]));

static MEETING_URLS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "meet.google.com",
        "zoom.us",
        "teams.microsoft.com",
        "webex.com",
        "slack.com/call",
    ])
});

static LEARNING_URLS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "udemy.com",
        "coursera.org",
        "khanacademy.org",
        "codecademy.com",
        "educative.io",
    ])
});

pub static IGNORED_APPS: LazyLock<HashMap<MonitoredApp, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (
            MonitoredApp::Code,
            "An editor extension for Visual Studio Code is available to capture more accurate data",
        ),
        (
            MonitoredApp::Windsurf,
            "An editor extension for Windsurf is available to capture more accurate data",
        ),
        (
            MonitoredApp::Cursor,
            "An editor extension for Cursor is available to capture more accurate data",
        ),
    ])
});

static CODING_URLS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "leetcode.com",
        "stackoverflow.com",
        "w3schools.com",
        "developer.mozilla.org",
        "codewars.com",
    ])
});

static APP_CATEGORIES: LazyLock<HashSet<(MonitoredApp, Category)>> = LazyLock::new(|| {
    HashSet::from([
        (MonitoredApp::Figma, Category::Designing),
        (MonitoredApp::Notion, Category::Planning),
        (MonitoredApp::Zoom, Category::Meeting),
        (MonitoredApp::Github, Category::CodeReviewing),
        (MonitoredApp::Postman, Category::Debugging),
        (MonitoredApp::Warp, Category::Coding),
        (MonitoredApp::Terminal, Category::Coding),
        (MonitoredApp::Iterm, Category::Coding),
    ])
});

/// A list of monitored applications for tracking user activity.
///
/// Each variant corresponds to an application identified by its bundle ID
#[derive(Debug, EnumString, Display, PartialEq, Eq, Hash, Clone, Copy)]
pub enum MonitoredApp {
    #[strum(serialize = "com.brave.Browser")]
    Brave,
    #[strum(serialize = "com.google.Chrome")]
    Chrome,
    #[strum(serialize = "com.google.Chrome.beta")]
    ChromeBeta,
    #[strum(serialize = "com.google.Chrome.canary")]
    ChromeCanary,
    #[strum(serialize = "com.microsoft.VSCode")]
    Code,
    #[strum(serialize = "com.figma.Desktop")]
    Figma,
    #[strum(serialize = "org.mozilla.firefox")]
    Firefox,
    #[strum(serialize = "com.github.GithubClient")]
    Github,
    #[strum(serialize = "com.postmanlabs.mac")]
    Postman,
    #[strum(serialize = "com.apple.Safari")]
    Safari,
    #[strum(serialize = "com.apple.SafariTechnologyPreview")]
    SafariPreview,
    #[strum(serialize = "com.apple.Terminal")]
    Terminal,
    #[strum(serialize = "com.googlecode.iterm2")]
    Iterm,
    #[strum(serialize = "com.apple.dt.Xcode")]
    Xcode,
    #[strum(serialize = "notion.id")]
    Notion,
    #[strum(serialize = "company.thebrowser.Browser")]
    ArcBrowser,
    #[strum(serialize = "company.thebrowser.dia")]
    Dia,
    #[strum(serialize = "dev.warp.Warp-Stable")]
    Warp,
    #[strum(serialize = "us.zoom.xos")]
    Zoom,
    #[strum(serialize = "com.exafunction.windsurf")]
    Windsurf,
    #[strum(serialize = "com.todesktop.230313mzl4w4u92")]
    Cursor,
    /// Fallback for unrecognized applications
    #[strum(serialize = "unknown")]
    Unknown,
}

/// Categories representing the type of activity detected in an application.
///
/// Used to classify user activity based on the application being used or the URL visited.
#[derive(Clone, Debug, EnumString, PartialEq, Eq, Hash, Display)]
pub enum Category {
    Browsing,
    Coding,
    Compiling,
    Debugging,
    Designing,
    #[strum(serialize = "Code Reviewing")]
    CodeReviewing,
    Meeting,
    Learning,
    Planning,
    #[strum(serialize = "Writing Docs")]
    WritingDocs,
    Other,
}

/// Defines the type of entity being tracked in a monitored application.
///
/// This helps determine whether the entity being logged is a file, an application or a URL.
#[derive(PartialEq, Clone, Debug, EnumString, Display)]
pub enum Entity {
    File,
    App,
    Url,
}

#[derive(Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OpenApp {
    app: TrackedApp,
    block_reason: Option<String>,
}

#[derive(Debug)]
pub struct AppDetails {
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub entity: String,
    pub entity_type: Entity,
    pub category: Category,
    pub language: Option<String>,
}

impl MonitoredApp {
    fn get_category(&self, entity: Option<&str>, url: Option<&str>, pid: i32) -> Category {
        if let Some(category) = APP_CATEGORIES
            .iter()
            .find(|(a, _)| a == self)
            .map(|(_, c)| c)
        {
            return category.clone();
        }

        if BROWSER_APPS.contains(self) {
            if let Some(url) = url {
                return get_browser_category(url);
            }
            return Category::Browsing;
        }

        if *self == MonitoredApp::Xcode {
            return get_xcode_category(entity.unwrap_or_default(), pid);
        }

        Category::Other
    }

    fn get_entity_type(&self) -> Entity {
        if BROWSER_APPS.contains(self) {
            Entity::Url
        } else if *self == MonitoredApp::Xcode {
            Entity::File
        } else {
            Entity::App
        }
    }
}

fn get_browser_category(url: &str) -> Category {
    if CODE_REVIEW_URLS
        .iter()
        .any(|&review_url| url.contains(review_url))
    {
        return Category::CodeReviewing;
    }
    if MEETING_URLS
        .iter()
        .any(|&meeting_url| url.contains(meeting_url))
    {
        return Category::Meeting;
    }
    if LEARNING_URLS
        .iter()
        .any(|&learning_url| url.contains(learning_url))
    {
        return Category::Learning;
    }
    if CODING_URLS
        .iter()
        .any(|&coding_url| url.contains(coding_url))
    {
        return Category::Coding;
    }
    Category::Browsing
}

pub unsafe fn is_xcode_compiling(pid: i32) -> Result<bool, AxError> {
    let app = AxElement::app(pid).ok_or(AxError::AccessibilityNotGranted)?;
    let win = match app.focused_window() {
        Some(w) => w,
        None => return Ok(false),
    };

    if let Some(toolbar) = win.find_descendant("AXToolbar", 6) {
        let mut stack = vec![toolbar.clone()];
        while let Some(node) = stack.pop() {
            if let Some(role) = node.role() {
                if role == "AXProgressIndicator" {
                    if let Some(v) = node.number_attr_f64("AXValue") {
                        if v.is_finite() && v > 0.0 && v <= 1.0 {
                            return Ok(true);
                        }
                    }

                    if node
                        .string_attr("AXDescription")
                        .or_else(|| node.title())
                        .map(|s| s.contains("Build") || s.contains("Compile") || s.contains("Link"))
                        .unwrap_or(false)
                    {
                        return Ok(true);
                    }
                }

                if role == "AXStaticText"
                    && node
                        .title()
                        .or_else(|| node.string_attr("AXDescription"))
                        .map(|s| {
                            s.contains("Building")
                                || s.contains("Compiling")
                                || s.contains("Linking")
                        })
                        .unwrap_or(false)
                {
                    return Ok(true);
                }
            }
            stack.extend(node.children());
        }
    }

    if let Some(text) = win
        .find_descendant("AXStaticText", 12)
        .and_then(|el| el.title().or_else(|| el.string_attr("AXDescription")))
    {
        if text.contains("Building") || text.contains("Compiling") || text.contains("Linking") {
            return Ok(true);
        }
    }

    Ok(false)
}

pub unsafe fn is_xcode_debugging(pid: i32) -> Result<bool, AxError> {
    let app = AxElement::app(pid).ok_or(AxError::AccessibilityNotGranted)?;
    let win = match app.focused_window() {
        Some(w) => w,
        None => return Ok(false),
    };

    if let Some(toolbar) = win.find_descendant("AXToolbar", 6) {
        let mut stack = vec![toolbar.clone()];
        while let Some(node) = stack.pop() {
            if let Some(role) = node.role() {
                if role == "AXButton" {
                    if let Some(id) = node.identifier() {
                        if (id.contains("Debugger") && id.contains("Stop"))
                            && node.enabled().unwrap_or(false)
                        {
                            return Ok(true);
                        }
                    }

                    let labeled_stop = node
                        .title()
                        .or_else(|| node.string_attr("AXDescription"))
                        .map(|s| s.contains("Stop"))
                        .unwrap_or(false);
                    if labeled_stop && node.enabled().unwrap_or(false) {
                        return Ok(true);
                    }
                }

                if role == "AXStaticText"
                    && node
                        .title()
                        .or_else(|| node.string_attr("AXDescription"))
                        .map(|s| s.contains("Running"))
                        .unwrap_or(false)
                {
                    return Ok(true);
                }
            }
            stack.extend(node.children());
        }
    }

    Ok(false)
}

fn get_xcode_category(entity: &str, pid: i32) -> Category {
    unsafe {
        if is_xcode_compiling(pid).unwrap_or(false) {
            return Category::Compiling;
        }

        if is_xcode_debugging(pid).unwrap_or(false) {
            return Category::Debugging;
        }
    }

    if is_documentation_entity(entity) {
        return Category::WritingDocs;
    }

    Category::Coding
}

fn is_documentation_entity(entity_path: &str) -> bool {
    if let Some(language) = detect_language(entity_path) {
        let doc_languages = HashSet::from(["Markdown", "Org", "ReStructuredText", "Text", "Tex"]);
        return doc_languages.contains(language.as_str());
    }
    false
}

pub fn resolve_app_details(
    app: &MonitoredApp,
    app_name: &str,
    app_path: &str,
    entity: &str,
    snapshot: &AxSnapshot,
    pid: i32,
) -> AppDetails {
    match app {
        MonitoredApp::Xcode => {
            let mut xi = snapshot.xcode.clone().unwrap_or_default();
            if xi.entity_path.trim().is_empty() || xi.entity_path == "unknown" {
                xi.entity_path = snapshot
                    .window_title
                    .clone()
                    .unwrap_or_else(|| "unknown".into());
            }
            if xi
                .project_path
                .as_deref()
                .map(str::is_empty)
                .unwrap_or(true)
            {
                xi.project_path = Some(app_path.to_string());
            }
            if xi
                .project_name
                .as_deref()
                .map(str::is_empty)
                .unwrap_or(true)
            {
                xi.project_name = Some(app_name.to_lowercase())
            }
            let language = detect_language(&xi.entity_path);
            AppDetails {
                project_name: xi.project_name,
                project_path: xi.project_path,
                entity: xi.entity_path.clone(),
                language,
                entity_type: app.get_entity_type(),
                category: app.get_category(Some(&xi.entity_path), None, pid),
            }
        }
        _ if BROWSER_APPS.contains(app) => {
            if let Some(bi) = snapshot
                .browser
                .clone()
                .filter(|b| !b.url.is_empty() && !b.domain.is_empty())
            {
                AppDetails {
                    project_name: Some(bi.domain),
                    project_path: Some(bi.url.clone()),
                    entity: bi.path,
                    language: None,
                    category: app.get_category(None, Some(&bi.url), pid),
                    entity_type: app.get_entity_type(),
                }
            } else {
                AppDetails {
                    project_name: Some(app_name.to_lowercase()),
                    project_path: Some(app_path.to_string()),
                    entity: entity.to_string(),
                    language: None,
                    category: app.get_category(None, None, pid),
                    entity_type: Entity::App,
                }
            }
        }
        _ => AppDetails {
            project_name: Some(app_name.to_lowercase()),
            project_path: Some(app_path.to_string()),
            entity: entity.to_string(),
            entity_type: app.get_entity_type(),
            category: app.get_category(None, None, pid),
            language: None,
        },
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_open_apps() -> Vec<OpenApp> {
    WindowTracker::list_open_apps()
        .into_iter()
        .map(|info| {
            let monitored =
                MonitoredApp::from_str(&info.bundle_id).unwrap_or(MonitoredApp::Unknown);
            let block_reason = IGNORED_APPS.get(&monitored).map(|&s| s.to_string());

            OpenApp {
                app: TrackedApp {
                    name: info.name,
                    bundle_id: info.bundle_id,
                },
                block_reason,
            }
        })
        .collect()
}
