use super::convert_iso_8859_1_to_utf8;
use super::headers::{Header, HeaderVariant, Headers};
use super::methods::Method;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ReqError {
    ParseError(String),
    ContentLenError,
    InvalidMethodErr,
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
            return Err(Self::Error::ParseError(
                "Unable to find header delimiter".to_string(),
            ));
        }
        let (header_lines, body) = &req.split_at(spl_ind.unwrap() + 4);
        let header_lines_str = convert_iso_8859_1_to_utf8(&header_lines.to_vec());
        let headers = header_lines_str.split_once("\r\n");
        if headers.is_none() {
            return Err(Self::Error::ParseError(
                "Unable to split header line".to_string(),
            ));
        }
        let headers = headers.unwrap();
        let (method, uri, version) = parse_request_line(headers.0)
            .map_err(|_| Self::Error::ParseError("No request line".to_string()))?;

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
