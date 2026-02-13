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
        format!("Connexion impossible — verifiez l'URL ou votre reseau")
    } else if err.is_timeout() {
        "Timeout — le serveur n'a pas repondu a temps".to_string()
    } else if err.is_redirect() {
        "Trop de redirections".to_string()
    } else if err.is_decode() {
        "Erreur de decodage de la reponse".to_string()
    } else if let Some(url) = err.url() {
        format!("Requete echouee vers {}", url)
    } else {
        format!("Erreur HTTP: {}", err)
    }
}
