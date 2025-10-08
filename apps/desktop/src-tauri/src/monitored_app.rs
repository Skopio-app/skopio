use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::LazyLock,
};

use common::language::detect_language;
use serde::Serialize;
use strum_macros::{Display, EnumString};

use crate::{
    trackers::{ax::types::AxSnapshot, window_tracker::WindowTracker},
    utils::{app::run_osascript, config::TrackedApp},
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

pub struct AppDetails {
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub entity: String,
    pub entity_type: Entity,
    pub category: Category,
    pub language: Option<String>,
}

fn get_entity_for_app(app: &MonitoredApp) -> Entity {
    if BROWSER_APPS.contains(app) {
        Entity::Url
    } else if *app == MonitoredApp::Xcode {
        Entity::File
    } else {
        Entity::App
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

fn get_xcode_category(entity: &str) -> Category {
    let is_compling = run_osascript("tell application \"Xcode\" to get build status") == "Building";

    if is_compling {
        return Category::Compiling;
    }

    let is_debugging = run_osascript("tell application \"Xcode\" to get run state") == "Running";

    if is_debugging {
        return Category::Debugging;
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

fn get_category_for_app(app: &MonitoredApp, entity: Option<&str>, url: Option<&str>) -> Category {
    if let Some(category) = APP_CATEGORIES
        .iter()
        .find(|(a, _)| a == app)
        .map(|(_, c)| c)
    {
        return category.clone();
    }

    if BROWSER_APPS.contains(app) {
        if let Some(url) = url {
            return get_browser_category(url);
        }
        return Category::Browsing;
    }

    if *app == MonitoredApp::Xcode {
        return get_xcode_category(entity.unwrap_or_default());
    }

    Category::Other
}

pub fn resolve_app_details(
    app: &MonitoredApp,
    app_name: &str,
    app_path: &str,
    entity: &str,
    snapshot: &AxSnapshot,
) -> AppDetails {
    match app {
        MonitoredApp::Xcode => {
            let xi = snapshot.xcode.clone().unwrap_or_default();
            let language = detect_language(&xi.entity_path);
            // let (project_name, project_path, entity, lang) = get_xcode_project_details();
            AppDetails {
                project_name: xi.project_name,
                project_path: xi.project_path,
                entity: xi.entity_path.clone(),
                language,
                entity_type: Entity::File,
                category: get_category_for_app(app, Some(&xi.entity_path), None),
            }
        }
        _ if BROWSER_APPS.contains(app) => {
            let bi = snapshot.browser.clone().unwrap_or_default();
            // let (domain, url, tab) = get_browser_active_tab(app);
            AppDetails {
                project_name: Some(bi.domain),
                project_path: Some(bi.url.clone()),
                entity: bi.path,
                language: None,
                category: get_category_for_app(app, None, Some(&bi.url)),
                entity_type: get_entity_for_app(app),
            }
        }
        _ => AppDetails {
            project_name: Some(app_name.to_lowercase()),
            project_path: Some(app_path.to_string()),
            entity: entity.to_string(),
            entity_type: Entity::App,
            category: get_category_for_app(app, None, None),
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
