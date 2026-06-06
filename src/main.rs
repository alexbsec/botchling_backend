mod error;
mod domain;
mod application;
mod infrastructure;

use infrastructure::config::Config;
use application::Application;

#[tokio::main]
async fn main() {
    infrastructure::logger::init(infrastructure::logger::LogLevel::Fatal);
    let cfg = match Config::load_from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            log_fatal!("Failed to load configuration: {}", e.message);
            std::process::exit(1);
        }
    };
    infrastructure::logger::set_log_level(&cfg.logger_level);

    let app = match Application::new(cfg).await {
        Ok(app) => app,
        Err(e) => {
            log_fatal!("Failed to initialize application: {}", e.message);
            std::process::exit(1);
        }
    };


    app.run().await.unwrap_or_else(|e| {
        log_fatal!("Application error: {}", e.message);
        std::process::exit(1);
    });
}
