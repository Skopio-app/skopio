use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use skopio_desktop_lib::{
    monitored_app::BundleIdExt,
    trackers::window_tracker::Window,
    utils::ax::{
        cache::{AxSnapshotCache, AxSnapshotCacheConfig},
        provider::AxProvider,
        types::{AxError, BrowserInfo, XcodeInfo},
    },
};
use tokio::sync::watch;

#[derive(Default, Clone)]
struct MockProvider {
    // Scripted responses
    browser_map: Arc<Mutex<HashMap<(String, i32), Result<BrowserInfo, ()>>>>,
    xcode_map: Arc<Mutex<HashMap<i32, Result<XcodeInfo, ()>>>>,
    // call counters
    calls_browser: Arc<Mutex<usize>>,
    calls_xcode: Arc<Mutex<usize>>,
}

impl MockProvider {
    fn with_browser(self, bundle: &str, pid: i32, bi: Result<BrowserInfo, ()>) -> Self {
        self.browser_map
            .lock()
            .unwrap()
            .insert((bundle.to_string(), pid), bi);
        self
    }
    fn with_xcode(self, pid: i32, xi: Result<XcodeInfo, ()>) -> Self {
        self.xcode_map.lock().unwrap().insert(pid, xi);
        self
    }
}

impl AxProvider for MockProvider {
    fn browser_info(&self, bundle_id: &str, pid: i32) -> Result<BrowserInfo, AxError> {
        let mut c = self.calls_browser.lock().unwrap();
        *c += 1;
        match self
            .browser_map
            .lock()
            .unwrap()
            .get(&(bundle_id.to_string(), pid))
            .cloned()
        {
            Some(Ok(b)) => Ok(b),
            _ => Err(AxError::NotAvailable),
        }
    }

    fn xcode_info(&self, pid: i32) -> Result<XcodeInfo, AxError> {
        let mut c = self.calls_xcode.lock().unwrap();
        *c += 1;
        match self.xcode_map.lock().unwrap().get(&pid).cloned() {
            Some(Ok(x)) => Ok(x),
            _ => Err(AxError::NotAvailable),
        }
    }
}

// helpers
fn mk_window(name: &str, bundle: &str, path: &str, title: &str, pid: i32) -> Window {
    Window {
        app_name: Arc::from(name),
        title: Arc::from(title),
        bundle_id: Arc::from(bundle),
        path: Arc::from(path),
        pid,
    }
}

#[tokio::test]
async fn cache_returns_cached_within_max_age() {
    let (tx, rx) = watch::channel::<Option<Window>>(None);

    let pid = 111;
    let chrome = "com.google.Chrome";
    assert!(chrome.is_browser_bundle());

    let provider = MockProvider::default().with_browser(
        chrome,
        pid,
        Ok(BrowserInfo {
            domain: "example.com".into(),
            url: "http://example.com/page".into(),
            path: "/page".into(),
        }),
    );

    let cache = AxSnapshotCache::new(
        Arc::new(provider.clone()),
        rx.clone(),
        AxSnapshotCacheConfig {
            max_age: Duration::from_millis(60),
        },
    );

    tx.send_replace(Some(mk_window(
        "Google Chrome",
        chrome,
        "/Applications/Chrome",
        "Page - Chrome",
        pid,
    )));

    let first = cache.refresh_now().await;
    assert_eq!(first.app.as_ref().unwrap().bundle_id, chrome);
    assert_eq!(first.browser.as_ref().unwrap().domain, "example.com");

    provider.browser_map.lock().unwrap().insert(
        (chrome.to_string(), pid),
        Ok(BrowserInfo {
            domain: "changed.com".into(),
            url: "https://changed.com".into(),
            path: "/".into(),
        }),
    );

    let second = cache.snapshot().await;
    assert_eq!(second.browser.as_ref().unwrap().domain, "example.com");
}

#[tokio::test]
async fn cache_refreshes_after_expiry() {
    let (tx, rx) = watch::channel::<Option<Window>>(None);

    let pid = 222;
    let chrome = "com.google.Chrome";

    let provider = MockProvider::default().with_browser(
        chrome,
        pid,
        Ok(BrowserInfo {
            domain: "a.com".into(),
            url: "https://a.com".into(),
            path: "/".into(),
        }),
    );

    let cache = AxSnapshotCache::new(
        Arc::new(provider.clone()),
        rx.clone(),
        AxSnapshotCacheConfig {
            max_age: Duration::from_millis(50),
        },
    );

    tx.send_replace(Some(mk_window(
        "Google Chrome",
        chrome,
        "/Applications/Chrome",
        "Tab A",
        pid,
    )));

    let first = cache.snapshot().await;
    assert_eq!(first.browser.as_ref().unwrap().domain, "a.com");

    // Update provider; after expiry, snapshot should change
    provider.browser_map.lock().unwrap().insert(
        (chrome.to_string(), pid),
        Ok(BrowserInfo {
            domain: "b.com".into(),
            url: "https://b.com".into(),
            path: "/b".into(),
        }),
    );

    tokio::time::sleep(Duration::from_millis(60)).await;

    let second = cache.snapshot().await;
    assert_eq!(second.browser.as_ref().unwrap().domain, "b.com");
}

