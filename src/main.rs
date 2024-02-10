use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::Context;
use nom::error::{Error, ErrorKind, ParseError};
use nom::{
    bytes::complete::take,
    character::complete::{crlf, not_line_ending},
    multi::count,
    sequence::terminated,
    IResult,
};

#[derive(Debug, Default)]
enum Message {
    #[default]
    Null,

    SimpleString(Vec<u8>), // +<data>\r\n
    Error(Vec<u8>),        // -<data>\r\n

    Integer(i64), // :[optional sign]<data>\r\n

    BulkString(Option<Vec<u8>>), // $<len>\r\n<data>r\n$

    Array(Option<Vec<Message>>),
}

fn parse_message(buffer: &[u8]) -> IResult<&[u8], Message> {
    let (buffer, val) = take(1u16)(buffer)?;
    match val[0] {
        b'+' => simple_string(buffer),
        b'-' => error(buffer),
        b':' => integer(buffer),
        b'$' => bulk_string(buffer),
        b'*' => array(buffer),
        _ => Err(nom::Err::Error(Error::from_error_kind(
            buffer,
            ErrorKind::Tag,
        ))),
    }
}

fn simple_string(input: &[u8]) -> IResult<&[u8], Message> {
    let (input, val) = terminated(not_line_ending, crlf)(input)?;
    Ok((input, Message::SimpleString(val.to_vec())))
}

fn integer(input: &[u8]) -> IResult<&[u8], Message> {
    let (input, val) = terminated(not_line_ending, crlf)(input)?;
    Ok((
        input,
        Message::Integer(String::from_utf8_lossy(val).parse::<i64>().unwrap()),
    ))
}

fn error(input: &[u8]) -> IResult<&[u8], Message> {
    let (input, val) = terminated(not_line_ending, crlf)(input)?;
    Ok((input, Message::Error(val.to_vec())))
}

fn bulk_string(input: &[u8]) -> IResult<&[u8], Message> {
    let (input, len) = length(input)?;
    if len == 0 {
        return Ok((input, Message::BulkString(None)));
    }
    let (input, val) = terminated(take(len), crlf)(input)?;

    Ok((input, Message::BulkString(Some(val.to_vec()))))
}

fn length(input: &[u8]) -> IResult<&[u8], usize> {
    let (input, len) = terminated(not_line_ending, crlf)(input)?;
    Ok((input, String::from_utf8_lossy(len).parse().unwrap()))
}

fn array(input: &[u8]) -> IResult<&[u8], Message> {
    let (input, len) = length(input)?;
    if len == 0 {
        return Ok((input, Message::Array(None)));
    }
    let (input, res) = count(parse_message, len)(input)?;
    Ok((input, Message::Array(Some(res))))
}

fn read_bytes(stream: &mut TcpStream) -> anyhow::Result<[u8; 1024]> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    println!("Received {} bytes from the client.", bytes_read);

    Ok(buffer)
}

fn handle_message(msg: &Message, stream: &mut TcpStream) -> anyhow::Result<()> {
    let response = "+PONG\r\n";
    match msg {
        Message::SimpleString(val) => println!(
            "Received simple string: {}",
            String::from_utf8_lossy(val).as_ref()
        ),
        Message::Integer(num) => println!("Received integer: {}", num),
        Message::BulkString(Some(val)) => println!(
            "Received bulk string: {}",
            String::from_utf8_lossy(&val).as_ref()
        ),
        Message::BulkString(None) => println!("Received empty bulk string"),
        Message::Array(Some(vec)) => {
            for val in vec {
                match val {
                    Message::SimpleString(s) => todo!(),
                    Message::Integer(_) => todo!(),
                    Message::BulkString(s) => {
                        if let Some(s) = s {
                            String::from_utf8_lossy(s)
                                .as_ref()
                                .split("\\n")
                                .for_each(|cmd| {
                                    println!("got {cmd}");
                                    stream.write_all(response.as_bytes()).unwrap();
                                    println!("sent pong")
                                })
                        }
                    }
                    Message::Array(_) => todo!(),
                    Message::Error(_) => todo!(),
                    Message::Null => {}
                }
            }
        }
        Message::Array(None) => println!("Received empty array"),
        Message::Error(val) => todo!(),
        Message::Null => {} //println("Received error: {}", val),
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    println!("accepting connections");

    for stream in listener.incoming() {
        let mut stream = stream.context("accepting connection")?;

        println!("accepted new connection");

        let buffer = read_bytes(&mut stream)?;
        let (_, message) = parse_message(&buffer).unwrap();

        handle_message(&message, &mut stream)?;
    }
    Ok(())
}
