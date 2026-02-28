use serde::{Deserialize, Serialize};
use std::fmt;

use super::auth::Auth;
use crate::errors::ApixError;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl Method {
    pub fn from_str(s: &str) -> crate::errors::Result<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "PATCH" => Ok(Method::PATCH),
            _ => Err(ApixError::InvalidMethod(s.to_string())),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::GET => write!(f, "GET"),
            Method::POST => write!(f, "POST"),
            Method::PUT => write!(f, "PUT"),
            Method::DELETE => write!(f, "DELETE"),
            Method::PATCH => write!(f, "PATCH"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    #[serde(default)]
    pub auth: Option<Auth>,
    #[serde(default)]
    pub params: Vec<(String, String)>,
}

impl Request {
    pub fn new(method: Method, url: String) -> Self {
        Self {
            method,
            url,
            headers: Vec::new(),
            body: None,
            auth: None,
            params: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_from_str_valid() {
        assert_eq!(Method::from_str("GET").unwrap(), Method::GET);
        assert_eq!(Method::from_str("POST").unwrap(), Method::POST);
        assert_eq!(Method::from_str("PUT").unwrap(), Method::PUT);
        assert_eq!(Method::from_str("DELETE").unwrap(), Method::DELETE);
        assert_eq!(Method::from_str("PATCH").unwrap(), Method::PATCH);
    }

    #[test]
    fn method_from_str_case_insensitive() {
        assert_eq!(Method::from_str("get").unwrap(), Method::GET);
        assert_eq!(Method::from_str("Post").unwrap(), Method::POST);
        assert_eq!(Method::from_str("dElEtE").unwrap(), Method::DELETE);
    }

    #[test]
    fn method_from_str_invalid() {
        assert!(Method::from_str("INVALID").is_err());
        assert!(Method::from_str("").is_err());
        assert!(Method::from_str("OPTION").is_err());
    }

    #[test]
    fn method_display() {
        assert_eq!(format!("{}", Method::GET), "GET");
        assert_eq!(format!("{}", Method::POST), "POST");
        assert_eq!(format!("{}", Method::PUT), "PUT");
        assert_eq!(format!("{}", Method::DELETE), "DELETE");
        assert_eq!(format!("{}", Method::PATCH), "PATCH");
    }

    #[test]
    fn request_new_defaults() {
        let req = Request::new(Method::GET, "https://example.com".to_string());
        assert_eq!(req.method, Method::GET);
        assert_eq!(req.url, "https://example.com");
        assert!(req.headers.is_empty());
        assert!(req.body.is_none());
    }
}
