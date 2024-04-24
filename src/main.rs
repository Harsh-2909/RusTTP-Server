use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    // Reading the HTTP request
    let mut buffer = [0; 2048];
    stream.read(&mut buffer).unwrap();
    let request_str = std::str::from_utf8(&buffer).unwrap();
    // println!("Request: {}", request_str);

    // Parsing the request and finding the path
    let method: &str = request_str.split_ascii_whitespace().next().unwrap();
    let path: &str = request_str.split_ascii_whitespace().nth(1).unwrap();
    let headers: HashMap<String, String> = header_parser(request_str);
    println!("Started {method} \"{path}\"", method = method, path = path);
    println!("  Headers: {:?}", headers);

    // Writing the response
    let response = route_handler(path, &headers);
    stream.write(response.as_bytes()).unwrap();
}

#[warn(dead_code)]
fn header_builder(headers: HashMap<String, String>) -> String {
    let mut header_str = String::new();
    for (key, value) in headers {
        header_str.push_str(&format!("{}: {}\r\n", key, value));
    }
    header_str
}

fn header_parser(header_str: &str) -> HashMap<String, String> {
    let mut headers: HashMap<String, String> = HashMap::new();
    header_str.lines().for_each(|line| {
        if line.is_empty() {
            return;
        }
        let mut parts = line.split(": ");
        let key = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default();
        if key.is_empty() || value.is_empty() {
            return;
        }
        headers.insert(key.to_string(), value.to_string());
    });
    headers
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
                println!("\naccepted new connection");
                handle_client(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
