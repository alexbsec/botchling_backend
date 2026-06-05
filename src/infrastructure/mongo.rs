use mongodb::{
    Client,
    bson::doc,
    options::{ClientOptions, Credential, ServerApi, ServerApiVersion},
};

use crate::error::Error;
use crate::infrastructure::config::Config;

pub struct Database {
    client: Client,
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

        Ok(Self { client: client })
    }
}
