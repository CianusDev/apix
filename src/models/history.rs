use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::errors::{ApixError, Result};
use crate::models::auth::Auth;
use crate::models::{Method, Request, Response};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub method: String,
    pub url: String,
    pub request_headers: Vec<(String, String)>,
    pub request_body: Option<String>,
    #[serde(default)]
    pub auth: Option<Auth>,
    pub status: Option<u16>,
    pub response_body: Option<String>,
}

impl HistoryEntry {
    pub fn from_request_response(request: &Request, response: &Response) -> Self {
        let body_str = serde_json::to_string(&response.body).ok();

        Self {
            timestamp: chrono::Local::now().to_rfc3339(),
            method: request.method.to_string(),
            url: request.url.clone(),
            request_headers: request.headers.clone(),
            request_body: request.body.clone(),
            auth: request.auth.clone(),
            status: Some(response.status),
            response_body: body_str,
        }
    }

    pub fn to_request(&self) -> Result<Request> {
        let method = Method::from_str(&self.method)?;
        let mut request = Request::new(method, self.url.clone());
        request.headers = self.request_headers.clone();
        request.body = self.request_body.clone();
        request.auth = self.auth.clone();
        Ok(request)
    }

    /// Reconstruit une Response simplifiee depuis l'historique
    pub fn to_response(&self) -> Option<Response> {
        let status = self.status?;
        let body = self.response_body.as_ref().and_then(|s| {
            serde_json::from_str::<serde_json::Value>(s).ok()
        }).unwrap_or(serde_json::Value::Null);

        Some(Response::new(status, reqwest::header::HeaderMap::new(), body))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
}

impl History {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| ApixError::ConfigError(format!("Lecture historique: {}", e)))?;
        let history: History = serde_json::from_str(&content)
            .map_err(|e| ApixError::ConfigError(format!("Parse historique: {}", e)))?;
        Ok(history)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ApixError::ConfigError(format!("Serialisation historique: {}", e)))?;
        std::fs::write(path, content)
            .map_err(|e| ApixError::ConfigError(format!("Ecriture historique: {}", e)))?;
        Ok(())
    }

    pub fn add(&mut self, entry: HistoryEntry) {
        self.entries.insert(0, entry); // Plus recente en premier
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry() -> HistoryEntry {
        HistoryEntry {
            timestamp: "2025-01-01T00:00:00+00:00".to_string(),
            method: "GET".to_string(),
            url: "https://example.com".to_string(),
            request_headers: vec![("Accept".to_string(), "application/json".to_string())],
            request_body: None,
            auth: None,
            status: Some(200),
            response_body: Some(r#"{"ok": true}"#.to_string()),
        }
    }

    #[test]
    fn history_add_prepends() {
        let mut history = History::default();
        let entry1 = sample_entry();
        let mut entry2 = sample_entry();
        entry2.url = "https://other.com".to_string();

        history.add(entry1);
        history.add(entry2);

        assert_eq!(history.entries.len(), 2);
        assert_eq!(history.entries[0].url, "https://other.com");
        assert_eq!(history.entries[1].url, "https://example.com");
    }

    #[test]
    fn history_remove() {
        let mut history = History::default();
        history.add(sample_entry());
        history.add(sample_entry());
        assert_eq!(history.entries.len(), 2);

        history.remove(0);
        assert_eq!(history.entries.len(), 1);
    }

    #[test]
    fn history_save_and_load() {
        let mut history = History::default();
        history.add(sample_entry());

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");

        history.save(&path).unwrap();
        assert!(path.exists());

        let loaded = History::load(&path).unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].url, "https://example.com");
    }

    #[test]
    fn history_load_missing_file_returns_empty() {
        let path = Path::new("/tmp/apix_test_nonexistent_history.json");
        let history = History::load(path).unwrap();
        assert!(history.entries.is_empty());
    }

    #[test]
    fn entry_to_request() {
        let entry = sample_entry();
        let request = entry.to_request().unwrap();

        assert_eq!(request.method, Method::GET);
        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.headers.len(), 1);
        assert!(request.body.is_none());
    }

    #[test]
    fn entry_to_response() {
        let entry = sample_entry();
        let response = entry.to_response().unwrap();

        assert_eq!(response.status, 200);
    }
}
