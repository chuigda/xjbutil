use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;
use std::thread;

use crate::http_commons::{HttpBody, HttpHeaders, HttpParams, HttpResponse, HttpUri};
use crate::http_commons::http_code_describe;

const HTTP_404_STRING: &'static str = include_str!("../resc/http_404.html");

pub type HttpHandler = Box<
    dyn Fn(HttpHeaders, HttpParams, HttpBody) -> Result<HttpResponse, Box<dyn Error>>
        + Send
        + Sync
        + 'static
>;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum HttpLogLevel {
    Debug, Info, Warn, Error
}

pub type HttpLogger = fn(level: HttpLogLevel, info: &str) -> ();

pub struct MinHttpd {
    handlers: HashMap<HttpUri, HttpHandler>,
    logger: Option<HttpLogger>,
    request_counter: AtomicU64
}

impl MinHttpd {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            logger: None,
            request_counter: AtomicU64::new(0)
        }
    }

    pub fn with_logger(logger: HttpLogger) -> Self {
        Self {
            handlers: HashMap::new(),
            logger: Some(logger),
            request_counter: AtomicU64::new(0)
        }
    }

    pub fn route(&mut self, uri: HttpUri, handler: HttpHandler) {
        self.handlers.insert(uri, handler);
    }

    pub fn serve(&self, addr: SocketAddrV4) -> Result<Infallible, Box<dyn Error>> {
        let tcp_listener = TcpListener::bind(addr)?;
        loop {
            let (stream, addr) = tcp_listener.accept()?;
            let request_id = self.request_counter.fetch_add(1, SeqCst);
            self.log(
                HttpLogLevel::Info,
                &format!("[MIN-HTTPD/{}] Accepted connection from: {}", request_id, addr)
            );

            self.handle_connection(stream, request_id);
        }
    }

    fn handle_connection(&self, stream: TcpStream, request_id: u64) {
        let unsafe_self = unsafe {
            std::mem::transmute::<&'_ Self, &'static Self>(&self)
        };

        thread::spawn(move || {
            match Self::handle_connection_impl(unsafe_self, stream, request_id) {
                Ok(_) => {},
                Err(e) => unsafe_self.log(
                    HttpLogLevel::Error,
                    &format!("[MIN-HTTPD/{}] Error handling connection: {}", request_id, e)
                )
            }
        });
    }

    fn handle_connection_impl(
        &self,
        stream: TcpStream,
        request_id: u64
    ) -> Result<(), Box<dyn Error>>{
        let mut reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);

        let mut line = String::new();
        reader.read_line(&mut line)?;

        let parts = line.trim().split_whitespace().collect::<Vec<_>>();
        if parts.len() != 3 {
            self.log(
                HttpLogLevel::Error,
                &format!("[MIN-HTTPD/{}] Invalid HTTP request: {}", request_id, line)
            );
            return Ok(());
        }
        let method = parts[0].to_lowercase();
        let version = parts[2].to_lowercase();

        if method != "get" && parts[0] != "post" {
            self.log(
                HttpLogLevel::Error,
                &format!("[MIN-HTTPD/{}] Invalid HTTP method: {}", request_id, parts[0])
            );
            return Ok(());
        }
        if version != "http/1.1" && version != "http/1.0" {
            self.log(
                HttpLogLevel::Error,
                &format!("[MIN-HTTPD/{}] Invalid HTTP version: {}", request_id, parts[2])
            );
            return Ok(());
        }

        let uri = parts[1].to_string();
        let uri_parts = uri.split("?").collect::<Vec<_>>();
        let uri = uri_parts[0].to_string();
        let params = if uri_parts.len() > 1 {
            let mut params = HashMap::new();
            for param in uri_parts[1].split("&") {
                let param_parts = param.split("=").collect::<Vec<_>>();
                if param_parts.len() != 2 {
                    self.log(
                        HttpLogLevel::Error,
                        &format!("[MIN-HTTPD/{}] Invalid HTTP parameter: {}", request_id, param)
                    );
                    return Ok(());
                }
                params.insert(
                    param_parts[0].to_string(),
                    param_parts[1].to_string()
                );
            }
            params
        } else {
            HashMap::new()
        };

        let mut headers = HashMap::new();
        loop {
            line.clear();
            reader.read_line(&mut line)?;
            if line.trim().is_empty() {
                break;
            }
            let parts = line.trim().split(": ").collect::<Vec<_>>();
            if parts.len() != 2 {
                self.log(
                    HttpLogLevel::Error,
                    &format!("[MIN-HTTPD/{}] Invalid HTTP header: {}", request_id, line)
                );
                return Ok(());
            }
            headers.insert(
                parts[0].to_lowercase().to_string(),
                parts[1].to_lowercase().to_string()
            );
        }

        let body = if headers.contains_key("content-length") {
            let content_length = headers["content-length"].parse::<usize>()?;
            let mut buffer = vec![0; content_length];
            reader.read_exact(&mut buffer)?;
            Some(buffer)
        } else {
            None
        };

        let handler = self.handlers.get(&uri);
        if let Some(handler) = handler {
            let result = handler(
                headers,
                params,
                body.map(|b| String::from_utf8_lossy(b.as_ref()).to_string()),
            );
            let mut response = match result {
                Ok(result) => result,
                Err(e) => {
                    HttpResponse::new(
                        500,
                        vec![("Connection".to_string(), "close".to_string())],
                        Some(format!(include_str!("../resc/http_500.html"), e)),
                    )
                }
            };

            if !response.has_header("Server") {
                response.add_header("Server", "xjbutil/0.7 rhttpd");
            }

            write!(
                writer,
                "HTTP/1.1 {} {}\r\n",
                response.code,
                http_code_describe(response.code)
            )?;
            for (key, value) in response.headers {
                write!(writer, "{}: {}\r\n", key, value)?;
            }
            if let Some(payload) = response.payload {
                write!(writer, "Content-Length: {}\r\n", payload.len())?;
                write!(writer, "\r\n")?;
                writer.write_all(payload.as_bytes())?;
            } else {
                write!(writer, "\r\n")?;
            }
        } else {
            self.log(
                HttpLogLevel::Warn,
                &format!("[MIN-HTTPD/{}] No handler for URI: {}", request_id, uri),
            );

            write!(writer, "HTTP/1.1 404 Not Found\r\n")?;
            write!(writer, "Connection: close\r\n")?;
            write!(writer, "Content-Length: {}\r\n", HTTP_404_STRING.len())?;
            write!(writer, "\r\n")?;
            writer.write_all(HTTP_404_STRING.as_bytes())?;
        };
        writer.flush()?;

        Ok(())
    }

    fn log(&self, log_level: HttpLogLevel, info: &str) {
        if let Some(logger) = &self.logger {
            logger(log_level, info);
        }
    }
}

impl Default for MinHttpd {
    fn default() -> Self {
        Self::new()
    }
}
