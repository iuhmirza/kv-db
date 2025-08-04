use std::collections::HashMap;
use std::net;
use std::sync::{Arc, RwLock};
use std::thread;
use crate::protocol::{Request, ReadRequestError, read_request, write_response};


type KVMap = Arc<RwLock<HashMap<String, String>>>;

fn handle_client(mut stream: net::TcpStream, map: KVMap) -> Result<(), ReadRequestError>{
    loop {
        let command = read_request(&mut stream)?;
        match command {
            Request::Get(key) => match map.read() {
                Ok(map) => {
                    let value = map.get(&key).map_or("", |v| v.as_str());
                    write_response(&mut stream, value.as_bytes())?
                }
                Err(e) => eprintln!("{e}"),
            },
            Request::Put(key, value) => match map.write() {
                Ok(mut map) => {
                    // let value = map.insert(key.clone(), value.clone()).map_or(value.as_str(), |_| "");
                    // write_response(&mut stream, value.as_bytes())?
                }
                Err(e) => eprintln!("{e}"),
            },
            Request::Delete(key) => match map.write() {
                Ok(mut map) => {
                    // let value = map.remove(&key).map_or(String::from(""), |v| v);
                    // write_response(&mut stream, value.as_bytes())?
                } 
                Err(e) => eprintln!("{e}"),
            },
        }
    }
}

pub fn run() -> std::io::Result<()> {
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
