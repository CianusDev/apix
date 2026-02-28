use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

use crate::models::environment::{substitute_variables, Environment};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Auth {
    BearerToken { token: String },
    BasicAuth { username: String, password: String },
    ApiKey { header: String, value: String },
}

impl Auth {
    /// Convertit l'auth en header HTTP (nom, valeur)
    pub fn to_header(&self) -> (String, String) {
        match self {
            Auth::BearerToken { token } => {
                ("Authorization".to_string(), format!("Bearer {}", token))
            }
            Auth::BasicAuth { username, password } => {
                let credentials = format!("{}:{}", username, password);
                let encoded = STANDARD.encode(credentials.as_bytes());
                ("Authorization".to_string(), format!("Basic {}", encoded))
            }
            Auth::ApiKey { header, value } => (header.clone(), value.clone()),
        }
    }

    /// Substitue les variables d'environnement dans l'auth
    pub fn substitute_env(&self, env: &Environment) -> Auth {
        match self {
            Auth::BearerToken { token } => Auth::BearerToken {
                token: substitute_variables(token, env),
            },
            Auth::BasicAuth { username, password } => Auth::BasicAuth {
                username: substitute_variables(username, env),
                password: substitute_variables(password, env),
            },
            Auth::ApiKey { header, value } => Auth::ApiKey {
                header: substitute_variables(header, env),
                value: substitute_variables(value, env),
            },
        }
    }

    /// Resume pour affichage TUI (masque partiel)
    pub fn display_summary(&self) -> String {
        match self {
            Auth::BearerToken { token } => {
                if token.len() > 8 {
                    format!("Bearer: {}...{}", &token[..3], &token[token.len() - 3..])
                } else {
                    format!("Bearer: {}", token)
                }
            }
            Auth::BasicAuth { username, .. } => {
                format!("Basic: {}:***", username)
            }
            Auth::ApiKey { header, .. } => {
                format!("API Key: {}:***", header)
            }
        }
    }

}

/// Noms des types d'auth pour le selecteur (index 0 = None)
pub const AUTH_TYPE_NAMES: [&str; 4] = ["None", "Bearer", "Basic", "API Key"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_to_header() {
        let auth = Auth::BearerToken {
            token: "my_token_123".to_string(),
        };
        let (name, value) = auth.to_header();
        assert_eq!(name, "Authorization");
        assert_eq!(value, "Bearer my_token_123");
    }

    #[test]
    fn basic_to_header() {
        let auth = Auth::BasicAuth {
            username: "admin".to_string(),
            password: "secret".to_string(),
        };
        let (name, value) = auth.to_header();
        assert_eq!(name, "Authorization");
        // base64("admin:secret") = "YWRtaW46c2VjcmV0"
        assert_eq!(value, "Basic YWRtaW46c2VjcmV0");
    }

    #[test]
    fn apikey_to_header() {
        let auth = Auth::ApiKey {
            header: "X-API-Key".to_string(),
            value: "key123".to_string(),
        };
        let (name, value) = auth.to_header();
        assert_eq!(name, "X-API-Key");
        assert_eq!(value, "key123");
    }

    #[test]
    fn substitute_env_bearer() {
        let mut env = Environment::new("Test".to_string());
        env.set_variable("TOKEN".to_string(), "real_token".to_string());

        let auth = Auth::BearerToken {
            token: "{{TOKEN}}".to_string(),
        };
        let substituted = auth.substitute_env(&env);
        assert_eq!(
            substituted,
            Auth::BearerToken {
                token: "real_token".to_string()
            }
        );
    }

    #[test]
    fn substitute_env_basic() {
        let mut env = Environment::new("Test".to_string());
        env.set_variable("USER".to_string(), "admin".to_string());
        env.set_variable("PASS".to_string(), "secret".to_string());

        let auth = Auth::BasicAuth {
            username: "{{USER}}".to_string(),
            password: "{{PASS}}".to_string(),
        };
        let substituted = auth.substitute_env(&env);
        assert_eq!(
            substituted,
            Auth::BasicAuth {
                username: "admin".to_string(),
                password: "secret".to_string(),
            }
        );
    }

    #[test]
    fn display_summary_bearer() {
        let auth = Auth::BearerToken {
            token: "abcdefghijklm".to_string(),
        };
        assert_eq!(auth.display_summary(), "Bearer: abc...klm");
    }

    #[test]
    fn display_summary_basic() {
        let auth = Auth::BasicAuth {
            username: "admin".to_string(),
            password: "secret".to_string(),
        };
        assert_eq!(auth.display_summary(), "Basic: admin:***");
    }

    #[test]
    fn display_summary_apikey() {
        let auth = Auth::ApiKey {
            header: "X-API-Key".to_string(),
            value: "secret".to_string(),
        };
        assert_eq!(auth.display_summary(), "API Key: X-API-Key:***");
    }
}
