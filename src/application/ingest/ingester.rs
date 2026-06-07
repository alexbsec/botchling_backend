use crate::domain::event::BotchlingEvent;
use crate::infrastructure::mongo::{Database as MongoDatabase, EventDataRepository};
use crate::error::Error;
use tokio::sync::mpsc::Sender;

pub struct Ingester {
    event_repo: EventDataRepository,
    tx: Sender<BotchlingEvent>,
}

impl Ingester {
    pub fn new(db: &MongoDatabase, tx: Sender<BotchlingEvent>) -> Self {
        let event_collection = db.collection("events");
        let event_repo = EventDataRepository::new(event_collection);
        Self { event_repo, tx }
    }

    pub async fn ingest(&self, event: BotchlingEvent) -> Result<(), Error> {
        self.event_repo.insert(event.clone()).await.map_err(|e| Error {
            message: format!("Failed to ingest event: {}", e.message),
        })?;
        let _ = self.tx.send(event).await;
        Ok(())
    }
}
