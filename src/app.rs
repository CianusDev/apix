use crate::config::Settings;
use crate::errors::Result;
use crate::http::HttpClient;
use crate::models::{Method, Request, Response};

pub struct App {
    pub settings: Settings,
    pub http_client: HttpClient,
    pub current_request: Request,
    pub current_response: Option<Response>,
    pub is_loading: bool,
    pub error_message: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        let settings = Settings::new()?;
        let http_client = HttpClient::new();
        let current_request = Request::new(Method::GET, String::new());

        Ok(Self {
            settings,
            http_client,
            current_request,
            current_response: None,
            is_loading: false,
            error_message: None,
        })
    }

    pub fn initialize(&self) -> Result<()> {
        self.settings.ensure_dirs()?;
        Ok(())
    }

    pub fn set_method(&mut self, method: Method) {
        self.current_request.method = method;
    }

    pub fn set_url(&mut self, url: String) {
        self.current_request.url = url;
    }

    pub fn set_body(&mut self, body: Option<String>) {
        self.current_request.body = body;
    }

    pub fn add_header(&mut self, key: String, value: String) {
        self.current_request.headers.push((key, value));
    }

    pub fn set_header(&mut self, index: usize, key: String, value: String) {
        if index < self.current_request.headers.len() {
            self.current_request.headers[index] = (key, value);
        }
    }

    pub fn remove_header(&mut self, index: usize) {
        if index < self.current_request.headers.len() {
            self.current_request.headers.remove(index);
        }
    }

    pub fn start_loading(&mut self) {
        self.is_loading = true;
        self.error_message = None;
    }

    pub fn finish_loading(&mut self, response: std::result::Result<Response, crate::errors::ApixError>) {
        self.is_loading = false;
        match response {
            Ok(resp) => {
                self.current_response = Some(resp);
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        App::new().unwrap()
    }

    #[test]
    fn add_header() {
        let mut app = test_app();
        app.add_header("Accept".into(), "application/json".into());
        assert_eq!(app.current_request.headers.len(), 1);
        assert_eq!(app.current_request.headers[0].0, "Accept");
        assert_eq!(app.current_request.headers[0].1, "application/json");
    }

    #[test]
    fn set_header_updates_existing() {
        let mut app = test_app();
        app.add_header("Accept".into(), "text/html".into());
        app.set_header(0, "Accept".into(), "application/json".into());
        assert_eq!(app.current_request.headers.len(), 1);
        assert_eq!(app.current_request.headers[0].1, "application/json");
    }

    #[test]
    fn set_header_out_of_bounds_does_nothing() {
        let mut app = test_app();
        app.set_header(5, "X-Key".into(), "val".into());
        assert!(app.current_request.headers.is_empty());
    }

    #[test]
    fn remove_header() {
        let mut app = test_app();
        app.add_header("A".into(), "1".into());
        app.add_header("B".into(), "2".into());
        app.remove_header(0);
        assert_eq!(app.current_request.headers.len(), 1);
        assert_eq!(app.current_request.headers[0].0, "B");
    }

    #[test]
    fn remove_header_out_of_bounds_does_nothing() {
        let mut app = test_app();
        app.remove_header(0);
        assert!(app.current_request.headers.is_empty());
    }
}
