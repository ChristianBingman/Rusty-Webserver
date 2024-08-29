use base64::Engine;
use chrono::Utc;

use crate::{
    file::{File, FileError},
    http10::{
        content_codings::ContentEncoding,
        headers::{Header, HeaderVariant, Headers},
        request::HTTPRequest,
        response::HTTPResponse,
        result_codes::ResultCode,
    },
    util::html::{dir_listing, error_page},
    Auth, Opts,
};

#[derive(Debug)]
pub struct AuthError {}

pub fn get_handler(req: &HTTPRequest, opts: &Opts) -> HTTPResponse {
    let mut headers = Headers::new();
    headers.set(Header::Date(Utc::now().into()));
    headers.set(Header::Server("Rusty Webserver".to_string()));

    let f = File::try_load(&req.uri, &opts.directory);
    match f {
        Ok(mut file) => {
            let cond_modified = req.headers.get(HeaderVariant::IfModifiedSince);
            if let Some(cond_modified) = cond_modified {
                let Header::IfModifiedSince(dt) = cond_modified else {
                    unimplemented!()
                };
                if dt > file.get_modified() {
                    return HTTPResponse::new(
                        opts.protocol.clone(),
                        ResultCode::NotModified,
                        headers,
                        None,
                    );
                }
            }
            let encodings = req.headers.get(HeaderVariant::ContentEncoding);

            if let Some(encodings) = encodings {
                let Header::AcceptEncoding(encodings) = encodings else {
                    unimplemented!()
                };
                if encodings
                    .iter()
                    .find(|encoding| **encoding == ContentEncoding::TOKEN)
                    .is_none()
                {
                    headers.set(Header::ContentEncoding(encodings[0].clone()));
                    file = file.compress(&encodings[0], opts.ratio).unwrap();
                }
            }
            headers.set(Header::ContentType(file.get_mime()));
            headers.set(Header::ContentLength(file.get_size()));
            headers.set(Header::LastModified(file.get_modified()));
            HTTPResponse::new(
                opts.protocol.clone(),
                ResultCode::OK,
                headers,
                Some(file.get_content()),
            )
        }
        Err(err) => match err {
            FileError::ReadError(err) if err.kind() == std::io::ErrorKind::NotFound => {
                headers.set(Header::ContentType("text/html".to_string()));
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

                let body = dir_listing(files);

                headers.set(Header::ContentType("text/html".to_string()));
                HTTPResponse::new(
                    opts.protocol.clone(),
                    ResultCode::OK,
                    headers,
                    Some(body.into()),
                )
            }
            _ => {
                headers.set(Header::ContentType("text/html".to_string()));
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

pub fn basic_auth(req: &HTTPRequest, auth: &Auth) -> Result<(), AuthError> {
    let auth_header = req.headers.get(HeaderVariant::Authorization);

    if let Some(auth_header) = auth_header {
        let Header::Authorization(inner) = auth_header else {
            return Err(AuthError {});
        };
        let mut inner = inner.split(' ');
        let typ = inner.next();
        let token = inner.next();
        if typ.is_none() || token.is_none() || typ.unwrap() != "Basic" {
            return Err(AuthError {});
        }
        if base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", auth.username, auth.password).as_bytes())
            == token.unwrap()
        {
            return Ok(());
        }
        log::debug!(
            "Auth failure: {} does not match {}",
            base64::engine::general_purpose::STANDARD
                .encode(format!("{}:{}", auth.username, auth.password)),
            token.unwrap()
        );
    }
    Err(AuthError {})
}

#[cfg(test)]
mod test {
    use crate::http10::headers::{Header, Headers};

    use super::*;

    #[test]
    fn test_basic_auth_success() {
        let mut headers = Headers::new();
        headers.set(Header::Authorization(
            "Basic YWRtaW46cGFzc3dvcmQ=".to_string(),
        ));
        let req = HTTPRequest {
            method: crate::http10::methods::Method::GET,
            uri: "/".to_string(),
            version: "HTTP/1.0".to_string(),
            headers,
            body: None,
        };
        let auth = Auth {
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        assert!(basic_auth(&req, &auth).is_ok());
    }

    #[test]
    fn test_basic_auth_failure() {
        let req = HTTPRequest {
            method: crate::http10::methods::Method::GET,
            uri: "/".to_string(),
            version: "HTTP/1.0".to_string(),
            headers: Headers::new(),
            body: None,
        };
        let auth = Auth {
            username: "admin".to_string(),
            password: "password".to_string(),
        };
        assert!(basic_auth(&req, &auth).is_err());
    }

    #[test]
    fn test_basic_auth_incorrect_basic() {
        let mut headers = Headers::new();
        headers.set(Header::Authorization(
            "Basic YWRtaW46cGFzc3dvcmQx".to_string(),
        ));
        let req = HTTPRequest {
            method: crate::http10::methods::Method::GET,
            uri: "/".to_string(),
            version: "HTTP/1.0".to_string(),
            headers,
            body: None,
        };
        let auth = Auth {
            username: "admin".to_string(),
            password: "password".to_string(),
        };
        assert!(basic_auth(&req, &auth).is_err());
    }
}
