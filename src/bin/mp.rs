use std::collections::HashMap;
use std::net;
use std::sync::mpsc;
use std::thread;
use kv_db::{Frame, read_frame, write_response, ReadFrameError};

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


fn handle_client(
    mut stream: net::TcpStream,
    sender: mpsc::Sender<Command>,
) -> Result<(), ReadFrameError> {
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
                write_response(&mut stream, value.as_bytes())?
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
