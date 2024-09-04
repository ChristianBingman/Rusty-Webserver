use super::headers::{Header, HeaderVariant, Headers};
use super::methods::Method;

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ReqError {
    ParseError(String),
    ContentLenError,
    InvalidMethodErr,
    InvalidHTTPVerError,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HTTPRequest {
    pub method: Method,
    pub uri: String,
    pub version: String,
    pub headers: Headers,
    pub body: Option<Vec<u8>>,
}

fn parse_request_line(line: impl Into<String>) -> Result<(Method, String, String), ReqError> {
    let line_s: String = line.into();
    let mut spl = line_s.split(" ");
    let method: Method;
    let uri: String;
    let version: String;

    if let Some(m) = spl.next() {
        method = Method::try_from(m).map_err(|_| ReqError::InvalidMethodErr)?;
    } else {
        return Err(ReqError::ParseError("Invalid header line".to_string()));
    }

    if let Some(u) = spl.next() {
        uri = u.to_string();
    } else {
        return Err(ReqError::ParseError("Invalid header line".to_string()));
    }

    if let Some(v) = spl.next() {
        version = v.to_string();
    } else {
        return Err(ReqError::ParseError("Invalid header line".to_string()));
    }
    Ok((method, uri, version))
}

// Convert from a string of bytes
impl TryFrom<&Vec<u8>> for HTTPRequest {
    type Error = ReqError;
    fn try_from(req: &Vec<u8>) -> Result<Self, Self::Error> {
        let spl_ind = &req.windows(4).position(|bytes| bytes == &[13, 10, 13, 10]);
        if spl_ind.is_none() {
            // Fail if we can't find \r\n\r\n
            return Err(Self::Error::ContentLenError);
        }
        let (header_lines, body) = &req.split_at(spl_ind.unwrap() + 4);
        let header_lines = header_lines.to_vec();
        let header_lines_str = match std::str::from_utf8(&header_lines) {
            Ok(lines) => lines,
            Err(err) => {
                log::debug!("Received invalid bytes {}", err);
                return Err(Self::Error::ParseError("Invalid header encoding".into()));
            }
        };
        let headers = header_lines_str.split_once("\r\n");
        if headers.is_none() {
            return Err(Self::Error::ParseError(
                "Unable to split header line".to_string(),
            ));
        }
        let headers = headers.unwrap();
        let (method, uri, version) = parse_request_line(headers.0)?;

        // We are only supporting 1.0, but 1.1 should be compatible for the most part
        if version != "HTTP/1.0" && version != "HTTP/1.1" {
            return Err(Self::Error::InvalidHTTPVerError);
        }

        let headers: Headers = Headers::try_from(headers.1).map_err(|err| {
            Self::Error::ParseError(format!("Unable to parse request line: {}", err))
        })?;

        if let Some(len) = headers.get(HeaderVariant::ContentLength) {
            let Header::ContentLength(len) = len else {
                return Err(Self::Error::ContentLenError);
            };
            if len != body.len() {
                return Err(Self::Error::ContentLenError);
            }
        }

        Ok(HTTPRequest {
            method,
            uri,
            version,
            headers,
            body: if body.len() != 0 {
                Some(body.to_vec())
            } else {
                None
            },
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_valid_request() {
        let request_buf = "GET / HTTP/1.0\r\n\
        Host: webserver.christianbingman.com\r\n\
        User-Agent: rusty-client/1.0\r\n\
        Accept: */*\r\n\r\n"
            .as_bytes()
            .to_vec();

        let req = HTTPRequest::try_from(&request_buf).unwrap();
        let mut headers = Headers::new();
        headers.set(Header::Host("webserver.christianbingman.com".into()));
        headers.set(Header::UserAgent("rusty-client/1.0".into()));
        headers.set(Header::Accept("*/*".into()));

        assert_eq!(req.method, Method::GET);
        assert_eq!(req.uri, "/");
        assert_eq!(req.version, "HTTP/1.0");
        assert_eq!(req.headers, headers);
    }

    #[test]
    fn test_fail_invalid_request() {
        let request_buf = "GET HTTP/1.0\r\n\
        Host: webserver.christianbingman.com\r\n\
        User-Agent: rusty-client/1.0\r\n\
        Accept: */*\r\n\r\n"
            .as_bytes()
            .to_vec();

        assert_eq!(
            HTTPRequest::try_from(&request_buf).unwrap_err(),
            ReqError::ParseError("Invalid header line".into())
        );
    }

    #[test]
    fn test_missing_header_delimiter() {
        let request_buf = "GET HTTP/1.0\r\n\
        Host: webserver.christianbingman.com\r\n\
        User-Agent: rusty-client/1.0\r\n\
        Accept: */*\r\n"
            .as_bytes()
            .to_vec();

        assert_eq!(
            HTTPRequest::try_from(&request_buf).unwrap_err(),
            ReqError::ContentLenError
        );
    }

    #[test]
    fn test_invalid_http_ver() {
        let request_buf = "GET / HTTP/2.0\r\n\
        Host: webserver.christianbingman.com\r\n\
        User-Agent: rusty-client/1.0\r\n\
        Accept: */*\r\n\r\n"
            .as_bytes()
            .to_vec();

        assert_eq!(
            HTTPRequest::try_from(&request_buf).unwrap_err(),
            ReqError::InvalidHTTPVerError
        );
    }

    #[test]
    fn test_invalid_header_charset() {
        let request_buf = b"GET HTTP/1.0\r\n\
        Host: webserver.christianbingman.com\r\n\
        User-Agent: rusty-client/1.0\r\n\
        Accept: */*\xc3\x28\r\n\r\n"
            .to_vec();

        assert_eq!(
            HTTPRequest::try_from(&request_buf).unwrap_err(),
            ReqError::ParseError("Invalid header encoding".into())
        );
    }
}
