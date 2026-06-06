pub mod app;
pub mod ingest;

pub use ingest::reader::SocketReader;

use crate::error::Error;
use crate::infrastructure::config::Config;
use crate::infrastructure::mongo::Database as MongoDatabase;
use crate::{log_error, log_info};

pub struct Application {
    pub reader: ingest::reader::SocketReader,
    pub db: MongoDatabase,
    config: Config,
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

        Ok(Self {
            reader,
            db: mongo_db,
            config: cfg,
        })
    }

    pub async fn run(&self) -> Result<(), Error> {
        log_info!("Application is running...");
        loop {
            match self.reader.read().await {
                Ok(event) => {
                    log_info!("Received event: {:?}", event);
                    // Process event & save in db
                }
                Err(e) => {
                    log_error!("Error reading from socket: {}", e.message);
                }
            }
        }
    }
}
