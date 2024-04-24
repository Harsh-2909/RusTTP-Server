use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    // Reading the HTTP request
    let mut buffer = [0; 2048];
    stream.read(&mut buffer).unwrap();
    let request_str = std::str::from_utf8(&buffer).unwrap();
    println!("Request: {}", request_str);

    // Parsing the request and finding the path
    let path: &str = request_str.split_ascii_whitespace().nth(1).unwrap();
    println!("Path: {}", path);

    // Writing the response
    match path {
        "/" => {
            stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
        }
        _ => {
            stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
        }
    }
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener: TcpListener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                handle_client(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
