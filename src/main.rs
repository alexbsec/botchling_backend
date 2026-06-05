mod error;
mod domain;
mod application;
mod infrastructure;

use infrastructure::config::Config;
use infrastructure::postgres::Database as PostgresDatabase;
use infrastructure::mongo::Database as MongoDatabase;

fn main() {
    infrastructure::logger::init(infrastructure::logger::LogLevel::Fatal);
    let cfg = match Config::load_from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            log_fatal!("Failed to load configuration: {}", e.message);
            std::process::exit(1);
        }
    };
    infrastructure::logger::set_log_level(&cfg.logger_level);

    let _db = match PostgresDatabase::new(&cfg) {
        Ok(db) => db,
        Err(e) => {
            log_fatal!("Failed to connect to Postgres: {}", e.message);
            return;
        }
    };

    let _mongo_db = match MongoDatabase::new(&cfg) {
        Ok(db) => db,
        Err(e) => {
            log_fatal!("Failed to connect to MongoDB: {}", e.message);
            return;
        }
    };

    log_info!("Starting {}...", cfg.app_name);
}
