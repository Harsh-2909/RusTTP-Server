use phf::phf_map;
use std::io::Read;
use std::{collections::HashMap, net::TcpStream};

static HTTP_RESPONSE_CODES: phf::Map<u16, &'static str> = phf_map! {
    // Informational
    100u16 => "Continue",
    101u16 => "Switching Protocols",
    102u16 => "Processing",
    103u16 => "Early Hints",
    // Success
    200u16 => "OK",
    201u16 => "Created",
    202u16 => "Accepted",
    203u16 => "Non-Authoritative Information",
    204u16 => "No Content",
    205u16 => "Reset Content",
    206u16 => "Partial Content",
    207u16 => "Multi-Status",
    208u16 => "Already Reported",
    226u16 => "IM Used",
    // Redirection
    300u16 => "Multiple Choices",
    301u16 => "Moved Permanently",
    302u16 => "Found",
    303u16 => "See Other",
    304u16 => "Not Modified",
    305u16 => "Use Proxy",
    306u16 => "Switch Proxy",
    307u16 => "Temporary Redirect",
    308u16 => "Permanent Redirect",
    // Client Error
    400u16 => "Bad Request",
    401u16 => "Unauthorized",
    402u16 => "Payment Required",
    403u16 => "Forbidden",
    404u16 => "Not Found",
    405u16 => "Method Not Allowed",
    406u16 => "Not Acceptable",
    407u16 => "Proxy Authentication Required",
    408u16 => "Request Timeout",
    409u16 => "Conflict",
    410u16 => "Gone",
    411u16 => "Length Required",
    412u16 => "Precondition Failed",
    413u16 => "Payload Too Large",
    414u16 => "URI Too Long",
    415u16 => "Unsupported Media Type",
    416u16 => "Range Not Satisfiable",
    417u16 => "Expectation Failed",
    418u16 => "I'm a teapot",
    421u16 => "Misdirected Request",
    422u16 => "Unprocessable Entity",
    423u16 => "Locked",
    424u16 => "Failed Dependency",
    425u16 => "Too Early",
    426u16 => "Upgrade Required",
    428u16 => "Precondition Required",
    429u16 => "Too Many Requests",
    431u16 => "Request Header Fields Too Large",
    444u16 => "No Response",
    449u16 => "Retry With",
    450u16 => "Blocked by Windows Parental Controls",
    451u16 => "Unavailable For Legal Reasons",
    499u16 => "Client Closed Request",
    // Server Error
    500u16 => "Internal Server Error",
    501u16 => "Not Implemented",
    502u16 => "Bad Gateway",
    503u16 => "Service Unavailable",
    504u16 => "Gateway Timeout",
    505u16 => "HTTP Version Not Supported",
    506u16 => "Variant Also Negotiates",
    507u16 => "Insufficient Storage",
    508u16 => "Loop Detected",
    510u16 => "Not Extended",
    511u16 => "Network Authentication Required",
    598u16 => "Network read timeout error",
    599u16 => "Network connect timeout error",
};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    pub fn new(
        method: HttpMethod,
        path: String,
        query_params: HashMap<String, String>,
        http_version: String,
        headers: HashMap<String, String>,
        body: String,
    ) -> Self {
        Self {
            method,
            path,
            query_params,
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
        let full_path: &str = request_str.split_ascii_whitespace().nth(1).unwrap();
        let path: &str = full_path.split('?').next().unwrap();
        let query_params: HashMap<String, String> = full_path
            .split('?')
            .nth(1)
            .unwrap_or_default()
            .split('&')
            .map(|param| {
                let mut parts = param.split('=');
                let key = parts.next().unwrap_or_default();
                let value = parts.next().unwrap_or_default();
                (key.to_string(), value.to_string())
            })
            .collect();
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
            query_params,
            http_version.to_string(),
            headers,
            body,
        )
    }

    pub fn query_string(&self) -> String {
        self.query_params
            .iter()
            .map(|(key, value)| {
                if !key.is_empty() && !value.is_empty() {
                    format!("{}={}", key, value)
                } else {
                    String::new()
                }
            })
            .collect::<Vec<String>>()
            .join("&")
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    http_version: String,
}

impl HttpResponse {
    pub fn new(
        status_code: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: String,
        http_version: String,
    ) -> Self {
        Self {
            status_code,
            status_text,
            headers,
            body,
            http_version,
        }
    }

    pub fn builder() -> Self {
        Self {
            status_code: 200,
            status_text: HTTP_RESPONSE_CODES.get(&200).unwrap().to_string(),
            headers: HashMap::new(),
            body: String::new(),
            http_version: "HTTP/1.1".to_string(),
        }
    }

    pub fn status_code(&mut self, status_code: u16) -> &mut Self {
        self.status_code = status_code;
        self.status_text = HTTP_RESPONSE_CODES
            .get(&status_code)
            .unwrap_or(&"Unknown Status Code")
            .to_string();
        self
    }

    pub fn body(&mut self, body: &str) -> &mut Self {
        self.body = body.to_string();
        self
    }

    pub fn headers(&mut self, headers: HashMap<String, String>) -> &mut Self {
        self.headers = headers;
        self
    }

    pub fn add_header(&mut self, key: &str, value: &str) -> &mut Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn init(&self) -> &Self {
        self
    }

    pub fn build(&self) -> String {
        let header_str = header_builder(self.headers.clone());
        format!("{}{}{}", self.status_line(), header_str, self.body)
    }

    pub fn status_line(&self) -> String {
        format!(
            "{} {} {}\r\n",
            self.http_version, self.status_code, self.status_text
        )
    }

    pub fn status(&self) -> String {
        format!("{} {}", self.status_code, self.status_text)
    }
}

pub fn header_builder(headers: HashMap<String, String>) -> String {
    let mut header_str = String::new();
    for (key, value) in headers {
        header_str.push_str(&format!("{}: {}\r\n", key, value));
    }
    header_str.push_str("\r\n");
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
