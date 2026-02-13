use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::errors::{ApixError, Result};
use crate::models::auth::Auth;
use crate::models::{Method, Request};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionEntry {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    #[serde(default)]
    pub auth: Option<Auth>,
}

impl CollectionEntry {
    pub fn from_request(name: &str, request: &Request) -> Self {
        Self {
            name: name.to_string(),
            method: request.method.to_string(),
            url: request.url.clone(),
            headers: request.headers.clone(),
            body: request.body.clone(),
            auth: request.auth.clone(),
        }
    }

    pub fn to_request(&self) -> Result<Request> {
        let method = Method::from_str(&self.method)?;
        let mut request = Request::new(method, self.url.clone());
        request.headers = self.headers.clone();
        request.body = self.body.clone();
        request.auth = self.auth.clone();
        Ok(request)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub entries: Vec<CollectionEntry>,
}

impl Collection {
    pub fn new(name: String) -> Self {
        Self {
            name,
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: CollectionEntry) {
        self.entries.push(entry);
    }

    pub fn remove_entry(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Collections {
    pub items: Vec<Collection>,
}

impl Collections {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| ApixError::ConfigError(format!("Lecture collections: {}", e)))?;
        let collections: Collections = serde_json::from_str(&content)
            .map_err(|e| ApixError::ConfigError(format!("Parse collections: {}", e)))?;
        Ok(collections)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ApixError::ConfigError(format!("Serialisation collections: {}", e)))?;
        std::fs::write(path, content)
            .map_err(|e| ApixError::ConfigError(format!("Ecriture collections: {}", e)))?;
        Ok(())
    }

    pub fn add_collection(&mut self, name: String) {
        self.items.push(Collection::new(name));
    }

    pub fn remove_collection(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collections_add_and_remove() {
        let mut cols = Collections::default();
        cols.add_collection("API Users".to_string());
        cols.add_collection("Auth".to_string());
        assert_eq!(cols.items.len(), 2);
        assert_eq!(cols.items[0].name, "API Users");

        cols.remove_collection(0);
        assert_eq!(cols.items.len(), 1);
        assert_eq!(cols.items[0].name, "Auth");
    }

    #[test]
    fn collection_add_and_remove_entry() {
        let mut col = Collection::new("Test".to_string());
        let entry = CollectionEntry {
            name: "Get users".to_string(),
            method: "GET".to_string(),
            url: "https://api.example.com/users".to_string(),
            headers: vec![],
            body: None,
            auth: None,
        };
        col.add_entry(entry);
        assert_eq!(col.entries.len(), 1);

        col.remove_entry(0);
        assert!(col.entries.is_empty());
    }

    #[test]
    fn entry_from_request_and_back() {
        let request = Request::new(Method::POST, "https://api.example.com".to_string());
        let entry = CollectionEntry::from_request("Create user", &request);

        assert_eq!(entry.name, "Create user");
        assert_eq!(entry.method, "POST");

        let restored = entry.to_request().unwrap();
        assert_eq!(restored.method, Method::POST);
        assert_eq!(restored.url, "https://api.example.com");
    }

    #[test]
    fn collections_save_and_load() {
        let mut cols = Collections::default();
        cols.add_collection("Test".to_string());
        let entry = CollectionEntry::from_request(
            "Get",
            &Request::new(Method::GET, "https://example.com".to_string()),
        );
        cols.items[0].add_entry(entry);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collections.json");

        cols.save(&path).unwrap();
        assert!(path.exists());

        let loaded = Collections::load(&path).unwrap();
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].name, "Test");
        assert_eq!(loaded.items[0].entries.len(), 1);
    }

    #[test]
    fn collections_load_missing_file_returns_empty() {
        let path = Path::new("/tmp/apix_test_nonexistent_collections.json");
        let cols = Collections::load(path).unwrap();
        assert!(cols.items.is_empty());
    }
}
