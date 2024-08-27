#[derive(Debug)]
pub struct InvalidContentEncodingErr;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ContentEncoding {
    GZIP,
    DEFLATE,
    TOKEN,
}

impl TryFrom<String> for ContentEncoding {
    type Error = InvalidContentEncodingErr;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "gzip" | "x-gzip" => Ok(Self::GZIP),
            "deflate" => Ok(Self::DEFLATE),
            "token" => Ok(Self::TOKEN),
            _ => Err(InvalidContentEncodingErr),
        }
    }
}

impl TryFrom<&str> for ContentEncoding {
    type Error = InvalidContentEncodingErr;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "gzip" | "x-gzip" => Ok(Self::GZIP),
            "deflate" => Ok(Self::DEFLATE),
            "token" => Ok(Self::TOKEN),
            _ => Err(InvalidContentEncodingErr),
        }
    }
}

impl std::fmt::Display for ContentEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ContentEncoding::GZIP => f.write_str("gzip"),
            ContentEncoding::DEFLATE => f.write_str("deflate"),
            ContentEncoding::TOKEN => f.write_str("token"),
        }
    }
}
