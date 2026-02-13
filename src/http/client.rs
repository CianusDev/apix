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
            client: reqwest::Client::new(),
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
        return Err(ApixError::InvalidUrl("URL vide".to_string()));
    }

    // Verifier le scheme
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ApixError::InvalidUrl(format!(
            "'{}' — l'URL doit commencer par http:// ou https://",
            url
        )));
    }

    // Verifier qu'il y a un host apres le scheme
    let after_scheme = if url.starts_with("https://") {
        &url[8..]
    } else {
        &url[7..]
    };

    if after_scheme.is_empty() || after_scheme == "/" {
        return Err(ApixError::InvalidUrl(format!(
            "'{}' — nom de domaine manquant",
            url
        )));
    }

    Ok(())
}
