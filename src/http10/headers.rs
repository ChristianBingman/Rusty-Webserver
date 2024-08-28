use std::collections::{HashMap, HashSet};

use chrono::{DateTime, FixedOffset};

use super::{
    content_codings::ContentEncoding,
    methods::{InvalidMethodErr, Method},
};

#[derive(Debug)]
pub enum HeaderErr {
    InvalidField(String),
}

impl std::fmt::Display for HeaderErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidField(err) => f.write_fmt(format_args!("Invalid Field: {}", err)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Headers {
    headers: HashMap<HeaderVariant, Header>,
    extra: Vec<Header>,
}

impl Headers {
    pub fn new() -> Headers {
        Headers {
            headers: HashMap::new(),
            extra: Vec::new(),
        }
    }
    pub fn get(&self, header: HeaderVariant) -> Option<Header> {
        self.headers.get(&header).cloned()
    }

    pub fn set(&mut self, header: Header) {
        match header {
            Header::Accept(_) => {
                self.headers.insert(HeaderVariant::Accept, header);
            }
            Header::AcceptEncoding(_) => {
                self.headers.insert(HeaderVariant::AcceptEncoding, header);
            }
            Header::Allow(_) => {
                self.headers.insert(HeaderVariant::Allow, header);
            }
            Header::Authorization(_) => {
                self.headers.insert(HeaderVariant::Authorization, header);
            }
            Header::ContentEncoding(_) => {
                self.headers.insert(HeaderVariant::ContentEncoding, header);
            }
            Header::ContentLength(_) => {
                self.headers.insert(HeaderVariant::ContentLength, header);
            }
            Header::ContentType(_) => {
                self.headers.insert(HeaderVariant::ContentType, header);
            }
            Header::Date(_) => {
                self.headers.insert(HeaderVariant::Date, header);
            }
            Header::Expires(_) => {
                self.headers.insert(HeaderVariant::Expires, header);
            }
            Header::From(_) => {
                self.headers.insert(HeaderVariant::From, header);
            }
            Header::Generic(_) => {
                self.extra.push(header);
            }
            Header::Host(_) => {
                self.headers.insert(HeaderVariant::Host, header);
            }
            Header::IfModifiedSince(_) => {
                self.headers.insert(HeaderVariant::IfModifiedSince, header);
            }
            Header::LastModified(_) => {
                self.headers.insert(HeaderVariant::LastModified, header);
            }
            Header::Location(_) => {
                self.headers.insert(HeaderVariant::Location, header);
            }
            Header::Pragma(_) => {
                self.headers.insert(HeaderVariant::Pragma, header);
            }
            Header::Referer(_) => {
                self.headers.insert(HeaderVariant::Referer, header);
            }
            Header::Server(_) => {
                self.headers.insert(HeaderVariant::Server, header);
            }
            Header::UserAgent(_) => {
                self.headers.insert(HeaderVariant::UserAgent, header);
            }
            Header::WWWAuthenticate(_) => {
                self.headers.insert(HeaderVariant::WWWAuthenticate, header);
            }
        }
    }

    pub fn get_generic(&self, _header: &str) -> Option<String> {
        unimplemented!();
    }
}

impl std::fmt::Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for header in self.headers.values() {
            f.write_str(header.to_string().as_str())?;
            f.write_str("\r\n")?;
        }
        for header in &self.extra {
            f.write_str(header.to_string().as_str())?;
            f.write_str("\r\n")?;
        }
        f.write_str("\r\n")?;
        Ok(())
    }
}

