use nom::error::{Error, ErrorKind, ParseError};
use nom::{
    bytes::complete::take,
    character::complete::{crlf, not_line_ending},
    multi::count,
    sequence::terminated,
    IResult,
};

#[derive(Debug, Default, Clone)]
pub enum Resp {
    #[default]
    Null,

    SimpleString(Vec<u8>), // +<data>\r\n
    Error(Vec<u8>),        // -<data>\r\n

    Integer(i64), // :[optional sign]<data>\r\n

    BulkString(Option<Vec<u8>>), // $<len>\r\n<data>r\n$

    Array(Option<Vec<Resp>>),
}

impl Resp {
    pub fn encode(&self) -> Vec<u8> {
        //let mut buf = Vec::new();
        match self {
            Resp::Null => Vec::new(),
            Resp::SimpleString(s) => {
                let mut buf = Vec::with_capacity(3 + s.len()); // `3` accounts for '+', '\r', '\n'
                buf.push(b'+');
                buf.extend(s);
                buf.extend_from_slice(b"\r\n");
                buf
            }
            Resp::Error(_) => todo!(),
            Resp::Integer(i) => {
                let mut buf = Vec::new();
                buf.push(b':');
                buf.extend_from_slice(i.to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                buf
            }

            Resp::BulkString(s) => match s {
                Some(s) => {
                    let mut buf = Vec::new();
                    let length = s.len();
                    buf.push(b'$');
                    buf.extend_from_slice(length.to_string().as_bytes());
                    buf.extend_from_slice(b"\r\n");
                    buf.extend(s);
                    buf.extend_from_slice(b"\r\n");
                    buf
                }
                None => Vec::from(b"$-1\r\n".as_ref()),
            },

            Resp::Array(_) => todo!(),
        }
    }
}

pub fn parse_message(buffer: &[u8]) -> IResult<&[u8], Resp> {
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

fn simple_string(input: &[u8]) -> IResult<&[u8], Resp> {
    let (input, val) = terminated(not_line_ending, crlf)(input)?;
    Ok((input, Resp::SimpleString(val.to_vec())))
}

fn integer(input: &[u8]) -> IResult<&[u8], Resp> {
    let (input, val) = terminated(not_line_ending, crlf)(input)?;
    Ok((
        input,
        Resp::Integer(String::from_utf8_lossy(val).parse::<i64>().unwrap()),
    ))
}

fn error(input: &[u8]) -> IResult<&[u8], Resp> {
    let (input, val) = terminated(not_line_ending, crlf)(input)?;
    Ok((input, Resp::Error(val.to_vec())))
}

fn bulk_string(input: &[u8]) -> IResult<&[u8], Resp> {
    let (input, len) = length(input)?;
    if len == 0 {
        return Ok((input, Resp::BulkString(None)));
    }
    let (input, val) = terminated(take(len), crlf)(input)?;

    Ok((input, Resp::BulkString(Some(val.to_vec()))))
}

fn length(input: &[u8]) -> IResult<&[u8], usize> {
    let (input, len) = terminated(not_line_ending, crlf)(input)?;
    Ok((input, String::from_utf8_lossy(len).parse().unwrap()))
}

fn array(input: &[u8]) -> IResult<&[u8], Resp> {
    let (input, len) = length(input)?;
    if len == 0 {
        return Ok((input, Resp::Array(None)));
    }
    let (input, res) = count(parse_message, len)(input)?;
    Ok((input, Resp::Array(Some(res))))
}
