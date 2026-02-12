use crate::errors::Result;
use crate::http::request_builder::RequestBuilder;
use crate::models::{Request, Response};

pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn execute(&self, request: &Request) -> Result<Response> {
        let builder = RequestBuilder::new(&self.client, request);
        builder.send().await
    }
}
