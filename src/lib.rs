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
        }
    }
}

pub mod http_server {
    use core::str;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;

    use chrono::Utc;

    use crate::file::{File, FileError};
    use crate::http10::headers::Header;
    use crate::http10::methods::Method;
    use crate::http10::result_codes::ResultCode;
    use crate::http10::{request::HTTPRequest, response::HTTPResponse};
    use crate::threadpool::ThreadPoolQ;
    use crate::util::html::error_page;
    use crate::{middleware, util};

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
            let mut headers: Vec<Header> = vec![
                Header::Date(Utc::now().into()),
                Header::Server("Rusty Webserver".to_string()),
            ];

            if let Some(auth) = &opts.auth {
                match middleware::basic_auth(&req, auth) {
                    Err(..) => {
                        headers.push(Header::WWWAuthenticate("Basic".to_string()));
                        headers.push(Header::ContentType("text/html".to_string()));
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
                Method::GET => {
                    let f = File::try_load(&req.uri, &opts.directory);
                    match f {
                        Ok(file) => {
                            let cond_modified = req.headers.iter().find(|header| {
                                matches!(header, crate::http10::headers::Header::IfModifiedSince(_))
                            });
                            if let Some(cond_modified) = cond_modified {
                                let dt = cond_modified.date_inner();
                                if dt.is_some() && dt.unwrap() > file.get_modified() {
                                    return HTTPResponse::new(
                                        opts.protocol.clone(),
                                        ResultCode::NotModified,
                                        headers,
                                        None,
                                    );
                                }
                            }
                            headers.push(Header::ContentType(file.get_mime()));
                            headers.push(Header::ContentLength(file.get_size()));
                            headers.push(Header::LastModified(file.get_modified()));
                            HTTPResponse::new(
                                opts.protocol.clone(),
                                ResultCode::OK,
                                headers,
                                Some(file.get_content()),
                            )
                        }
                        Err(err) => match err {
                            FileError::ReadError(err)
                                if err.kind() == std::io::ErrorKind::NotFound =>
                            {
                                headers.push(Header::ContentType("text/html".to_string()));
                                HTTPResponse::new(
                                    opts.protocol.clone(),
                                    ResultCode::NotFound,
                                    headers,
                                    Some(error_page(ResultCode::NotFound).as_bytes().to_vec()),
                                )
                            }
                            FileError::IsADirectory => {
                                log::debug!("{} is a directory", &req.uri);
                                // Get a listing of files
                                let files = File::get_listing(&req.uri, &opts.directory);
                                log::debug!("Returning files: {}", &files.join("\n"));

                                let body = util::html::dir_listing(files);

                                headers.push(Header::ContentType("text/html".to_string()));
                                HTTPResponse::new(
                                    opts.protocol.clone(),
                                    ResultCode::OK,
                                    headers,
                                    Some(body.into()),
                                )
                            }
                            _ => {
                                headers.push(Header::ContentType("text/html".to_string()));
                                HTTPResponse::new(
                                    opts.protocol.clone(),
                                    ResultCode::InternalServerError,
                                    headers,
                                    Some(
                                        error_page(ResultCode::InternalServerError)
                                            .as_bytes()
                                            .to_vec(),
                                    ),
                                )
                            }
                        },
                    }
                }
                Method::HEAD => {
                    let f = File::try_load(&req.uri, &opts.directory);
                    match f {
                        Ok(file) => {
                            let cond_modified = req.headers.iter().find(|header| {
                                matches!(header, crate::http10::headers::Header::IfModifiedSince(_))
                            });
                            if let Some(cond_modified) = cond_modified {
                                let dt = cond_modified.date_inner();
                                if dt.is_some() && dt.unwrap() > file.get_modified() {
                                    return HTTPResponse::new(
                                        opts.protocol.clone(),
                                        ResultCode::NotModified,
                                        headers,
                                        None,
                                    );
                                }
                            }
                            headers.push(Header::ContentType(file.get_mime()));
                            headers.push(Header::ContentLength(file.get_size()));
                            headers.push(Header::LastModified(file.get_modified()));
                            HTTPResponse::new(opts.protocol.clone(), ResultCode::OK, headers, None)
                        }
                        Err(err) => match err {
                            FileError::ReadError(err)
                                if err.kind() == std::io::ErrorKind::NotFound =>
                            {
                                HTTPResponse::new(
                                    opts.protocol.clone(),
                                    ResultCode::NotFound,
                                    headers,
                                    None,
                                )
                            }
                            FileError::IsADirectory => {
                                // Get a listing of files
                                HTTPResponse::new(
                                    opts.protocol.clone(),
                                    ResultCode::NotFound,
                                    headers,
                                    None,
                                )
                            }
                            _ => HTTPResponse::new(
                                opts.protocol.clone(),
                                ResultCode::InternalServerError,
                                headers,
                                None,
                            ),
                        },
                    }
                }
                Method::POST => {
                    log::info!(
                        "Received POST Data {:?}",
                        str::from_utf8(req.body.unwrap().as_ref())
                    );
                    headers.push(Header::ContentType("text/html".to_string()));
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
            let remote: String = match stream.peer_addr() {
                Ok(addr) => addr.to_string(),
                Err(_) => "Invalid Address".to_string(),
            };
            let mut request: Vec<u8> = Vec::new();
            let mut buf = [0u8; 4096];
            while HTTPRequest::try_from(&request).is_err() {
                match stream.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        request.append(buf[..n].to_vec().as_mut());
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => (),
                    Err(_) => break,
                }
            }
            let request = HTTPRequest::try_from(&request).unwrap();
            let headline = format!(
                "{} {} {}",
                Into::<String>::into(request.method),
                request.uri,
                request.version
            );
            let user_agent = request
                .headers
                .iter()
                .find(|header| matches!(header, crate::http10::headers::Header::UserAgent(_)));
            let user_agent = if user_agent.is_some() {
                user_agent.unwrap().str_inner().unwrap()
            } else {
                "-".to_string()
            };
            let req_headers = request
                .headers
                .clone()
                .iter()
                .map(|header| header.to_string())
                .collect::<Vec<String>>()
                .join("\n");

            let mut resp = handler(request, opts);
            let code = Into::<usize>::into(resp.status);
            let content_len = resp
                .headers
                .iter()
                .find(|header| matches!(header, Header::ContentLength(_)));
            let content_len = if content_len.is_some() {
                content_len.unwrap().num_inner().unwrap()
            } else {
                0
            };
            let resp_headers = resp
                .headers
                .clone()
                .iter()
                .map(|header| header.to_string())
                .collect::<Vec<String>>()
                .join("\n");
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
                Some(handl) => HTTPServer {
                    class,
                    opts,
                    handler: handl,
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
