use std::io::Read;
use std::{collections::HashMap, net::TcpStream};
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
    TRACE,
    CONNECT,
}

impl HttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::TRACE => "TRACE",
            HttpMethod::CONNECT => "CONNECT",
        }
    }

    pub fn from_str(method: &str) -> Self {
        match method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "PATCH" => HttpMethod::PATCH,
            "DELETE" => HttpMethod::DELETE,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            "TRACE" => HttpMethod::TRACE,
            "CONNECT" => HttpMethod::CONNECT,
            _ => HttpMethod::GET,
        }
    }
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    pub fn new(
        method: HttpMethod,
        path: String,
        http_version: String,
        headers: HashMap<String, String>,
        body: String,
    ) -> Self {
        Self {
            method,
            path,
            http_version,
            headers,
            body,
        }
    }

    pub fn build(mut stream: &TcpStream) -> Self {
        // Reading the HTTP request from the stream
        let mut buffer = [0; 2048];
        stream.read(&mut buffer).unwrap();
        let request_str = String::from_utf8_lossy(&buffer).to_string();

        // Parsing the request and building the HttpRequest object
        let method: &str = request_str.split_ascii_whitespace().next().unwrap();
        let method: HttpMethod = HttpMethod::from_str(method);
        let path: &str = request_str.split_ascii_whitespace().nth(1).unwrap();
        let http_version: &str = request_str.split_ascii_whitespace().nth(2).unwrap();
        let header_str: &str = request_str.split("\r\n\r\n").next().unwrap();
        let headers: HashMap<String, String> = header_parser(header_str);
        let body: String = request_str
            .split("\r\n\r\n")
            .nth(1)
            .unwrap_or_default()
            .replace("\0", ""); // Remove escape null characters

        Self::new(
            method,
            path.to_string(),
            http_version.to_string(),
            headers,
            body,
        )
    }
}

pub fn header_builder(headers: HashMap<String, String>) -> String {
    let mut header_str = String::new();
    for (key, value) in headers {
        header_str.push_str(&format!("{}: {}\r\n", key, value));
    }
    header_str
}

pub fn header_parser(header_str: &str) -> HashMap<String, String> {
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
