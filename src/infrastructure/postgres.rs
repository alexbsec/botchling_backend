use postgres::{Client, Config as PgConfig, NoTls};

use crate::infrastructure::config::Config;
use crate::error::Error;

pub struct Database {
    pub client: Client,
}

impl Database {
    pub fn new(cfg: &Config) -> Result<Self, Error> {
        let client = PgConfig::new()
            .user(&cfg.postgres_user)
            .password(&cfg.postgres_password)
            .host("127.0.0.1")
            .port(cfg.postgres_port)
            .dbname(&cfg.postgres_db)
            .connect(NoTls)
            .map_err(|e| Error {
                message: format!("Failed to connect to database: {}", e),
            })?;

        Ok(Self { client })
    }
}
