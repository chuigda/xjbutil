use std::collections::HashMap;

pub type HttpUri = String;
pub type HttpHeaders = HashMap<String, String>;
pub type HttpParams = HashMap<String, String>;
pub type HttpBody = Option<String>;

pub struct HttpResponse {
    pub code: u16,
    pub headers: Vec<(String, String)>,
    pub payload: Option<String>
}

impl HttpResponse {
    pub fn new(code: u16, headers: Vec<(String, String)>, payload: Option<String>) -> Self {
        Self {
            code,
            headers,
            payload
        }
    }

    pub fn builder() -> HttpResponseBuilder {
        HttpResponseBuilder {
            code: 200,
            headers: Vec::new(),
            payload: None
        }
    }

    pub fn add_header(&mut self, header: &str, value: &str) {
        self.headers.push((header.to_string(), value.to_string()));
    }

    pub fn has_header(&self, header: &str) -> bool {
        self.headers.iter().any(|(h, _)| h.eq_ignore_ascii_case(header))
    }
}

pub struct HttpResponseBuilder {
    code: u16,
    headers: Vec<(String, String)>,
    payload: Option<String>
}

impl HttpResponseBuilder {
    pub fn set_code(mut self, code: u16) -> Self {
        self.code = code;
        self
    }

    pub fn add_header(mut self, key: String, value: String) -> Self {
        self.headers.push((key, value));
        self
    }

    pub fn set_payload(mut self, payload: String) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn build(self) -> HttpResponse {
        HttpResponse::new(self.code, self.headers, self.payload)
    }
}

pub fn http_code_describe(code: u16) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Request Entity Too Large",
        414 => "Request-URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Requested Range Not Satisfiable",
        417 => "Expectation Failed",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        _ => "Unknown"
    }
}
