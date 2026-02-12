use reqwest::header::HeaderMap;
use serde_json::Value;

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: HeaderMap,
    pub body: Value,
}

impl Response {
    pub fn new(status: u16, headers: HeaderMap, body: Value) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    pub fn format(&self) -> String {
        format!(
            "status: {}\nheaders: {:?}\nbody: {:#?}",
            self.status, self.headers, self.body
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn response_new() {
        let resp = Response::new(200, HeaderMap::new(), json!({"key": "value"}));
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body["key"], "value");
    }

    #[test]
    fn response_format_contains_status() {
        let resp = Response::new(404, HeaderMap::new(), json!(null));
        let output = resp.format();
        assert!(output.contains("404"));
    }

    #[test]
    fn response_format_contains_body() {
        let resp = Response::new(200, HeaderMap::new(), json!({"msg": "hello"}));
        let output = resp.format();
        assert!(output.contains("hello"));
    }
}
