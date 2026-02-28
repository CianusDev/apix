use crate::errors::{ApixError, Result};
use crate::http::request_builder::RequestBuilder;
use crate::models::{Request, Response};

#[derive(Clone)]
pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .cookie_store(true)
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    pub async fn execute(&self, request: &Request) -> Result<Response> {
        validate_url(&request.url)?;
        let builder = RequestBuilder::new(&self.client, request);
        builder.send().await
    }
}

fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(ApixError::InvalidUrl("URL is empty".to_string()));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ApixError::InvalidUrl(format!(
            "'{}' — URL must start with http:// or https://",
            url
        )));
    }

    let after_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or("");

    if after_scheme.is_empty() || after_scheme == "/" {
        return Err(ApixError::InvalidUrl(format!(
            "'{}' — missing hostname",
            url
        )));
    }

    Ok(())
}
