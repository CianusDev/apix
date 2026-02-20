use crate::errors::Result;
use crate::models::{Method, Request, Response};

fn percent_encode(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            let buf = &mut [0u8; 4];
            let bytes = c.encode_utf8(buf);
            bytes
                .bytes()
                .map(|b| match b {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                        (b as char).to_string()
                    }
                    _ => format!("%{:02X}", b),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn build_url(base: &str, params: &[(String, String)]) -> String {
    if params.is_empty() {
        return base.to_string();
    }
    let sep = if base.contains('?') { '&' } else { '?' };
    let qs: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{}{}{}", base, sep, qs)
}

pub struct RequestBuilder<'a> {
    client: &'a reqwest::Client,
    request: &'a Request,
}

impl<'a> RequestBuilder<'a> {
    pub fn new(client: &'a reqwest::Client, request: &'a Request) -> Self {
        Self { client, request }
    }

    pub async fn send(self) -> Result<Response> {
        let url = build_url(&self.request.url, &self.request.params);
        let mut req_builder = match self.request.method {
            Method::GET => self.client.get(&url),
            Method::POST => self.client.post(&url),
            Method::PUT => self.client.put(&url),
            Method::DELETE => self.client.delete(&url),
            Method::PATCH => self.client.patch(&url),
        };

        for (key, value) in &self.request.headers {
            req_builder = req_builder.header(key, value);
        }

        // Appliquer l'authentification
        if let Some(auth) = &self.request.auth {
            let (name, value) = auth.to_header();
            req_builder = req_builder.header(name, value);
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
