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
#[allow(dead_code)]
pub enum Header {
    Accept(String),
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

impl Header {
    pub fn str_inner(&self) -> Option<String> {
        match self {
            Self::Authorization(inner) => Some(inner.to_string()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Header::Accept(suf) => f.write_fmt(format_args!("Accept: {}", suf)),
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

impl TryFrom<String> for Header {
    type Error = HeaderErr;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some((field, suffix)) = value.split_once(':') {
            let suf = suffix.trim();
            match field {
                "Accept" => Ok(Self::Accept(suf.to_string())),
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
}
