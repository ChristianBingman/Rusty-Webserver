mod file;
mod http10;
mod middleware;
mod threadpool;
mod util;

#[derive(Debug, PartialEq)]
pub struct Opts {
    /// port to bind to
    pub port: u16,

    /// address to bind to
    pub bind: String,

    /// directory to serve
    pub directory: String,

    /// protocol to use (supports http 1.0)
    pub protocol: String,

    /// Auth for basic authentication
    pub auth: Option<Auth>,

    /// compression ratio (0-9, default 6)
    pub ratio: u32,
}

#[derive(Debug, PartialEq)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

impl Default for Opts {
    fn default() -> Self {
        Opts {
            port: 8080,
            bind: "127.0.0.1".to_string(),
            directory: "./".to_string(),
            protocol: "HTTP/1.0".to_string(),
            auth: None,
            ratio: 6,
        }
    }
}

pub mod http_server {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;
    use std::time::Duration;

    use crate::http10::headers::{Header, HeaderVariant, Headers};
    use crate::http10::methods::Method;
    use crate::http10::request::ReqError;
    use crate::http10::result_codes::ResultCode;
    use crate::http10::{request::HTTPRequest, response::HTTPResponse};
    use crate::middleware;
    use crate::middleware::get_handler;
    use crate::threadpool::ThreadPoolQ;
    use crate::util::html::error_page;

    use super::Opts;

    #[derive(Debug, PartialEq)]
    pub enum HTTPServerClass {
        Simple,
        Threaded,
        ThreadPooled(usize),
    }

    pub struct HTTPServer {
        class: HTTPServerClass,
        opts: Arc<Opts>,
        handler: Box<dyn Fn(HTTPRequest, &Arc<Opts>) -> HTTPResponse + Send + Sync + 'static>,
    }

    impl HTTPServer {
        fn default_handler(req: HTTPRequest, opts: &Arc<Opts>) -> HTTPResponse {
            if let Some(auth) = &opts.auth {
                match middleware::basic_auth(&req, auth) {
                    Err(..) => {
                        let mut headers = Headers::default();
                        headers.set(Header::WWWAuthenticate("Basic".to_string()));
                        headers.set(Header::ContentType("text/html".to_string()));
                        return HTTPResponse::new(
                            opts.protocol.clone(),
                            ResultCode::Unauthorized,
                            headers,
                            Some(error_page(ResultCode::Unauthorized).as_bytes().to_vec()),
                        );
                    }
                    Ok(..) => (),
                }
            }

            match req.method {
                Method::GET => get_handler(&req, opts),
                Method::HEAD => {
                    let mut resp = get_handler(&req, opts);
                    resp.body = None;
                    resp
                }
                Method::POST => {
                    let mut headers = Headers::default();
                    headers.set(Header::ContentType("text/html".to_string()));
                    HTTPResponse::new(
                        opts.protocol.clone(),
                        ResultCode::NotImplemented,
                        headers,
                        Some(error_page(ResultCode::NotImplemented).as_bytes().to_vec()),
                    )
                }
            }
        }

        fn handle_stream(
            mut stream: TcpStream,
            handler: &Box<dyn Fn(HTTPRequest, &Arc<Opts>) -> HTTPResponse + Send + Sync + 'static>,
            opts: &Arc<Opts>,
        ) {
            // Only fails when duration is 0 which we explicitly do not set
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            let remote: String = match stream.peer_addr() {
                Ok(addr) => addr.to_string(),
                Err(_) => "Invalid Address".to_string(),
            };
            let mut request: Vec<u8> = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                match HTTPRequest::try_from(&request) {
                    Err(ReqError::ContentLenError) => match stream.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            request.append(buf[..n].to_vec().as_mut());
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => (),
                        Err(_) => break,
                    },
                    _ => break,
                }
            }
            let request = match HTTPRequest::try_from(&request) {
                Ok(req) => req,
                Err(_) => {
                    let headers = Headers::default();
                    let mut resp = HTTPResponse {
                        version: opts.protocol.clone(),
                        status: ResultCode::BadRequest,
                        headers,
                        body: Some(error_page(ResultCode::BadRequest).as_bytes().to_vec()),
                    };
                    let _ = stream.write_all(resp.as_bytes().as_slice());
                    log::error!("Malformed request from: {}", remote);
                    log::debug!("Received: {:?}", request);
                    return;
                }
            };

