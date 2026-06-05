use mongodb::{ sync::Client, options::{ ClientOptions, ServerApi, ServerApiVersion } };

use crate::infrastructure::config::Config;
use crate::error::Error;
use url::form_urlencoded;

pub struct Database {
    pub client: Client,
}

impl Database {
    pub fn new(cfg: &Config) -> Result<Self, Error> {
        let encoded_user = form_urlencoded::byte_serialize(cfg.mongo_user.as_bytes()).collect::<String>();
        let encoded_password = form_urlencoded::byte_serialize(cfg.mongo_password.as_bytes()).collect::<String>();

        let uri = format!(
            "{}://{}:{}@{}:{}/{}",
            cfg.mongo_url_prefix,
            encoded_user,
            encoded_password,
            cfg.mongo_host,
            cfg.mongo_port,
            cfg.mongo_db
        );
            
        let mut client_options = match ClientOptions::parse(uri).run() {
            Ok(options) => options,
            Err(e) => return Err(Error { message: format!("Failed to parse MongoDB URI: {}", e) }),
        };

        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);

        let client = match Client::with_options(client_options) {
            Ok(client) => client,
            Err(e) => return Err(Error { message: format!("Failed to create MongoDB client: {}", e) }),
        };

        Ok(Self {
            client: client,
        })
    }
}
