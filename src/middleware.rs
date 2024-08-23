use base64::Engine;

use crate::{http10::request::HTTPRequest, Auth};

#[derive(Debug)]
pub struct AuthError {}

pub fn basic_auth(req: &HTTPRequest, auth: &Auth) -> Result<(), AuthError> {
    let auth_header = req
        .headers
        .iter()
        .find(|header| matches!(header, crate::http10::headers::Header::Authorization(_)));

    if let Some(auth_header) = auth_header {
        let inner = auth_header.str_inner().unwrap();
        let mut inner = inner.split(' ');
        let typ = inner.next();
        let token = inner.next();
        if typ.is_none() || token.is_none() || typ.unwrap() != "Basic" {
            return Err(AuthError {});
        }
        if base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", auth.username, auth.password).as_bytes())
            == token.unwrap()
        {
            return Ok(());
        }
    }
    Err(AuthError {})
}

#[cfg(test)]
mod test {
    use crate::http10::headers::Header;

    use super::*;

    #[test]
    fn test_basic_auth_success() {
        let req = HTTPRequest {
            method: crate::http10::methods::Method::GET,
            uri: "/".to_string(),
            version: "HTTP/1.0".to_string(),
            headers: vec![Header::Authorization(
                "Basic YWRtaW46cGFzc3dvcmQ=".to_string(),
            )],
            body: None,
        };
        let auth = Auth {
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        assert!(basic_auth(&req, &auth).is_ok());
    }

    #[test]
    fn test_basic_auth_failure() {
        let req = HTTPRequest {
            method: crate::http10::methods::Method::GET,
            uri: "/".to_string(),
            version: "HTTP/1.0".to_string(),
            headers: vec![],
            body: None,
        };
        let auth = Auth {
            username: "admin".to_string(),
            password: "password".to_string(),
        };
        assert!(basic_auth(&req, &auth).is_err());
    }

    #[test]
    fn test_basic_auth_incorrect_basic() {
        let req = HTTPRequest {
            method: crate::http10::methods::Method::GET,
            uri: "/".to_string(),
            version: "HTTP/1.0".to_string(),
            headers: vec![Header::Authorization(
                "Basic YWRtaW46cGFzc3dvcmQx".to_string(),
            )],
            body: None,
        };
        let auth = Auth {
            username: "admin".to_string(),
            password: "password".to_string(),
        };
        assert!(basic_auth(&req, &auth).is_err());
    }
}
