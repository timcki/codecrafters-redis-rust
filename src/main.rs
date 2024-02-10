mod command;
mod database;
mod resp;

use std::sync::Arc;

use anyhow::Context;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .context("binding to socket")?;

    let db = Arc::new(database::Database::new("test"));

    println!("accepting connections");
    loop {
        let (mut stream, _addr) = listener.accept().await.context("accepting connection")?;
        let db_clone = db.clone();
        tokio::spawn(async move { db_clone.handle_connection(&mut stream).await.unwrap() });
    }
}
