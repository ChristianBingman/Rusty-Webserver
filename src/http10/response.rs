use core::str;

use super::{headers::Headers, result_codes::ResultCode};

#[derive(Debug, Clone)]
pub struct HTTPResponse {
    pub version: String,
    pub status: ResultCode,
    pub headers: Headers,
    pub body: Option<Vec<u8>>,
}

impl HTTPResponse {
    pub fn new(
        version: impl Into<String>,
        status: ResultCode,
        headers: Headers,
        body: Option<Vec<u8>>,
    ) -> Self {
        HTTPResponse {
            version: version.into(),
            status,
            headers,
            body,
        }
    }

    pub fn as_bytes(&mut self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut response: String =
            format!("{} {}\r\n", self.version, Into::<String>::into(self.status));
        response += &self.headers.to_string();
        bytes.append(&mut response.as_bytes().to_vec());
        if let Some(body) = &mut self.body {
            bytes.append(body);
        }
        bytes
    }
}

impl std::fmt::Display for HTTPResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {}\r\n",
            self.version,
            Into::<String>::into(self.status)
        ))?;
        f.write_str(&self.headers.to_string())?;
        if let Some(body) = &self.body {
            f.write_str(str::from_utf8(body).map_err(|_| std::fmt::Error)?)?;
        }
        Ok(())
    }
}
