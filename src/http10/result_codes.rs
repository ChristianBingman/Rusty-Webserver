pub struct ResultCodeConversionError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ResultCode {
    OK,
    Created,
    Accepted,
    NoContent,
    MultipleChoices,
    MovedPermanently,
    MovedTemporarily,
    NotModified,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable
}

impl Into<String> for ResultCode {
    fn into(self) -> String {
        match self {
            Self::OK => "200 OK",
            Self::Created => "201 Created",
            Self::Accepted => "202 Accepted",
            Self::NoContent => "204 No Content",
            Self::MultipleChoices => "300 Multiple Choices",
            Self::MovedPermanently => "301 Moved Permanently",
            Self::MovedTemporarily => "302 Moved Temporarily",
            Self::NotModified => "304 Not Modified",
            Self::BadRequest => "400 Bad Request",
            Self::Unauthorized => "401 Unauthorized",
            Self::Forbidden => "403 Forbidden",
            Self::NotFound => "404 Not Found",
            Self::InternalServerError => "500 Internal Server Error",
            Self::NotImplemented => "501 Not Implemented",
            Self::BadGateway => "502 Bad Gateway",
            Self::ServiceUnavailable => "503 Service Unavailable"
        }.to_string()
    }
}

impl Into<usize> for ResultCode {
    fn into(self) -> usize {
        match self {
            Self::OK => 200,
            Self::Created => 201,
            Self::Accepted => 202,
            Self::NoContent => 204,
            Self::MultipleChoices => 300,
            Self::MovedPermanently => 301,
            Self::MovedTemporarily => 302,
            Self::NotModified => 304,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::InternalServerError => 500,
            Self::NotImplemented => 501,
            Self::BadGateway => 502,
            Self::ServiceUnavailable => 503
        }
    }
}

impl TryFrom<usize> for ResultCode {
    type Error = ResultCodeConversionError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            200 => Ok(Self::OK),
            201 => Ok(Self::Created),
            202 => Ok(Self::Accepted),
            204 => Ok(Self::NoContent),
            300 => Ok(Self::MultipleChoices),
            301 => Ok(Self::MovedPermanently),
            302 => Ok(Self::MovedTemporarily),
            304 => Ok(Self::NotModified),
            400 => Ok(Self::BadRequest),
            401 => Ok(Self::Unauthorized),
            403 => Ok(Self::Forbidden),
            404 => Ok(Self::NotFound),
            500 => Ok(Self::InternalServerError),
            501 => Ok(Self::NotImplemented),
            502 => Ok(Self::BadGateway),
            503 => Ok(Self::ServiceUnavailable),
            _ => Err(ResultCodeConversionError)
        }
    }
}