            // Gathering info used for logging
            let headline = format!(
                "{} {} {}",
                Into::<String>::into(request.method),
                request.uri,
                request.version
            );
            let user_agent = request.headers.get(HeaderVariant::UserAgent);
            let user_agent = match user_agent {
                Some(Header::UserAgent(inner)) => inner,
                _ => "-".to_string(),
            };
            let req_headers = request.headers.to_string();

            // Pass off the request to the handler
            let mut resp = handler(request, opts);

            //More log data gathering
            let code = Into::<usize>::into(resp.status);
            let content_len = match resp.headers.get(HeaderVariant::ContentLength) {
                Some(Header::ContentLength(len)) => len,
                _ => 0,
            };
            let resp_headers = resp.headers.to_string();

            // Send the response back
            stream.write_all(resp.as_bytes().as_slice()).unwrap();

            log::info!(
                "{} {} {} {} {}",
                headline,
                code,
                content_len,
                user_agent,
                remote
            );
            log::debug!(
                "Request headers: {}\nResponse Headers: {}",
                req_headers,
                resp_headers
            );
        }

        pub fn new(
            class: HTTPServerClass,
            opts: Opts,
            handler: Option<
                Box<dyn Fn(HTTPRequest, &Arc<Opts>) -> HTTPResponse + Send + Sync + 'static>,
            >,
        ) -> HTTPServer {
            let opts = Arc::new(opts);
            match handler {
                Some(handler) => HTTPServer {
                    class,
                    opts,
                    handler,
                },
                None => HTTPServer {
                    class,
                    opts,
                    handler: Box::new(HTTPServer::default_handler),
                },
            }
        }

        pub fn serve_forever(self) {
            let listener = TcpListener::bind(format!("{}:{}", self.opts.bind, self.opts.port))
                .expect("Unable to bind!");

            log::info!("Started listener on {}:{}", self.opts.bind, self.opts.port);

            match self.class {
                HTTPServerClass::Simple => {
                    let opts = Arc::clone(&self.opts);

                    for stream in listener.incoming() {
                        match stream {
                            Ok(stream) => HTTPServer::handle_stream(stream, &self.handler, &opts),
                            Err(e) => {
                                log::error!("Failed to establish a connection: {}", e);
                            }
                        }
                    }
                }
                HTTPServerClass::Threaded => {
                    let handler = Arc::new(self.handler);

                    for stream in listener.incoming() {
                        match stream {
                            Ok(stream) => {
                                let handler = Arc::clone(&handler);
                                let opts = Arc::clone(&self.opts);
                                std::thread::spawn(move || {
                                    HTTPServer::handle_stream(stream, &handler, &opts);
                                });
                            }
                            Err(e) => {
                                log::error!("Failed to establish a connection: {}", e);
                            }
                        }
                    }
                }
                HTTPServerClass::ThreadPooled(threads) => {
                    let opts = Arc::clone(&self.opts);
                    let mut tpq = ThreadPoolQ::new(threads, move |stream| {
                        HTTPServer::handle_stream(stream, &self.handler, &opts)
                    });
                    for stream in listener.incoming() {
                        match stream {
                            Ok(stream) => {
                                tpq.push_job(stream);
                            }
                            Err(e) => {
                                log::error!("Failed to establish a connection: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_create_single_threaded_server() {
            HTTPServer::new(HTTPServerClass::Simple, Opts::default(), None);
        }

        #[test]
        fn test_create_threaded_server() {
            HTTPServer::new(HTTPServerClass::Threaded, Opts::default(), None);
        }

        #[test]
        fn test_create_threadpool_server() {
            HTTPServer::new(HTTPServerClass::ThreadPooled(5), Opts::default(), None);
        }
    }
}
