use std::collections::HashMap;

use crate::command::Command;
use crate::resp::{parse_message, Resp};
use tokio::sync::Mutex;
use tokio::{io::AsyncReadExt, net::TcpStream};
pub struct Database {
    name: String,
    elements: Mutex<HashMap<String, Resp>>,
}

async fn read_bytes(stream: &mut TcpStream) -> anyhow::Result<([u8; 1024], usize)> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).await?;

    Ok((buffer, bytes_read))
}

impl Database {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            elements: Mutex::new(HashMap::new()),
        }
    }

    pub async fn set(&self, key: String, value: Resp) {
        self.elements.lock().await.insert(key, value);
    }

    pub async fn get(&self, key: &str) -> Option<Resp> {
        let map = self.elements.lock().await;
        let el = map.get(key)?;
        Some(el.clone())
    }

    pub async fn handle_connection(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        loop {
            let (buffer, read) = read_bytes(stream).await?;
            if read == 0 {
                return Ok(());
            }

            let (_, message) = parse_message(&buffer).unwrap_or_default();
            let cmd = Command::parse(message)?;
            println!("parsed command: {:?}", cmd);
            let cmd = match cmd.clone() {
                Command::SET(key, value) => {
                    self.set(key, value).await;
                    cmd
                }
                Command::GET(key, _) => {
                    if let Some(value) = self.get(&key).await {
                        Command::GET(key, value)
                    } else {
                        Command::GET(key, Resp::Null)
                    }
                }
                _ => cmd,
            };
            cmd.respond(stream)?;
        }
    }
}
