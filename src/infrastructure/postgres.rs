use tokio_postgres::{Client, NoTls};

use crate::error::Error;
use crate::infrastructure::config::Config;

pub struct Database {
    client: Client,
}

impl Database {
    pub async fn new(cfg: &Config) -> Result<Self, Error> {
        let result = tokio_postgres::Config::new()
            .user(&cfg.postgres_user)
            .password(&cfg.postgres_password)
            .host("localhost")
            .port(cfg.postgres_port)
            .dbname(&cfg.postgres_db)
            .connect(NoTls)
            .await
            .map_err(|e| Error {
                message: format!("Failed to connect to Postgres: {}", e),
            });

        let (client, connection) = match result {
            Ok((client, connection)) => (client, connection),
            Err(e) => return Err(e),
        };


        tokio::spawn(async move {
            if let Err(e) = connection.await {
                crate::log_error!("Postgres connection error: {}", e);
            }
        });

        Ok(Self { client })
    }
}
