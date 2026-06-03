extern crate dotenv;

use dotenv::dotenv;
use std::env;

pub struct Config {
    pub app_name: String,
}

impl Config {
    pub fn load_from_env() -> Self {
        dotenv().ok();
        let app_name = env::var("APP_NAME").unwrap_or_else(|_| "MyApp".to_string());
        Self { app_name }
    }
}
