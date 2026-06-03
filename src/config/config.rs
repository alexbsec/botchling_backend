pub struct Configuration {
    pub app_port: u16,
}

impl Configuration {
    pub fn new(port: u16) -> Self {
        Self {
            app_port: port,
        }
    }
}


