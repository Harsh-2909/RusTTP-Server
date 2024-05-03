use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

use http_server_starter_rust::{HttpRequest, HttpResponse};
use std::env;

fn handle_client(mut stream: TcpStream, directory: String) {
    let http_request = HttpRequest::build(&stream);
    let method = http_request.method.as_str();
    let path = http_request.path.as_str();
    let headers = &http_request.headers;
    let body = http_request.body.as_str();
    let query_string = http_request.query_string();

    let mut log_string = format!("Started {} {}", method, path);
    if !query_string.is_empty() {
        log_string.push_str(format!("?{}", query_string).as_str());
    }

    // Logging the request
    println!("\n{}", log_string);
    println!("  Headers: {:?}", headers);
    println!("  Body: {:?}", body);

    // Writing the response
    let response = route_handler(&http_request, directory);
    stream.write(response.as_bytes()).unwrap();
}

fn route_handler(http_request: &HttpRequest, directory: String) -> String {
    let method = http_request.method.as_str();
    let path = http_request.path.as_str();
    let headers = &http_request.headers;

    let response: String = match path {
        "/" => HttpResponse::builder()
            .status_code(200)
            .body("Hello, World!")
            .build(),

        path if path.starts_with("/echo") => {
            let query = &path[6..];
            let query_len = query.len();
            HttpResponse::builder()
                .status_code(200)
                .body(query)
                .add_header("Content-Type", "text/plain")
                .add_header("Content-Length", query_len.to_string().as_str())
                .build()
        }

        "/user-agent" => {
            let default_user_agent = "Unknown".to_string();
            let user_agent = headers.get("User-Agent").unwrap_or(&default_user_agent);
            HttpResponse::builder()
                .status_code(200)
                .body(user_agent)
                .add_header("Content-Type", "text/plain")
                .add_header("Content-Length", user_agent.len().to_string().as_str())
                .build()
        }

        path if path.starts_with("/files") => {
            let file_path = &path[7..];
            let file_path = format!("{}/{}", directory, file_path);
            if method == "GET" {
                match std::fs::read_to_string(file_path) {
                    Ok(content) => HttpResponse::builder()
                        .status_code(200)
                        .body(content.as_str())
                        .add_header("Content-Type", "application/octet-stream")
                        .add_header("Content-Length", content.len().to_string().as_str())
                        .build(),
                    Err(_) => HttpResponse::builder().status_code(404).build(),
                }
            } else if method == "POST" {
                let body = http_request.body.as_str();
                match std::fs::write(file_path, body) {
                    Ok(_) => HttpResponse::builder().status_code(201).build(),
                    Err(_) => HttpResponse::builder().status_code(500).build(),
                }
            } else {
                HttpResponse::builder().status_code(405).build()
            }
        }

        _ => HttpResponse::builder().status_code(404).build(),
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