impl TryFrom<&str> for Headers {
    type Error = HeaderErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lines = value.trim_end().split("\r\n");
        let mut hm: HashMap<HeaderVariant, Header> = HashMap::new();
        let mut ex = Vec::new();
        for line in lines {
            let (k, mut v) = match Header::try_from(line)? {
                Header::Accept(val) => (HeaderVariant::Accept, Header::Accept(val)),
                Header::AcceptEncoding(val) => {
                    (HeaderVariant::AcceptEncoding, Header::AcceptEncoding(val))
                }
                Header::Allow(val) => (HeaderVariant::Allow, Header::Allow(val)),
                Header::Authorization(val) => {
                    (HeaderVariant::Authorization, Header::Authorization(val))
                }
                Header::ContentEncoding(val) => {
                    (HeaderVariant::ContentEncoding, Header::ContentEncoding(val))
                }
                Header::ContentLength(val) => {
                    (HeaderVariant::ContentLength, Header::ContentLength(val))
                }
                Header::ContentType(val) => (HeaderVariant::ContentType, Header::ContentType(val)),
                Header::Date(val) => (HeaderVariant::Date, Header::Date(val)),
                Header::Expires(val) => (HeaderVariant::Expires, Header::Expires(val)),
                Header::From(val) => (HeaderVariant::From, Header::From(val)),
                Header::Generic(val) => (HeaderVariant::Generic, Header::Generic(val)),
                Header::Host(val) => (HeaderVariant::Host, Header::Host(val)),
                Header::IfModifiedSince(val) => {
                    (HeaderVariant::IfModifiedSince, Header::IfModifiedSince(val))
                }
                Header::LastModified(val) => {
                    (HeaderVariant::LastModified, Header::LastModified(val))
                }
                Header::Location(val) => (HeaderVariant::Location, Header::Location(val)),
                Header::Pragma(val) => (HeaderVariant::Pragma, Header::Pragma(val)),
                Header::Referer(val) => (HeaderVariant::Referer, Header::Referer(val)),
                Header::Server(val) => (HeaderVariant::Server, Header::Server(val)),
                Header::UserAgent(val) => (HeaderVariant::UserAgent, Header::UserAgent(val)),
                Header::WWWAuthenticate(val) => {
                    (HeaderVariant::WWWAuthenticate, Header::WWWAuthenticate(val))
                }
            };
            if k == HeaderVariant::Generic {
                ex.push(v);
                continue;
            }
            if let Some(value) = hm.get(&k) {
                // Merge them if possible, otherwise error
                match value {
                    Header::Accept(inner) => {
                        let Header::Accept(inner_v) = v else {
                            return Err(HeaderErr::InvalidField(
                                "Error merging Accept header".to_string(),
                            ));
                        };
                        v = Header::Accept(inner_v + inner);
                    }
                    Header::AcceptEncoding(encodings) => {
                        let Header::AcceptEncoding(ex_enc) = v else {
                            return Err(HeaderErr::InvalidField(
                                "Error merging Accept header".to_string(),
                            ));
                        };
                        let mut encs = encodings.clone();
                        encs.append(&mut ex_enc.clone());
                        let hs: HashSet<ContentEncoding> = HashSet::from_iter(encs.iter().cloned());
                        v = Header::AcceptEncoding(hs.into_iter().collect());
                    }
                    Header::Allow(methods) => {
                        let Header::Allow(ex_met) = v else {
                            return Err(HeaderErr::InvalidField(
                                "Error merging Accept header".to_string(),
                            ));
                        };
                        let mut mets = methods.clone();
                        mets.append(&mut ex_met.clone());
                        let hs: HashSet<Method> = HashSet::from_iter(mets.iter().cloned());
                        v = Header::Allow(hs.into_iter().collect());
                    }
                    _ => {
                        return Err(HeaderErr::InvalidField(format!(
                            "Cannot merge multiple of field {}, {}",
                            v, value
                        )))
                    }
                }
            }
            hm.insert(k, v);
        }
        Ok(Headers {
            headers: hm,
            extra: ex,
        })
    }
}

