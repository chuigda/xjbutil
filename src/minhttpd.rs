use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;
use std::thread;

pub use crate::http_commons::{HttpBody, HttpHeaders, HttpParams, HttpResponse, HttpUri};
pub use crate::http_commons::http_code_describe;

const HTTP_404_STRING: &'static str = include_str!("../resc/http_404.html");

pub type HttpHandler = Box<
    dyn Fn(HttpHeaders, HttpParams, HttpBody) -> Result<HttpResponse, Box<dyn Error>>
        + Send
        + Sync
        + 'static
>;

type HttpHandlerFn = fn(HttpHeaders, HttpParams, HttpBody) -> Result<HttpResponse, Box<dyn Error>>;

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

    pub fn route(&mut self, uri: &str, handler: HttpHandler) {
        self.handlers.insert(uri.to_string(), handler);
    }

    pub fn route_fn(&mut self, uri: &str, handler_fn: HttpHandlerFn) {
        self.handlers.insert(uri.to_string(), Box::new(handler_fn));
    }

    pub fn route_static(&mut self, uri: &str, content_type: &str, content: String) {
        let content_type: String = content_type.to_string();
        self.handlers.insert(uri.to_string(), Box::new(move |_, _, _| {
            Ok(HttpResponse::new(
                200,
                vec![("Content-Type".to_string(), content_type.clone())],
                Some(content.clone()))
            )
        }));
    }

    pub fn serve(&self, addr: SocketAddrV4) -> Result<Infallible, Box<dyn Error>> {
        let tcp_listener: TcpListener = TcpListener::bind(addr)?;
        loop {
            let (stream, addr): (TcpStream, SocketAddr) = tcp_listener.accept()?;
            let request_id: u64 = self.request_counter.fetch_add(1, SeqCst);
            self.log(
                HttpLogLevel::Info,
                &format!("[MIN-HTTPD/{}] Accepted connection from: {}", request_id, addr)
            );

            self.handle_connection(stream, addr.ip().to_string(), request_id);
        }
    }

    fn handle_connection(&self, stream: TcpStream, remote_addr: String, request_id: u64) {
        let unsafe_self: &'static Self = unsafe { std::mem::transmute::<&'_ _, &'static _>(self) };
        thread::spawn(move || {
            match Self::handle_connection_impl(unsafe_self, stream, remote_addr, request_id) {
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
        remote_addr: String,
        request_id: u64
    ) -> Result<(), Box<dyn Error>> {
        let mut reader: BufReader<&TcpStream> = BufReader::new(&stream);
        let mut writer: BufWriter<&TcpStream> = BufWriter::new(&stream);

        let mut line: String = String::new();
        reader.read_line(&mut line)?;

        let parts: Vec<&str> = line.trim().split_whitespace().collect::<Vec<_>>();
        if parts.len() != 3 {
            self.log(
                HttpLogLevel::Error,
                &format!("[MIN-HTTPD/{}] Invalid HTTP request: {}", request_id, line)
            );
            return Ok(());
        }
        let method: String = parts[0].to_lowercase();
        let version: String = parts[2].to_lowercase();

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

        let uri: String = parts[1].to_string();
        let uri_parts: Vec<&str> = uri.split("?").collect::<Vec<_>>();
        let mut uri: String = uri_parts[0].to_string();

        if uri.ends_with("/") {
            uri.pop();
        }

        let params: HashMap<String, String> = if uri_parts.len() > 1 {
            let mut params: HashMap<String, String> = HashMap::new();
            for param in uri_parts[1].split("&") {
                let param_parts: Vec<&str> = param.split("=").collect::<Vec<_>>();
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

        let mut headers: HashMap<String, String> = HashMap::new();
        loop {
            line.clear();
            reader.read_line(&mut line)?;
            if line.trim().is_empty() {
                break;
            }
            let parts: Vec<&str> = line.trim().split(": ").collect::<Vec<_>>();
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
        headers.insert("X-47-Remote-Addr".to_string(), remote_addr);

        let body: Option<Vec<u8>> = if headers.contains_key("content-length") {
            let content_length: usize = headers["content-length"].parse()?;
            let mut buffer: Vec<u8> = vec![0; content_length];
            reader.read_exact(&mut buffer)?;
            Some(buffer)
        } else {
            None
        };

        let handler: Option<&HttpHandler> = self.handlers.get(&uri);
        if let Some(handler) = handler {
            let result: Result<HttpResponse, Box<dyn Error>> = handler(
                headers,
                params,
                body.map(|b| String::from_utf8_lossy(b.as_ref()).to_string()),
            );
            let mut response: HttpResponse = match result {
                Ok(result) => result,
                Err(e) => {
                    self.log(
                        HttpLogLevel::Error,
                        &format!("[MIN-HTTPD/{}] Error handling request: {}", request_id, e)
                    );
                    HttpResponse::new(
                        500,
                        vec![("Content-Type".to_string(), "text/html".to_string())],
                        Some(format!(include_str!("../resc/http_500.html"), e)),
                    )
                }
            };

            if response.has_header("Content-Length") {
                self.log(
                    HttpLogLevel::Error,
                    &format!("[MIN-HTTPD/{}] Setting `Content-Length` is not allowed", request_id)
                );
                return Ok(());
            }

            if response.has_header("Connection") {
                self.log(
                    HttpLogLevel::Error,
                    &format!("[MIN-HTTPD/{}] Setting `Connection` is not allowed", request_id)
                );
                return Ok(());
            }

            response.add_header("Connection", "close");
            if !response.has_header("Server") {
                response.add_header("Server", "xjbutil/0.7 rhttpd");
            }

            write!(
                writer,
                "HTTP/1.1 {} {}\r\n",
                response.code,
                http_code_describe(response.code)
            )?;
            for (key /*: String*/, value /*: String*/) in response.headers {
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
            write!(writer, "Content-Type: text/html\r\n")?;
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::error::Error;
    use std::net::{Ipv4Addr, SocketAddrV4};

    use crate::minhttpd::{HttpResponse, MinHttpd};

    #[test]
    #[ignore]
    fn test_min_httpd() {
        fn example_handler(
            _headers: HashMap<String, String>,
            params: HashMap<String, String>,
            _body: Option<String>,
        ) -> Result<HttpResponse, Box<dyn Error>> {
            Ok(
                HttpResponse::builder()
                    .add_header("Content-Type", "text/plain")
                    .set_payload(
                        format!("Hello, {}!", params.get("name").unwrap_or(&"world".to_string())),
                    )
                    .build()
            )
        }

        fn example_500_handler(
            _headers: HashMap<String, String>,
            _params: HashMap<String, String>,
            _body: Option<String>,
        ) -> Result<HttpResponse, Box<dyn Error>> {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Example error")))
        }

        let mut min_httpd = MinHttpd::with_logger(|_, content| { dbg!(content); });
        min_httpd.route_fn("/hello", example_handler);
        min_httpd.route_fn("/error", example_500_handler);
        if let Err(e) = min_httpd.serve(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3080)) {
            panic!("{}", e);
        }
    }
}
