use std::sync::Arc;

use async_trait::async_trait;
use db::{
    DBContext,
    desktop::{afk_events::AFKEvent, events::Event},
    error::DBError,
};

#[async_trait]
pub trait TrackingService: Send + Sync {
    async fn insert_event(&self, event: &Event) -> Result<(), DBError>;
    async fn insert_afk(&self, afk: &AFKEvent) -> Result<(), DBError>;
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
    async fn insert_event(&self, event: &Event) -> Result<(), DBError> {
        event.insert(&self.db).await
    }

    async fn insert_afk(&self, afk: &AFKEvent) -> Result<(), DBError> {
        afk.insert(&self.db).await
    }
}
