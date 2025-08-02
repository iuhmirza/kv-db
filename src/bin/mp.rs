use std::collections::HashMap;
use std::io::{Read, Write};
use std::net;
use std::sync::mpsc;
use std::thread;

enum Command {
    Put(String, String),
    Get(String, mpsc::Sender<Option<String>>),
    Delete(String),
}

fn run_db_worker(rx: mpsc::Receiver<Command>) {
    let mut map: HashMap<String, String> = HashMap::new();
    loop {
        let read: Command = rx.recv().expect("no more senders");
        match read {
            Command::Get(k, tx) => {
                let value = match map.get(&k) {
                    Some(v) => Some(v.clone()),
                    None => None
                };
                tx.send(value);
            }
            Command::Put(k, v) => {
                map.insert(k, v);
            }
            Command::Delete(k) => {
                map.remove(&k);
            }
        }
    }
}

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

fn handle_client(
    mut stream: net::TcpStream,
    sender: mpsc::Sender<Command>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let frame = read_frame(&mut stream)?;
        match frame {
            Frame::Get(key) => {
                let (tx, rx) = mpsc::channel();
                sender.send(Command::Get(key, tx));
                let value = match rx.recv() {
                    Ok(Some(v)) => v,
                    _ => String::from(""),
                };
                write_response(&mut stream, value.as_bytes())
            }
            Frame::Put(key, value) => {
                sender.send(Command::Put(key, value));
            }
            Frame::Delete(key) => {
                sender.send(Command::Delete(key));
            }
        }
    }
}


fn main() -> std::io::Result<()> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || run_db_worker(rx));
    
    let listener = net::TcpListener::bind("127.0.0.1:6379")?;
    for incoming in listener.incoming() {
        let tx = tx.clone();
        if let Ok(incoming) = incoming {
            thread::spawn(move || {
                if let Err(e) = handle_client(incoming, tx) {
                    eprintln!("Client error: {e:?}");
                }
            });
        }
    }
    Ok(())
}
