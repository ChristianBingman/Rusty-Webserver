pub struct InvalidContentEncodingErr;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ContentEncoding {
    GZIP,
    COMPRESS,
    TOKEN
}

impl TryFrom<String> for ContentEncoding {
    type Error = InvalidContentEncodingErr;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "gzip" | "x-gzip" => Ok(Self::GZIP),
            "compress" | "x-compress" => Ok(Self::COMPRESS),
            "token" => Ok(Self::TOKEN),
            _ => Err(InvalidContentEncodingErr)
        }
    }
}

impl TryFrom<&str> for ContentEncoding {
    type Error = InvalidContentEncodingErr;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "gzip" | "x-gzip" => Ok(Self::GZIP),
            "compress" | "x-compress" => Ok(Self::COMPRESS),
            "token" => Ok(Self::TOKEN),
            _ => Err(InvalidContentEncodingErr)
        }
    }
}

impl std::fmt::Display for ContentEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ContentEncoding::GZIP => f.write_str("x-gzip"),
            ContentEncoding::COMPRESS => f.write_str("x-compress"),
            ContentEncoding::TOKEN => f.write_str("token"),
        }
    }
}