impl TryFrom<String> for Headers {
    type Error = HeaderErr;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Headers::try_from(value.as_ref())
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum HeaderVariant {
    Accept,
    AcceptEncoding,
    Allow,
    Authorization,
    ContentEncoding,
    ContentLength,
    ContentType,
    Date,
    Expires,
    From,
    Generic,
    Host,
    IfModifiedSince,
    LastModified,
    Location,
    Pragma,
    Referer,
    Server,
    UserAgent,
    WWWAuthenticate,
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(dead_code)]
pub enum Header {
    Accept(String),
    AcceptEncoding(Vec<ContentEncoding>),
    Allow(Vec<Method>),
    Authorization(String),
    ContentEncoding(ContentEncoding),
    ContentLength(usize),
    ContentType(String),
    Date(DateTime<FixedOffset>),
    Expires(DateTime<FixedOffset>),
    From(String),
    Generic((String, String)),
    Host(String),
    IfModifiedSince(DateTime<FixedOffset>),
    LastModified(DateTime<FixedOffset>),
    Location(String),
    Pragma(String),
    Referer(String),
    Server(String),
    UserAgent(String),
    WWWAuthenticate(String),
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Header::Accept(suf) => f.write_fmt(format_args!("Accept: {}", suf)),
            Header::AcceptEncoding(encodings) => f.write_fmt(format_args!(
                "Accept-Encoding: {}",
                encodings
                    .iter()
                    .map(|coding| coding.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )),
            Header::Allow(methods) => f.write_fmt(format_args!(
                "Allow: {}",
                methods
                    .iter()
                    .map(|method| Into::<String>::into(*method))
                    .collect::<Vec<String>>()
                    .join(",")
            )),
            Header::Authorization(suf) => f.write_fmt(format_args!("Authorization: {}", suf)),
            Header::ContentEncoding(encoding) => {
                f.write_fmt(format_args!("Content-Encoding: {}", encoding))
            }
            Header::ContentLength(len) => f.write_fmt(format_args!("Content-Length: {}", len)),
            Header::ContentType(mime) => f.write_fmt(format_args!("Content-Type: {}", mime)),
            Header::Date(date) => f.write_fmt(format_args!("Date: {}", date.to_rfc2822())),
            Header::Expires(date) => f.write_fmt(format_args!("Expires: {}", date.to_rfc2822())),
            Header::From(suf) => f.write_fmt(format_args!("From: {}", suf)),
            Header::Generic((pref, suf)) => f.write_fmt(format_args!("{}: {}", pref, suf)),
            Header::Host(suf) => f.write_fmt(format_args!("Host: {}", suf)),
            Header::IfModifiedSince(date) => {
                f.write_fmt(format_args!("If-Modified-Since: {}", date.to_rfc2822()))
            }
            Header::LastModified(date) => {
                f.write_fmt(format_args!("Last-Modified: {}", date.to_rfc2822()))
            }
            Header::Location(suf) => f.write_fmt(format_args!("Location: {}", suf)),
            Header::Pragma(suf) => f.write_fmt(format_args!("Pragma: {}", suf)),
            Header::Referer(suf) => f.write_fmt(format_args!("Referer: {}", suf)),
            Header::Server(suf) => f.write_fmt(format_args!("Server: {}", suf)),
            Header::UserAgent(suf) => f.write_fmt(format_args!("User-Agent: {}", suf)),
            Header::WWWAuthenticate(suf) => f.write_fmt(format_args!("WWW-Authenticate: {}", suf)),
        }
    }
}

impl TryFrom<&str> for Header {
    type Error = HeaderErr;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Header::try_from(value.to_string())
    }
}

