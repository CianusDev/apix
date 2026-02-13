use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::errors::{ApixError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Environment {
    pub name: String,
    pub variables: HashMap<String, String>,
}

impl Environment {
    pub fn new(name: String) -> Self {
        Self {
            name,
            variables: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    pub fn remove_variable(&mut self, key: &str) {
        self.variables.remove(key);
    }

    /// Retourne les cles triees pour un affichage stable
    pub fn sorted_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.variables.keys().cloned().collect();
        keys.sort();
        keys
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Environments {
    pub items: Vec<Environment>,
}

impl Environments {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| ApixError::ConfigError(format!("Lecture environments: {}", e)))?;
        let envs: Environments = serde_json::from_str(&content)
            .map_err(|e| ApixError::ConfigError(format!("Parse environments: {}", e)))?;
        Ok(envs)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ApixError::ConfigError(format!("Serialisation environments: {}", e)))?;
        std::fs::write(path, content)
            .map_err(|e| ApixError::ConfigError(format!("Ecriture environments: {}", e)))?;
        Ok(())
    }

    pub fn add_environment(&mut self, name: String) {
        self.items.push(Environment::new(name));
    }

    pub fn remove_environment(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
        }
    }
}

/// Remplace toutes les occurrences de `{{key}}` par la valeur correspondante.
/// Les variables non trouvees restent telles quelles.
pub fn substitute_variables(text: &str, env: &Environment) -> String {
    let mut result = text.to_string();
    for (key, value) in &env.variables {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitute_replaces_variables() {
        let mut env = Environment::new("Test".to_string());
        env.set_variable("BASE_URL".to_string(), "http://localhost:3000".to_string());
        env.set_variable("TOKEN".to_string(), "abc123".to_string());

        let result = substitute_variables("{{BASE_URL}}/api/users", &env);
        assert_eq!(result, "http://localhost:3000/api/users");

        let result = substitute_variables("Bearer {{TOKEN}}", &env);
        assert_eq!(result, "Bearer abc123");
    }

    #[test]
    fn substitute_keeps_unknown_variables() {
        let env = Environment::new("Empty".to_string());
        let result = substitute_variables("{{UNKNOWN}}/path", &env);
        assert_eq!(result, "{{UNKNOWN}}/path");
    }

    #[test]
    fn substitute_multiple_in_same_string() {
        let mut env = Environment::new("Test".to_string());
        env.set_variable("HOST".to_string(), "example.com".to_string());
        env.set_variable("PORT".to_string(), "8080".to_string());

        let result = substitute_variables("http://{{HOST}}:{{PORT}}/api", &env);
        assert_eq!(result, "http://example.com:8080/api");
    }

    #[test]
    fn environment_set_and_remove_variable() {
        let mut env = Environment::new("Dev".to_string());
        env.set_variable("KEY".to_string(), "val".to_string());
        assert_eq!(env.variables.get("KEY").unwrap(), "val");

        env.remove_variable("KEY");
        assert!(env.variables.get("KEY").is_none());
    }

    #[test]
    fn environments_add_and_remove() {
        let mut envs = Environments::default();
        envs.add_environment("Dev".to_string());
        envs.add_environment("Prod".to_string());
        assert_eq!(envs.items.len(), 2);

        envs.remove_environment(0);
        assert_eq!(envs.items.len(), 1);
        assert_eq!(envs.items[0].name, "Prod");
    }

    #[test]
    fn environments_save_and_load() {
        let mut envs = Environments::default();
        envs.add_environment("Dev".to_string());
        envs.items[0].set_variable("URL".to_string(), "http://localhost".to_string());

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("environments.json");

        envs.save(&path).unwrap();
        assert!(path.exists());

        let loaded = Environments::load(&path).unwrap();
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].name, "Dev");
        assert_eq!(
            loaded.items[0].variables.get("URL").unwrap(),
            "http://localhost"
        );
    }

    #[test]
    fn environments_load_missing_returns_empty() {
        let path = Path::new("/tmp/apix_test_nonexistent_envs.json");
        let envs = Environments::load(path).unwrap();
        assert!(envs.items.is_empty());
    }
}
