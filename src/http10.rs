use core::str;
use std::io;

use chrono::Utc;
use headers::Header;
use methods::Method;
use request::HTTPRequest;
use response::HTTPResponse;
use result_codes::ResultCode;

use crate::file;

pub mod result_codes;
pub mod methods;
pub mod headers;
pub mod content_codings;
pub mod content_types;
pub mod response;
pub mod request;

pub fn handle_request(buf: Vec<u8>) -> HTTPResponse {
    let mut headers: Vec<Header> = vec![
        Header::Date(Utc::now().into()),
        Header::Server("Rusty Webserver".to_string()),
    ];
    let req = HTTPRequest::try_from(&buf);
    match req {
        Err(_) => return HTTPResponse::new("HTTP/1.0", ResultCode::BadRequest, headers, Some(Into::<String>::into(ResultCode::BadRequest).as_bytes().to_vec())),
        Ok(_) => ()
    }

    match req.as_ref().unwrap().method {
        Method::GET => {
            let f = file::File::try_load(req.unwrap().uri);
            match f {
                Ok(file) => {
                    headers.push(Header::ContentType(file.get_mime()));
                    headers.push(Header::ContentLength(file.get_size()));
                    HTTPResponse::new("HTTP/1.0".to_string(), result_codes::ResultCode::OK, headers, Some(file.get_content()))
                },
                Err(err) => match err {
                    file::FileError::ReadError(err) if err.kind() == io::ErrorKind::NotFound => {
                        headers.push(Header::ContentType("text/plain".to_string()));
                        HTTPResponse::new("HTTP/1.0".to_string(), result_codes::ResultCode::NotFound, headers, Some(Into::<String>::into(result_codes::ResultCode::NotFound).as_bytes().to_vec()))
                    }
                    _ => {
                        headers.push(Header::ContentType("text/plain".to_string()));
                        HTTPResponse::new("HTTP/1.0".to_string(), result_codes::ResultCode::InternalServerError, headers, Some(Into::<String>::into(result_codes::ResultCode::InternalServerError).as_bytes().to_vec()))
                    }
                }
            }
        },
        Method::HEAD => {
            HTTPResponse::new("HTTP/1.0".to_string(), result_codes::ResultCode::OK, headers, None)
        },
        Method::POST => {
            log::info!("Received POST Data {:?}", str::from_utf8(req.unwrap().body.unwrap().as_ref()));
            headers.push(Header::ContentType("text/plain".to_string()));
            HTTPResponse::new("HTTP/1.0".to_string(), result_codes::ResultCode::NotImplemented, headers, Some(Into::<String>::into(result_codes::ResultCode::NotImplemented).as_bytes().to_vec()))
        }
    }
}
