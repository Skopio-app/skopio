use std::sync::Arc;

use async_trait::async_trait;
use db::{
    desktop::{afk_events::AFKEvent, events::Event, heartbeats::Heartbeat},
    DBContext,
};

#[async_trait]
pub trait TrackingService: Send + Sync {
    async fn insert_heartbeat(&self, heartbeat: Heartbeat) -> Result<(), anyhow::Error>;
    async fn insert_event(&self, event: Event) -> Result<(), anyhow::Error>;
    async fn insert_afk(&self, afk: AFKEvent) -> Result<(), anyhow::Error>;
}

pub struct DBService {
    db: Arc<DBContext>,
}

impl DBService {
    pub fn new(db: Arc<DBContext>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TrackingService for DBService {
    async fn insert_heartbeat(&self, heartbeat: Heartbeat) -> Result<(), anyhow::Error> {
        let result = heartbeat.insert(&self.db).await;
        result.map_err(Into::into)
    }

    async fn insert_event(&self, event: Event) -> Result<(), anyhow::Error> {
        let result = event.insert(&self.db).await;
        result.map_err(Into::into)
    }

    async fn insert_afk(&self, afk: AFKEvent) -> Result<(), anyhow::Error> {
        let result = afk.insert(&self.db).await;
        result.map_err(Into::into)
    }
}
