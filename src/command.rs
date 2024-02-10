use crate::resp::Resp;
use anyhow::{anyhow, bail, Context};
use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub enum Command {
    PING,
    ECHO(Resp),
    SET(String, Resp),
    GET(String, Resp),
}

impl Command {
    pub fn parse(msg: Resp) -> anyhow::Result<Command> {
        match msg {
            Resp::SimpleString(val) => match String::from_utf8_lossy(&val).as_ref() {
                "PING" | "ping" => Ok(Command::PING),
                cmd => bail!("unkown command {cmd}"),
            },
            Resp::Array(Some(arr)) => {
                let mut arr = arr.into_iter();
                match arr.next().ok_or(anyhow!("invalid message"))? {
                    Resp::BulkString(Some(cmd)) => match String::from_utf8_lossy(&cmd).as_ref() {
                        "ECHO" | "echo" => Ok(Command::ECHO(
                            arr.next().ok_or(anyhow!("no parameter for echo"))?,
                        )),
                        "GET" | "get" => {
                            let key = arr.next().ok_or(anyhow!("no key for set"))?;
                            let key = if let Resp::BulkString(Some(key)) = key {
                                String::from_utf8(key)?
                            } else {
                                bail!("key for SET is not a string")
                            };
                            Ok(Command::GET(key, Resp::Null))
                        }
                        "SET" | "set" => {
                            let key = arr.next().ok_or(anyhow!("no key for set"))?;
                            let key = if let Resp::BulkString(Some(key)) = key {
                                String::from_utf8(key)?
                            } else {
                                bail!("key for SET is not a string")
                            };
                            Ok(Command::SET(
                                key,
                                arr.next().ok_or(anyhow!("no value for set"))?,
                            ))
                        }
                        "PING" | "ping" => Ok(Command::PING),
                        _ => bail!("invalid message"),
                    },
                    _ => bail!("invalid message"),
                }
            }
            _ => bail!("invalid message"),
        }
    }

    pub fn respond(self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let msg = match self {
            Command::PING => Resp::SimpleString(b"PONG".to_vec()).encode(),
            Command::ECHO(val) => val.encode(),
            Command::SET(_, _) => Resp::SimpleString(b"OK".to_vec()).encode(),
            Command::GET(_, val) => val.encode(),
            //_ => bail!("unknown response"),
        };
        stream.try_write(&msg).context("writing PONG")?;
        Ok(())
    }
}
