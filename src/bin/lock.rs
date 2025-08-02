use std::collections::HashMap;
use std::io::{Read, Write};
use std::net;
use std::sync::{Arc, RwLock, mpsc};
use std::thread;


#[derive(Debug)]
enum ReadFrameError {
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

fn read_string_from_stream(stream: &mut net::TcpStream) -> Result<String, ReadFrameError> {
    let mut buffer = [0u8; 1];
    stream.read_exact(&mut buffer)?;
    let mut buffer: Vec<u8> = vec![0u8; buffer[0] as usize];
    stream.read_exact(&mut buffer)?;
    let s = String::from_utf8(buffer)?;
    Ok(s)
}

enum Frame {
    Put(String, String),
    Get(String),
    Delete(String),
}

fn read_frame(mut stream: &mut net::TcpStream) -> Result<Frame, ReadFrameError> {
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

fn write_response(stream: &mut net::TcpStream, buffer: &[u8]) -> () {
    stream.write(&[buffer.len() as u8]);
    stream.write_all(buffer);
}

type KVMap = Arc<RwLock<HashMap<String, String>>>;

fn handle_client(mut stream: net::TcpStream, map: KVMap) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let command = read_frame(&mut stream)?;
        match command {
            Frame::Get(key) => match map.read() {
                Ok(map) => {
                    let value = map.get(&key).map_or("", |v| v.as_str());
                    write_response(&mut stream, value.as_bytes());
                }
                Err(e) => eprintln!("{e}"),
            },
            Frame::Put(key, value) => match map.write() {
                Ok(mut map) => {
                    let value = map.insert(key.clone(), value.clone()).map_or(value.as_str(), |_| "");
                    write_response(&mut stream, value.as_bytes());
                }
                Err(e) => eprintln!("{e}"),
            },
            Frame::Delete(key) => match map.write() {
                Ok(mut map) => {
                    let value = map.remove(&key).map_or(String::from(""), |v| v);
                    write_response(&mut stream, value.as_bytes());
                } 
                Err(e) => eprintln!("{e}"),
            },
        }
    }
}

fn main() -> std::io::Result<()> {
    let map = Arc::new(RwLock::new(HashMap::<String, String>::new()));
    let listener = net::TcpListener::bind("127.0.0.1:6379")?;
    for incoming in listener.incoming() {
        let map = map.clone();
        if let Ok(incoming) = incoming {
            thread::spawn(move || {
                if let Err(e) = handle_client(incoming, map) {
                    eprintln!("Client error: {e:?}");
                }
            });
        }
    }
    Ok(())
}
