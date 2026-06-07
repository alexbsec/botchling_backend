pub mod ingest;
pub mod worker;

pub use ingest::ingester::Ingester;
pub use ingest::reader::SocketReader;
pub use worker::Worker;

use crate::domain::event::BotchlingEvent;
use crate::error::Error;
use crate::infrastructure::config::Config;
use crate::infrastructure::mongo::Database as MongoDatabase;
use crate::{log_error};

pub struct Application {
    pub reader: ingest::reader::SocketReader,
    pub ingester: ingest::ingester::Ingester,
    pub worker: worker::Worker,
}

impl Application {
    pub async fn new(cfg: Config) -> Result<Self, Error> {
        let reader = match SocketReader::new(&cfg.socket_path) {
            Ok(r) => r,
            Err(e) => {
                return Err(Error {
                    message: format!("Failed to initialize socket reader: {}", e.message),
                });
            }
        };

        let mongo_db = match MongoDatabase::new(&cfg).await {
            Ok(db) => db,
            Err(e) => {
                return Err(Error {
                    message: format!("Failed to connect to MongoDB: {}", e.message),
                });
            }
        };

        let (tx, rx) = tokio::sync::mpsc::channel::<BotchlingEvent>(1024);
        let ingester = Ingester::new(&mongo_db, tx);
        let worker = Worker::new(&mongo_db, rx);
        Ok(Self {
            reader,
            ingester,
            worker,
        })
    }

    pub async fn run(self) -> Result<(), Error> {
        let Application {
            reader,
            ingester,
            mut worker,
        } = self;

        let ingest_handle = tokio::spawn(async move {
            loop {
                match reader.read().await {
                    Ok(event) => {
                        let _ = ingester.ingest(event).await;
                    }
                    Err(e) => {
                        log_error!("Error reading from socket: {}", e.message);
                    }
                }
            }
        });

        let worker_handle = tokio::spawn(async move {
            worker.run().await;
        });

        tokio::try_join!(ingest_handle, worker_handle);
        Ok(())
    }
}
