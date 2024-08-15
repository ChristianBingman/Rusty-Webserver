use super::methods::Method;
use super::headers::Header;

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
    pub headers: Vec<Header>,
    pub body: Option<Vec<u8>>,
}

fn parse_request_line(line: impl Into<String>) -> Result<(Method, String, String), ReqError>{
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
            return Err(Self::Error::ParseError("Unable to find header delimiter".to_string()));
        }
        let (header_lines, body) = &req.split_at(spl_ind.unwrap()+4);
        let header_lines_str = String::from_utf8(header_lines.to_vec()).map_err(|_| Self::Error::ParseError("Headers aren't valid UTF-8".to_string()))?;
        let mut headers = header_lines_str.strip_suffix("\r\n\r\n").unwrap().split("\r\n");
        let (method, uri, version) = headers.next()
            .map(|reqline| parse_request_line(reqline))
            .ok_or(Self::Error::ParseError("No request line".to_string()))??;

        let headers: Vec<Header> = headers.map(|line| {
            Header::try_from(line.to_string()).map_err(|err| Self::Error::ParseError(err.to_string()))
        }).collect::<Result<Vec<Header>, ReqError>>()?;

        let len: Vec<&usize> = headers.iter().filter_map(|e| match e { Header::ContentLength(len) => Some(len), _ => None, }).collect();
        if let Some(len) = len.get(0) {
            if **len != body.len() {
                return Err(Self::Error::ContentLenError);
            }
        }

        Ok(HTTPRequest { method, uri, version, headers, body: if body.len() != 0 { Some(body.to_vec()) } else {None} })
    }
}
