mod app;
mod logger;
mod config;

use config::Config;

fn main() {
    logger::init(logger::LogLevel::Debug);
    let cfg = Config::load_from_env();
    log_info!("Starting {}...", cfg.app_name);
}
