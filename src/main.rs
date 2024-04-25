use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

use http_server_starter_rust::HttpRequest;

fn handle_client(mut stream: TcpStream) {
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
    let response = route_handler(path, &headers);
    stream.write(response.as_bytes()).unwrap();
}

fn route_handler(path: &str, headers: &HashMap<String, String>) -> String {
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

        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
    };
    response
}

fn main() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
