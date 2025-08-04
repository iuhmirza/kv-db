use kv_db::protocol::{Request, read_string_from_stream, write_request};
use std::io;
use std::net;

fn parse_args(args: &[&str]) -> Result<Request, String> {
    match args {
        ["get", key] => Ok(Request::Get(key.to_string())),
        ["put", key, value] => Ok(Request::Put(key.to_string(), value.to_string())),
        ["del", key] => Ok(Request::Delete(key.to_string())),
        _ => Err(String::from(
            "Invalid command. Use 'get <key>', 'put <key> <value>', 'del <key>' or 'exit'.",
        )),
    }
}

fn main() {
    let mut stream = net::TcpStream::connect("127.0.0.1:6379").expect("Failed to connect to kv-db");
    loop {
        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_line(&mut buffer) {
            eprintln!("failed to read line from stdin: {e}");
            continue;
        }
        let args: Vec<&str> = buffer.split_whitespace().collect();
        if args.is_empty() {
            continue
        }
        if args[0] == "exit" {
            break
        }
        let request = match parse_args(&args) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };
        write_request(&mut stream, &request).expect("Failed to write frame to TCP stream");
        if let Request::Get(key) = request {
            let value = read_string_from_stream(&mut stream).expect("expected string from stream");
            println!("{key}:{value}");
        }
    }
}