#[tokio::test]
async fn reuse_browser_when_error_and_same_app_title() {
    let (tx, rx) = watch::channel::<Option<Window>>(None);

    let pid = 333;
    let chrome = "com.google.Chrome";

    let provider = MockProvider::default().with_browser(
        chrome,
        pid,
        Ok(BrowserInfo {
            domain: "keep.me".into(),
            url: "https://keep.me".into(),
            path: "/p".into(),
        }),
    );

    let cache = AxSnapshotCache::new(
        Arc::new(provider.clone()),
        rx.clone(),
        AxSnapshotCacheConfig {
            max_age: Duration::from_millis(1),
        },
    );

    let w = mk_window(
        "Google Chrome",
        chrome,
        "/Applications/Chrome",
        "Same title",
        pid,
    );
    tx.send_replace(Some(w.clone()));

    let first = cache.refresh_now().await;
    assert_eq!(first.browser.as_ref().unwrap().domain, "keep.me");

    // Now make provider fail, but keep same app + title; cache should reuse previous
    provider
        .browser_map
        .lock()
        .unwrap()
        .insert((chrome.to_string(), pid), Err(()));

    // expire
    tokio::time::sleep(Duration::from_millis(2)).await;

    let second = cache.refresh_now().await;
    assert_eq!(second.browser.as_ref().unwrap().domain, "keep.me");
}

#[tokio::test]
async fn clear_browser_on_app_change_or_title_change() {
    let (tx, rx) = watch::channel::<Option<Window>>(None);

    let pid_chrome = 444;
    let chrome = "com.google.Chrome";
    let pid_xcode = 555;
    let xcode = "com.apple.dt.Xcode";

    let provider = MockProvider::default()
        .with_browser(
            chrome,
            pid_chrome,
            Ok(BrowserInfo {
                domain: "site.com".into(),
                url: "https:://site.com".into(),
                path: "/path".into(),
            }),
        )
        .with_xcode(
            pid_xcode,
            Ok(XcodeInfo {
                entity_path: "/tmp/Starter.playground".into(),
                project_path: Some("/tmp".into()),
                project_name: Some("Starter".into()),
            }),
        );

    let cache = AxSnapshotCache::new(
        Arc::new(provider.clone()),
        rx.clone(),
        AxSnapshotCacheConfig {
            max_age: Duration::from_millis(1),
        },
    );

    // Start on Chrome
    tx.send_replace(Some(mk_window(
        "Google Chrome",
        chrome,
        "/Applications/Chrome",
        "Title A",
        pid_chrome,
    )));
    let s1 = cache.refresh_now().await;
    assert!(s1.browser.is_some());

    // Same app but title changes, and provider errors -> browser must clear
    provider
        .browser_map
        .lock()
        .unwrap()
        .insert((chrome.to_string(), pid_chrome), Err(()));
    tokio::time::sleep(Duration::from_millis(2)).await;

    tx.send_replace(Some(mk_window(
        "Google Chrome",
        chrome,
        "/Applications/Chrome",
        "Different Title",
        pid_chrome,
    )));
    let s2 = cache.refresh_now().await;
    assert!(
        s2.browser.is_none(),
        "browser should clear when title changed and provider errored"
    );

    // Switch to Xcode (non-browser) -> browser must be None, xcode should be populated
    tx.send_replace(Some(mk_window(
        "Xcode",
        xcode,
        "/Applications/Xcode",
        "Starter.playground",
        pid_xcode,
    )));
    let s3 = cache.refresh_now().await;
    assert!(s3.browser.is_none());
    assert_eq!(
        s3.xcode.as_ref().unwrap().project_name.as_deref(),
        Some("Starter")
    );
}

#[tokio::test]
async fn none_window_yields_none_app() {
    let (_tx, rx) = watch::channel::<Option<Window>>(None);
    let provider = MockProvider::default();

    let cache = AxSnapshotCache::new(
        Arc::new(provider),
        rx.clone(),
        AxSnapshotCacheConfig {
            max_age: Duration::from_secs(5),
        },
    );

    let s = cache.refresh_now().await;
    assert!(s.app.is_none());
}
