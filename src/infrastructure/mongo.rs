use mongodb::{
    Client, Collection, IndexModel,
    bson::{Document, doc, from_document, to_document},
    options::{ClientOptions, Credential, IndexOptions, ServerApi, ServerApiVersion},
};

use crate::infrastructure::config::Config;
use crate::{
    domain::{event::BotchlingEvent, session::Session},
    error::Error,
};
use futures::StreamExt;
use std::time::Duration;

pub struct Database {
    client: Client,
    db_name: String,
}

pub struct EventDataRepository {
    collection: Collection<Document>,
}

pub struct SessionRepository {
    collection: Collection<Document>,
}

impl Database {
    pub async fn new(cfg: &Config) -> Result<Self, Error> {
        let uri = format!(
            "{}://{}:{}/{}",
            cfg.mongo_url_prefix, cfg.mongo_host, cfg.mongo_port, cfg.mongo_db
        );

        let mut client_options = match ClientOptions::parse(uri).await {
            Ok(options) => options,
            Err(e) => {
                return Err(Error {
                    message: format!("Failed to parse MongoDB URI: {}", e),
                });
            }
        };

        client_options.credential = Some(
            Credential::builder()
                .username(cfg.mongo_user.clone())
                .password(cfg.mongo_password.clone())
                .build(),
        );

        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);

        let client = match Client::with_options(client_options) {
            Ok(client) => client,
            Err(e) => {
                return Err(Error {
                    message: format!("Failed to create MongoDB client: {}", e),
                });
            }
        };

        match client
            .database(&cfg.mongo_db)
            .run_command(doc! {"ping": 1})
            .await
        {
            Ok(_) => (),
            Err(e) => {
                return Err(Error {
                    message: format!("Failed to connect to MongoDB: {}", e),
                });
            }
        };

        Ok(Self {
            client: client,
            db_name: cfg.mongo_db.clone(),
        })
    }

    pub fn collection(&self, name: &str) -> Collection<Document> {
        self.client.database(&self.db_name).collection(name)
    }

    /// Creates (or leaves alone, if already present) a TTL index on
    /// `created_at` for the given collection, so documents older than
    /// `ttl_days` are auto-deleted by Mongo's background TTL monitor.
    /// Idempotent -- safe to call on every startup.
    pub async fn ensure_ttl_index(&self, collection_name: &str, ttl_days: u64) -> Result<(), Error> {
        let collection: Collection<Document> = self.collection(collection_name);
        let options = IndexOptions::builder()
            .expire_after(Duration::from_secs(ttl_days * 86_400))
            .build();
        let index = IndexModel::builder()
            .keys(doc! { "created_at": 1 })
            .options(options)
            .build();

        collection.create_index(index).await.map_err(|e| Error {
            message: format!(
                "Failed to create TTL index on '{}': {}",
                collection_name, e
            ),
        })?;

        Ok(())
    }
}

impl EventDataRepository {
    pub fn new(collection: Collection<Document>) -> Self {
        Self { collection }
    }

    // Ingest called
    pub async fn insert(&self, event: BotchlingEvent) -> Result<(), Error> {
        let mut doc = to_document(&event).map_err(|e| Error {
            message: format!("Failed to serialize event: {}", e),
        })?;
        doc.insert("created_at", mongodb::bson::DateTime::now());

        self.collection.insert_one(doc).await.map_err(|e| Error {
            message: format!("Failed to insert event into MongoDB: {}", e),
        })?;

        Ok(())
    }
}

impl SessionRepository {
    pub fn new(collection: Collection<Document>) -> Self {
        Self { collection }
    }

    pub async fn insert(&self, session: Session) -> Result<(), Error> {
        let mut doc = to_document(&session).map_err(|e| Error {
            message: format!("Failed to serialize session: {}", e),
        })?;
        doc.insert("created_at", mongodb::bson::DateTime::now());

        self.collection.insert_one(doc).await.map_err(|e| Error {
            message: format!("Failed to insert session into MongoDB: {}", e),
        })?;

        Ok(())
    }

    pub async fn find_by_char_id(&self, char_id: u32) -> Result<Vec<Session>, Error> {
        self.find_by(doc! { "char_id": char_id as i64 }).await
    }

    pub async fn find_by_account_id(&self, account_id: u32) -> Result<Vec<Session>, Error> {
        self.find_by(doc! { "account_id": account_id as i64 }).await
    }

    async fn find_by(&self, filter: Document) -> Result<Vec<Session>, Error> {
        let mut cursor = self.collection.find(filter).await.map_err(|e| Error {
            message: format!("Failed to query sessions from MongoDB: {}", e),
        })?;

        let mut sessions = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    let session: Session = from_document(doc).map_err(|e| Error {
                        message: format!("Failed to deserialize session: {}", e),
                    })?;
                    sessions.push(session);
                }
                Err(e) => {
                    return Err(Error {
                        message: format!("Error iterating MongoDB cursor: {}", e),
                    });
                }
            }
        }

        Ok(sessions)
    }
}
