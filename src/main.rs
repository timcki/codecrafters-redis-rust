mod command;
mod resp;

use anyhow::Context;
use command::Command;
use resp::{parse_message, Resp};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .context("binding to socket")?;

    println!("accepting connections");
    loop {
        let (mut stream, _addr) = listener.accept().await.context("accepting connection")?;
        tokio::spawn(async move { process_connection(&mut stream).await.unwrap() });
    }
}

async fn read_bytes(stream: &mut TcpStream) -> anyhow::Result<([u8; 1024], usize)> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).await?;

    Ok((buffer, bytes_read))
}

async fn process_connection(stream: &mut TcpStream) -> anyhow::Result<()> {
    loop {
        let (buffer, read) = read_bytes(stream).await?;
        if read == 0 {
            return Ok(());
        }

        let (_, message) = parse_message(&buffer).unwrap_or_default();
        println!("parsed message: {:?}", message);
        let cmd = Command::parse(&message)?;
        cmd.respond(stream)?;
    }
}
/*

println!("accepting connections");

for stream in listener.incoming() {
    let mut stream = stream.context("accepting connection")?;

    println!("accepted new connection");

    loop {
        let (buffer, read) = read_bytes(&mut stream)?;
        if read == 0 {
            continue;
        }
        let (_, message) = parse_message(&buffer).unwrap_or_default();
        println!("parsed message: {:?}", message);

        handle_message(&message, &mut stream)?;
    }
}
Ok(())
*/

// async fn process_connection(stream: &mut TcpStream) -> anyhow::Result<()> {}
