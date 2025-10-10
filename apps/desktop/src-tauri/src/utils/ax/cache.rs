use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::{watch, RwLock};

use crate::{
    monitored_app::MonitoredApp,
    trackers::window_tracker::Window,
    utils::ax::{
        provider::AxProvider,
        types::{ActiveApp, AxSnapshot},
    },
};

#[derive(Clone, Copy, Debug)]
pub struct AxSnapshotCacheConfig {
    pub max_age: Duration,
}

pub struct AxSnapshotCache<P: AxProvider> {
    provider: Arc<P>,
    windows_rx: watch::Receiver<Option<Window>>,
    cfg: AxSnapshotCacheConfig,
    inner: RwLock<CacheInner>,
}

#[derive(Default)]
struct CacheInner {
    last: Option<AxSnapshot>,
    last_at: Option<Instant>,
}

impl<P: AxProvider> AxSnapshotCache<P> {
    pub fn new(
        provider: Arc<P>,
        windows_rx: watch::Receiver<Option<Window>>,
        cfg: AxSnapshotCacheConfig,
    ) -> Self {
        Self {
            provider,
            windows_rx,
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
        let current_window = self.windows_rx.borrow().clone();

        if let Some(ref w) = current_window {
            let app: ActiveApp = w.into();
            out.app = Some(app.clone());
            out.window_title = {
                let title = w.title.as_ref();
                if !title.is_empty() && title != "unknown" {
                    Some(title.to_string())
                } else {
                    None
                }
            };

            let app_changed = prev
                .as_ref()
                .and_then(|p| p.app.as_ref())
                .map_or(true, |pa| {
                    pa.bundle_id != app.bundle_id || pa.pid != app.pid
                });

            let same_title = match (
                &prev.as_ref().and_then(|p| p.window_title.clone()),
                &out.window_title,
            ) {
                (Some(pt), Some(nt)) => *pt == *nt,
                _ => false,
            };

            match self.provider.browser_info(&app.bundle_id, app.pid) {
                Ok(bi) => {
                    out.browser = Some(bi);
                }
                Err(_) => {
                    if !app_changed && same_title {
                        out.browser = prev.and_then(|p| p.browser);
                    } else {
                        out.browser = None;
                    }
                }
            }

            if app
                .bundle_id
                .parse::<MonitoredApp>()
                .unwrap_or(MonitoredApp::Unknown)
                == MonitoredApp::Xcode
            {
                if let Ok(xi) = self.provider.xcode_info(app.pid) {
                    out.xcode = Some(xi);
                }
            }
        } else {
            out.app = None;
        }

        let mut inner = self.inner.write().await;
        inner.last = Some(out.clone());
        inner.last_at = Some(Instant::now());
        out
    }
}
