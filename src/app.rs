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
