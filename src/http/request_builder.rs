use crate::errors::Result;
use crate::models::{Method, Request, Response};

pub struct RequestBuilder<'a> {
    client: &'a reqwest::Client,
    request: &'a Request,
}

impl<'a> RequestBuilder<'a> {
    pub fn new(client: &'a reqwest::Client, request: &'a Request) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<Response> {
        let mut req_builder = match self.request.method {
            Method::GET => self.client.get(&self.request.url),
            Method::POST => self.client.post(&self.request.url),
            Method::PUT => self.client.put(&self.request.url),
            Method::DELETE => self.client.delete(&self.request.url),
            Method::PATCH => self.client.patch(&self.request.url),
        };

        for (key, value) in &self.request.headers {
            req_builder = req_builder.header(key, value);
        }

        if let Some(body) = &self.request.body {
            req_builder = req_builder.body(body.clone());
        }

        let response = req_builder.send().await?;
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let raw_body = response.text().await?;

        let body = match serde_json::from_str::<serde_json::Value>(&raw_body) {
            Ok(json) => json,
            Err(_) => serde_json::Value::String(raw_body),
        };

        Ok(Response::new(status, headers, body))
    }
}
