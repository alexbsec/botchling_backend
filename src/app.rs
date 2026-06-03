use crate::log_info;

pub struct App;

impl App {
    pub fn new() -> Self {
        App
    }

    pub fn run(&self) {
        log_info!("App is running");
    }
}