impl TryFrom<String> for Header {
    type Error = HeaderErr;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some((field, suffix)) = value.split_once(':') {
            let suf = suffix.trim();
            match field {
                "Accept" => Ok(Self::Accept(suf.to_string())),
                "Accept-Encoding" => {
                    let codings = suf
                        .split(',')
                        .map(|coding| ContentEncoding::try_from(coding.trim()))
                        .filter_map(|coding| coding.ok())
                        .collect::<Vec<ContentEncoding>>();
                    if codings.is_empty() {
                        Err(Self::Error::InvalidField(format!(
                            "No supported formats in: {}",
                            suf
                        )))
                    } else {
                        dbg!(Ok(Self::AcceptEncoding(codings)))
                    }
                }
                "Allow" => {
                    let methods = suf
                        .split(',')
                        .map(|method| Method::try_from(method))
                        .collect::<Result<Vec<Method>, InvalidMethodErr>>()
                        .map_err(|_| {
                            Self::Error::InvalidField(format!("Unable to parse suffix {}", suf))
                        });
                    Ok(Self::Allow(methods?))
                }
                "Authorization" => Ok(Self::Authorization(suf.to_string())),
                "Content-Encoding" => Ok(Self::ContentEncoding(
                    ContentEncoding::try_from(suf).map_err(|_| {
                        Self::Error::InvalidField(format!("Unable to parse suffix {}", suf))
                    })?,
                )),
                "Content-Length" => {
                    Ok(Self::ContentLength(suf.parse::<usize>().map_err(|_| {
                        Self::Error::InvalidField(format!("Unable to parse suffix {}", suf))
                    })?))
                }
                "Content-Type" => Ok(Self::ContentType(suf.to_string())),
                "Date" => Ok(Self::Date(DateTime::parse_from_rfc2822(suf).map_err(
                    |_| Self::Error::InvalidField(format!("Unable to parse suffix {}", suf)),
                )?)),
                "Expires" => Ok(Self::Expires(DateTime::parse_from_rfc2822(suf).map_err(
                    |_| Self::Error::InvalidField(format!("Unable to parse suffix {}", suf)),
                )?)),
                "From" => Ok(Self::From(suf.to_string())),
                "Host" => Ok(Self::Host(suf.to_string())),
                "If-Modified-Since" => Ok(Self::IfModifiedSince(
                    DateTime::parse_from_rfc2822(suf).map_err(|_| {
                        Self::Error::InvalidField(format!("Unable to parse suffix {}", suf))
                    })?,
                )),
                "Last-Modified" => Ok(Self::LastModified(
                    DateTime::parse_from_rfc2822(suf).map_err(|_| {
                        Self::Error::InvalidField(format!("Unable to parse suffix {}", suf))
                    })?,
                )),
                "Location" => Ok(Self::Location(suf.to_string())),
                "Pragma" => Ok(Self::Pragma(suf.to_string())),
                "Referer" => Ok(Self::Referer(suf.to_string())),
                "Server" => Ok(Self::Server(suf.to_string())),
                "User-Agent" => Ok(Self::UserAgent(suf.to_string())),
                "WWW-Authenticate" => Ok(Self::WWWAuthenticate(suf.to_string())),
                _ => Ok(Self::Generic((field.to_string(), suf.to_string()))),
            }
        } else {
            Err(HeaderErr::InvalidField(format!(
                "Unable to parse field {}",
                value
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_authorization_from_string() {
        assert_eq!(
            Header::try_from("Authorization: Bearer".to_string()).unwrap(),
            Header::Authorization("Bearer".to_string())
        );
    }

    #[test]
    fn converts_if_modified_since_from_string() {
        assert_eq!(
            Header::try_from("If-Modified-Since: Tue, 15 Nov 1994 08:12:31 GMT".to_string())
                .unwrap(),
            Header::IfModifiedSince(
                DateTime::parse_from_rfc2822("Tue, 15 Nov 1994 08:12:31 GMT").unwrap()
            )
        );
    }

    #[test]
    fn builds_header_list_from_string() {
        let headers_str = "Content-Type: text/html\r\n\
        Accept: */*\r\n\
        Server: test-server/1.0\r\n\
        Host: www.mywebserver.com\r\n\r\n";

        let headers = Headers {
            headers: HashMap::from([
                (
                    HeaderVariant::ContentType,
                    Header::ContentType("text/html".to_string()),
                ),
                (HeaderVariant::Accept, Header::Accept("*/*".to_string())),
                (
                    HeaderVariant::Server,
                    Header::Server("test-server/1.0".to_string()),
                ),
                (
                    HeaderVariant::Host,
                    Header::Host("www.mywebserver.com".to_string()),
                ),
            ]),
            extra: vec![],
        };

        assert_eq!(Headers::try_from(headers_str).unwrap(), headers);
    }

    #[test]
    fn builds_string_from_header_list() {
        let headers_str = "Content-Type: text/html\r\n\
        Accept: */*\r\n\
        Server: test-server/1.0\r\n\
        Host: www.mywebserver.com\r\n\r\n"
            .to_string();

        let headers = Headers {
            headers: HashMap::from([
                (
                    HeaderVariant::ContentType,
                    Header::ContentType("text/html".to_string()),
                ),
                (HeaderVariant::Accept, Header::Accept("*/*".to_string())),
                (
                    HeaderVariant::Server,
                    Header::Server("test-server/1.0".to_string()),
                ),
                (
                    HeaderVariant::Host,
                    Header::Host("www.mywebserver.com".to_string()),
                ),
            ]),
            extra: vec![],
        };

        assert_eq!(headers.to_string(), headers_str);
    }

    #[test]
    fn merges_valid_headers() {
        let headers_str = "Content-Type: text/html\r\n\
        Accept: */*\r\n\
        Server: test-server/1.0\r\n\
        Host: www.mywebserver.com\r\n\
        Accept-Encoding: deflate, gzip\r\n\r\n"
            .to_string();

        let headers = Headers {
            headers: HashMap::from([
                (
                    HeaderVariant::ContentType,
                    Header::ContentType("text/html".to_string()),
                ),
                (HeaderVariant::Accept, Header::Accept("*/*".to_string())),
                (
                    HeaderVariant::Server,
                    Header::Server("test-server/1.0".to_string()),
                ),
                (
                    HeaderVariant::Host,
                    Header::Host("www.mywebserver.com".to_string()),
                ),
                (
                    HeaderVariant::AcceptEncoding,
                    Header::AcceptEncoding(vec![ContentEncoding::DEFLATE, ContentEncoding::GZIP]),
                ),
            ]),
            extra: vec![],
        };

        assert_eq!(headers.to_string(), headers_str);
    }
}
