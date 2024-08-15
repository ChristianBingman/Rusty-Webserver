#[derive(Debug)]
pub struct InvalidMethodErr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Method {
    GET,
    POST,
    HEAD
}

impl TryFrom<String> for Method {
    type Error = InvalidMethodErr;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "HEAD" => Ok(Method::HEAD),
            _ => Err(InvalidMethodErr)
        }
    }
}

impl TryFrom<&str> for Method {
    type Error = InvalidMethodErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "HEAD" => Ok(Method::HEAD),
            _ => Err(InvalidMethodErr)
        }
    }
}

impl From<Method> for String {
    fn from(value: Method) -> Self {
        match value {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::HEAD => "HEAD"
        }.to_string()
    }
}
