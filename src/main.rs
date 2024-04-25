use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

use http_server_starter_rust::HttpRequest;
use std::env;

fn handle_client(mut stream: TcpStream, directory: String) {
    let http_request = HttpRequest::build(&stream);
    let method = http_request.method.as_str();
    let path = http_request.path.as_str();
    let headers = &http_request.headers;
    let body = http_request.body.as_str();

    // Logging the request
    println!("\nStarted {} \"{}\"", method, path);
    println!("  Headers: {:?}", headers);
    println!("  Body: {}", body);

    // Writing the response
    let response = route_handler(path, &headers, directory);
    stream.write(response.as_bytes()).unwrap();
}

fn route_handler(path: &str, headers: &HashMap<String, String>, directory: String) -> String {
    let response: String = match path {
        "/" => "HTTP/1.1 200 OK\r\n\r\n".to_string(),

        path if path.starts_with("/echo") => {
            let query = &path[6..];
            let query_len = query.len();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                query_len, query
            )
        }

        "/user-agent" => {
            let default_user_agent = "Unknown".to_string();
            let user_agent = headers.get("User-Agent").unwrap_or(&default_user_agent);
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                user_agent.len(),
                user_agent
            )
        }

        path if path.starts_with("/files") => {
            let file_path = &path[7..];
            let file_path = format!("{}/{}", directory, file_path);
            match std::fs::read_to_string(file_path) {
                Ok(content) => {
                    let content_len = content.len();
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                        content_len, content
                    )
                }
                Err(_) => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
            }
        }

        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
    };
    response
}

fn main() {
    let mut args = env::args();

    let mut directory: String = "".to_string();
    while let Some(arg) = args.next() {
        if arg == "--directory" {
            directory = args.next().unwrap();
        }
    }
    println!("directory: {}", directory);

    let port = 4221;
    let address = format!("127.0.0.1:{}", port);
    let listener: TcpListener = TcpListener::bind(address).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let new_dir = directory.clone();
                thread::spawn(move || {
                    handle_client(stream, new_dir);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
