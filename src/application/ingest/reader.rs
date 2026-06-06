use crate::domain::event::BotchlingEvent;
use crate::error::Error;
use tokio::net::UnixDatagram;

use std::path::Path;

pub struct SocketReader {
    sock: UnixDatagram,
}

impl SocketReader {
    pub fn new(path: &str) -> Result<Self, Error> {
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error {
                message: format!("Failed to create socket directory: {}", e),
            })?;
        }
        let _ = std::fs::remove_file(path);
        let sock = UnixDatagram::bind(path).map_err(|e| Error {
            message: format!("Failed to bind to socket: {}", e),
        })?;

        Ok(Self { sock })
    }

    pub async fn read(&self) -> Result<BotchlingEvent, Error> {
        let mut buf = [0u8; 289];
        let n = self.sock.recv(&mut buf).await.map_err(|e| Error {
            message: format!("Failed to read from socket: {}", e),
        })?;

        BotchlingEvent::try_from(&buf[..n]).map_err(|e| Error {
            message: format!("Failed to parse event: {}", e.message),
        })
    }
}
