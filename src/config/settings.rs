use std::path::PathBuf;

use crate::errors::Result;

#[derive(Debug, Clone)]
pub struct Settings {
    pub apix_dir: PathBuf,
    pub history_file: PathBuf,
    pub collections_file: PathBuf,
    pub environments_file: PathBuf,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        let apix_dir = PathBuf::from(home).join(".apix");
        let history_file = apix_dir.join("history.json");
        let collections_file = apix_dir.join("collections.json");
        let environments_file = apix_dir.join("environments.json");

        Ok(Self {
            apix_dir,
            history_file,
            collections_file,
            environments_file,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.apix_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_new_paths() {
        let settings = Settings::new().unwrap();
        assert!(settings.apix_dir.ends_with(".apix"));
        assert!(settings.history_file.ends_with("history.json"));
        assert!(settings.collections_file.ends_with("collections.json"));
        assert!(settings.environments_file.ends_with("environments.json"));
    }

    #[test]
    fn settings_files_inside_apix_dir() {
        let settings = Settings::new().unwrap();
        assert!(settings.history_file.starts_with(&settings.apix_dir));
        assert!(settings.collections_file.starts_with(&settings.apix_dir));
        assert!(settings.environments_file.starts_with(&settings.apix_dir));
    }

    #[test]
    fn settings_ensure_dirs() {
        let settings = Settings::new().unwrap();
        assert!(settings.ensure_dirs().is_ok());
        assert!(settings.apix_dir.exists());
    }
}
