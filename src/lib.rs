use std::io::{Read, Write};
use std::net;

#[derive(Debug)]
pub enum Frame {
    Put(String, String),
    Get(String),
    Delete(String),
}

#[derive(Debug)]
pub enum ReadFrameError {
    IoError(std::io::Error),
    InvalidUtf8(std::string::FromUtf8Error),
    InvalidCommand,
}

impl std::fmt::Display for ReadFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadFrameError::IoError(err) => write!(f, "IO error: {}", err),
            ReadFrameError::InvalidUtf8(err) => write!(f, "Invalid UTF-8: {}", err),
            ReadFrameError::InvalidCommand => write!(f, "Invalid command"),
        }
    }
}

impl std::error::Error for ReadFrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ReadFrameError::IoError(err) => Some(err),
            ReadFrameError::InvalidUtf8(err) => Some(err),
            ReadFrameError::InvalidCommand => None,
        }
    }
}

impl From<std::io::Error> for ReadFrameError {
    fn from(err: std::io::Error) -> Self {
        ReadFrameError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for ReadFrameError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ReadFrameError::InvalidUtf8(err)
    }
}

pub fn read_string_from_stream(stream: &mut net::TcpStream) -> Result<String, ReadFrameError> {
    let mut buffer = [0u8; 1];
    stream.read_exact(&mut buffer)?;
    let mut buffer: Vec<u8> = vec![0u8; buffer[0] as usize];
    stream.read_exact(&mut buffer)?;
    let s = String::from_utf8(buffer)?;
    Ok(s)
}

pub fn read_frame(mut stream: &mut net::TcpStream) -> Result<Frame, ReadFrameError> {
    let mut buffer = [0; 1];
    stream.read_exact(&mut buffer)?;
    match buffer[0] as char {
        '=' => {
            let key = read_string_from_stream(&mut stream)?;
            Ok(Frame::Get(key))
        }
        '+' => {
            let key = read_string_from_stream(&mut stream)?;
            let value = read_string_from_stream(&mut stream)?;
            Ok(Frame::Put(key, value))
        }
        '-' => {
            let key = read_string_from_stream(&mut stream)?;
            Ok(Frame::Delete(key))
        }
        _ => return Err(ReadFrameError::InvalidCommand),
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

pub fn write_frame(mut stream: &mut net::TcpStream, frame: &Frame) -> std::io::Result<()> {
    match frame {
        Frame::Get(key) => {
            stream.write_all(&[b'='])?;
            write_len_string(&mut stream, &key)?;
        }
        Frame::Put(key, value) => {
            stream.write_all(&[b'+'])?;
            write_len_string(&mut stream, &key)?;
            write_len_string(&mut stream, &value)?;
        }
        Frame::Delete(key) => {
            stream.write_all(&[b'-'])?;
            write_len_string(&mut stream, &key)?;
        }
    }
    Ok(())
}