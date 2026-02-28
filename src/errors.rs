use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApixError {
    #[error("{}", friendly_reqwest_error(.0))]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ApixError>;

fn friendly_reqwest_error(err: &reqwest::Error) -> String {
    if err.is_connect() {
        "Connection failed — check the URL or your network".to_string()
    } else if err.is_timeout() {
        "Timeout — the server did not respond in time".to_string()
    } else if err.is_redirect() {
        "Too many redirects".to_string()
    } else if err.is_decode() {
        "Failed to decode response".to_string()
    } else if let Some(url) = err.url() {
        format!("Request failed to {}", url)
    } else {
        format!("HTTP error: {}", err)
    }
}
