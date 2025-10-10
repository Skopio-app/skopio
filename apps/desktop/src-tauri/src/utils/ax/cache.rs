use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::{
    monitored_app::MonitoredApp,
    utils::ax::{provider::AxProvider, types::AxSnapshot},
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

        let prev = {
            let i = self.inner.read().await;
            i.last.clone()
        };

        let app = self.provider.frontmost_app().ok();

        if let Some(ref a) = app {
            out.window_title = self.provider.focused_window_title(a.pid).ok();

            let app_changed = prev
                .as_ref()
                .and_then(|p| p.app.as_ref())
                .map_or(true, |pa| pa.bundle_id != a.bundle_id || pa.pid != a.pid);

            let same_window_title = match (
                &prev.as_ref().and_then(|p| p.window_title.clone()),
                &out.window_title,
            ) {
                (Some(pt), Some(nt)) => *pt == *nt,
                _ => false,
            };

            match self.provider.browser_info(&a.bundle_id, a.pid) {
                Ok(bi) => {
                    out.browser = Some(bi);
                }
                Err(_) => {
                    if !app_changed && same_window_title {
                        out.browser = prev.as_ref().and_then(|p| p.browser.clone());
                    } else {
                        out.browser = None;
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
