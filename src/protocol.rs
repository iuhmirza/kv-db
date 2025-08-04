use std::io::{Read, Write};
use std::net;

#[derive(Debug)]
pub enum Request {
    Put(String, String),
    Get(String),
    Delete(String),
}

#[derive(Debug)]
pub enum ReadRequestError {
    IoError(std::io::Error),
    InvalidUtf8(std::string::FromUtf8Error),
    InvalidCommand,
}

impl std::fmt::Display for ReadRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadRequestError::IoError(err) => write!(f, "IO error: {}", err),
            ReadRequestError::InvalidUtf8(err) => write!(f, "Invalid UTF-8: {}", err),
            ReadRequestError::InvalidCommand => write!(f, "Invalid command"),
        }
    }
}

impl std::error::Error for ReadRequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ReadRequestError::IoError(err) => Some(err),
            ReadRequestError::InvalidUtf8(err) => Some(err),
            ReadRequestError::InvalidCommand => None,
        }
    }
}

impl From<std::io::Error> for ReadRequestError {
    fn from(err: std::io::Error) -> Self {
        ReadRequestError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for ReadRequestError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ReadRequestError::InvalidUtf8(err)
    }
}

pub fn read_string_from_stream(stream: &mut net::TcpStream) -> Result<String, ReadRequestError> {
    let mut buffer = [0u8; 1];
    stream.read_exact(&mut buffer)?;
    let mut buffer: Vec<u8> = vec![0u8; buffer[0] as usize];
    stream.read_exact(&mut buffer)?;
    let s = String::from_utf8(buffer)?;
    Ok(s)
}

pub fn read_request(mut stream: &mut net::TcpStream) -> Result<Request, ReadRequestError> {
    let mut buffer = [0; 1];
    stream.read_exact(&mut buffer)?;
    match buffer[0] as char {
        '=' => {
            let key = read_string_from_stream(&mut stream)?;
            Ok(Request::Get(key))
        }
        '+' => {
            let key = read_string_from_stream(&mut stream)?;
            let value = read_string_from_stream(&mut stream)?;
            Ok(Request::Put(key, value))
        }
        '-' => {
            let key = read_string_from_stream(&mut stream)?;
            Ok(Request::Delete(key))
        }
        _ => return Err(ReadRequestError::InvalidCommand),
    }
}

pub fn write_response(stream: &mut net::TcpStream, buffer: &[u8]) -> std::io::Result<()> {
    stream.write(&[buffer.len() as u8])?;
    stream.write_all(buffer)?;
    Ok(())
}

fn write_len_string(stream: &mut net::TcpStream, s: &str) -> std::io::Result<()> {
    let s = s.as_bytes();
    // consier case where string is greater than 255 bytes?
    stream.write_all(&[s.len() as u8])?;
    stream.write_all(s)?;
    Ok(())
}

pub fn write_request(mut stream: &mut net::TcpStream, request: &Request) -> std::io::Result<()> {
    match request {
        Request::Get(key) => {
            stream.write_all(&[b'='])?;
            write_len_string(&mut stream, &key)?;
        }
        Request::Put(key, value) => {
            stream.write_all(&[b'+'])?;
            write_len_string(&mut stream, &key)?;
            write_len_string(&mut stream, &value)?;
        }
        Request::Delete(key) => {
            stream.write_all(&[b'-'])?;
            write_len_string(&mut stream, &key)?;
        }
    }
    Ok(())
}