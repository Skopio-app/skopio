use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::{
    monitored_app::MonitoredApp,
    trackers::ax::{provider::AxProvider, types::AxSnapshot},
};

#[derive(Clone, Copy, Debug)]
pub struct AxSnapshotCacheConfig {
    pub max_age: Duration,
}

pub struct AxSnapshotCache<P: AxProvider> {
    provider: Arc<P>,
    cfg: AxSnapshotCacheConfig,
    inner: RwLock<CacheInner>,
}

#[derive(Default)]
struct CacheInner {
    last: Option<AxSnapshot>,
    last_at: Option<Instant>,
}

impl<P: AxProvider> AxSnapshotCache<P> {
    pub fn new(provider: Arc<P>, cfg: AxSnapshotCacheConfig) -> Self {
        Self {
            provider,
            cfg,
            inner: RwLock::new(CacheInner::default()),
        }
    }

    /// Get (or refresh) a snapshot; throttled by `max_age`.
    pub async fn snapshot(&self) -> AxSnapshot {
        {
            let inner = self.inner.read().await;
            if let (Some(snap), Some(at)) = (&inner.last, inner.last_at) {
                if at.elapsed() <= self.cfg.max_age {
                    return snap.clone();
                }
            }
        }
        let snap = self.refresh_now().await;
        snap
    }

    /// Force refresh
    pub async fn refresh_now(&self) -> AxSnapshot {
        let mut out = AxSnapshot::default();

        let app = self.provider.frontmost_app().ok();

        if let Some(ref a) = app {
            out.window_title = self.provider.focused_window_title(a.pid).ok();

            match self.provider.browser_info(&a.bundle_id, a.pid) {
                Ok(bi) => {
                    out.browser = Some(bi);
                }
                Err(_) => {
                    let inner = self.inner.read().await;
                    if let Some(ref last_snap) = inner.last {
                        out.browser = last_snap.browser.clone();
                    }
                }
            }

            if a.bundle_id
                .parse::<MonitoredApp>()
                .unwrap_or(MonitoredApp::Unknown)
                == MonitoredApp::Xcode
            {
                if let Ok(xi) = self.provider.xcode_info(a.pid) {
                    out.xcode = Some(xi);
                }
            }
        }
        out.app = app;

        let mut inner = self.inner.write().await;
        inner.last = Some(out.clone());
        inner.last_at = Some(Instant::now());
        out
    }
}
