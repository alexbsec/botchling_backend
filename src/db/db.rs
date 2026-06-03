use postgres::{Client, NoTls};

pub struct Database {
    host: String,
    port: u16,
    database_name: String,
    username: String,
    password: String,
    client: Client,
}

impl Database {
    pub fn new() -> Result<Self, Error> {
        Ok(Self{})
    }
}
