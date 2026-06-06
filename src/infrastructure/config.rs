extern crate dotenv;

use crate::error::Error;
use dotenv::dotenv;
use std::env;

pub struct Config {
    pub app_name: String,

    pub postgres_db: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,

    pub mongo_url_prefix: String,
    pub mongo_user: String,
    pub mongo_password: String,
    pub mongo_host: String,
    pub mongo_port: u16,
    pub mongo_db: String,

    pub socket_path: String,

    pub logger_level: String,
}

impl Config {
    pub fn load_from_env() -> Result<Self, Error> {
        dotenv().ok();
        let app_name = env::var("APP_NAME").unwrap_or_else(|_| "RustApp".to_string());
        let postgres_db = env::var("POSTGRES_DB").map_err(|_| Error {
            message: "POSTGRES_DB environment variable is missing".to_string(),
        })?;
        let postgres_user = env::var("POSTGRES_USER").map_err(|_| Error {
            message: "POSTGRES_USER environment variable is missing".to_string(),
        })?;
        let postgres_password = env::var("POSTGRES_PASSWORD").map_err(|_| Error {
            message: "POSTGRES_PASSWORD environment variable is missing".to_string(),
        })?;
        let postgres_port = env::var("POSTGRES_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse::<u16>()
            .map_err(|_| Error {
                message: "POSTGRES_PORT environment variable must be a valid number".to_string(),
            })?;
        let mongo_url_prefix =
            env::var("MONGO_URL_PREFIX").unwrap_or_else(|_| "mongodb".to_string());
        let mongo_user = env::var("MONGO_USER").map_err(|_| Error {
            message: "MONGO_USER environment variable is missing".to_string(),
        })?;
        let mongo_password = env::var("MONGO_PASSWORD").map_err(|_| Error {
            message: "MONGO_PASSWORD environment variable is missing".to_string(),
        })?;
        let mongo_host = env::var("MONGO_HOST").map_err(|_| Error {
            message: "MONGO_HOST environment variable is missing".to_string(),
        })?;
        let mongo_port = env::var("MONGO_PORT")
            .unwrap_or_else(|_| "27017".to_string())
            .parse::<u16>()
            .map_err(|_| Error {
                message: "MONGO_PORT environment variable must be a valid number".to_string(),
            })?;
        let mongo_db = env::var("MONGO_DB").map_err(|_| Error {
            message: "MONGO_DB environment variable is missing".to_string(),
        })?;

        let socket_path =
            env::var("SOCKET_PATH").unwrap_or_else(|_| "/var/run/botchling/botchling.sock".to_string());

        let logger_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        Ok(Self {
            app_name,
            postgres_db,
            postgres_port,
            postgres_user,
            postgres_password,
            mongo_url_prefix,
            mongo_user,
            mongo_password,
            mongo_host,
            mongo_port,
            mongo_db,
            socket_path,
            logger_level,
        })
    }
}
