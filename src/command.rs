use crate::resp::Resp;
use anyhow::{anyhow, bail, Context};
use tokio::net::TcpStream;

pub enum Command {
    PING,
    ECHO(Resp),
}

impl Command {
    pub fn parse(msg: &Resp) -> anyhow::Result<Command> {
        match msg {
            Resp::SimpleString(val) => match String::from_utf8_lossy(val).as_ref() {
                "PING" | "ping" => Ok(Command::PING),
                cmd => bail!("unkown command {cmd}"),
            },
            Resp::Array(Some(arr)) => match arr.first().ok_or(anyhow!("invalid message"))? {
                Resp::BulkString(Some(cmd)) => match String::from_utf8_lossy(cmd).as_ref() {
                    "ECHO" | "echo" => Ok(Command::ECHO(
                        arr.last().ok_or(anyhow!("no parameter for echo"))?.clone(),
                    )),
                    "PING" | "ping" => Ok(Command::PING),
                    _ => bail!("invalid message"),
                },
                _ => bail!("invalid message"),
            },
            _ => bail!("invalid message"),
        }
    }

    pub fn respond(self, stream: &mut TcpStream) -> anyhow::Result<()> {
        match self {
            Command::PING => {
                stream.try_write(b"+PONG\r\n").context("writing PONG")?;
                Ok(())
            }
            Command::ECHO(val) => {
                stream.try_write(&val.encode()).context("echoing val")?;
                Ok(())
            }
        }
    }
}

/*
pub fn parse(msg: &Resp, stream: &mut TcpStream) -> anyhow::Result<()> {
    let response = "+PONG\r\n";
    match msg {
        Resp::SimpleString(val) => println!(
            "Received simple string: {}",
            String::from_utf8_lossy(val).as_ref()
        ),
        Resp::Integer(num) => println!("Received integer: {}", num),
        Resp::BulkString(Some(val)) => println!(
            "Received bulk string: {}",
            String::from_utf8_lossy(&val).as_ref()
        ),
        Resp::BulkString(None) => println!("Received empty bulk string"),
        Resp::Array(Some(vec)) => {
            for val in vec {
                match val {
                    Resp::SimpleString(s) => todo!(),
                    Resp::Integer(_) => todo!(),
                    Resp::BulkString(s) => {
                        if let Some(s) = s {
                            String::from_utf8_lossy(s)
                                .as_ref()
                                .split("\\n")
                                .for_each(|cmd| {
                                    println!("got {cmd}");
                                    stream.try_write(response.as_bytes()).unwrap();
                                    println!("sent pong")
                                })
                        }
                    }
                    Resp::Array(_) => todo!(),
                    Resp::Error(_) => todo!(),
                    Resp::Null => {}
                }
            }
        }
        Resp::Array(None) => println!("Received empty array"),
        Resp::Error(val) => todo!(),
        Resp::Null => {} //println("Received error: {}", val),
    }

    Ok(())
}
*/
