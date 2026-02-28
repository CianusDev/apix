use crate::config::Settings;
use crate::errors::Result;
use crate::http::HttpClient;
use crate::models::{Auth, CollectionEntry, Collections, Environments, History, HistoryEntry, Method, Request, Response};
use crate::models::environment::substitute_variables;

pub struct App {
    pub settings: Settings,
    pub http_client: HttpClient,
    pub current_request: Request,
    pub current_response: Option<Response>,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub history: History,
    pub collections: Collections,
    pub environments: Environments,
    pub active_environment: Option<usize>,
    /// Message temporaire affiché dans la barre de titre (ex: "Copié !")
    pub status_notice: Option<(String, std::time::Instant)>,
}

impl App {
    pub fn new() -> Result<Self> {
        let settings = Settings::new()?;
        let http_client = HttpClient::new();
        let current_request = Request::new(Method::GET, String::new());

        Ok(Self {
            settings,
            http_client,
            current_request,
            current_response: None,
            is_loading: false,
            error_message: None,
            history: History::default(),
            collections: Collections::default(),
            environments: Environments::default(),
            active_environment: None,
            status_notice: None,
        })
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.settings.ensure_dirs()?;
        self.history = History::load(&self.settings.history_file)?;
        self.collections = Collections::load(&self.settings.collections_file)?;
        self.environments = Environments::load(&self.settings.environments_file)?;
        Ok(())
    }

    pub fn set_method(&mut self, method: Method) {
        self.current_request.method = method;
    }

    pub fn set_url(&mut self, url: String) {
        self.current_request.url = url;
    }

    pub fn set_body(&mut self, body: Option<String>) {
        self.current_request.body = body;
    }

    pub fn add_header(&mut self, key: String, value: String) {
        self.current_request.headers.push((key, value));
    }

    pub fn set_header(&mut self, index: usize, key: String, value: String) {
        if index < self.current_request.headers.len() {
            self.current_request.headers[index] = (key, value);
        }
    }

    pub fn remove_header(&mut self, index: usize) {
        if index < self.current_request.headers.len() {
            self.current_request.headers.remove(index);
        }
    }

    pub fn start_loading(&mut self) {
        self.is_loading = true;
        self.error_message = None;
    }

    pub fn finish_loading(&mut self, response: std::result::Result<Response, crate::errors::ApixError>) {
        self.is_loading = false;
        match response {
            Ok(resp) => {
                // Sauvegarder dans l'historique
                let entry = HistoryEntry::from_request_response(&self.current_request, &resp);
                self.history.add(entry);
                let _ = self.history.save(&self.settings.history_file);

                self.current_response = Some(resp);
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
    }

    pub fn load_history_entry(&mut self, index: usize) {
        if let Some(entry) = self.history.entries.get(index)
            && let Ok(request) = entry.to_request()
        {
            self.current_request = request;
            self.current_response = entry.to_response();
            self.error_message = None;
        }
    }

    pub fn remove_history_entry(&mut self, index: usize) {
        self.history.remove(index);
        let _ = self.history.save(&self.settings.history_file);
    }

    // --- Collections ---

    pub fn save_collections(&self) {
        let _ = self.collections.save(&self.settings.collections_file);
    }

    pub fn add_request_to_collection(&mut self, col_index: usize, name: String) {
        if let Some(col) = self.collections.items.get_mut(col_index) {
            let entry = CollectionEntry::from_request(&name, &self.current_request);
            col.add_entry(entry);
            self.save_collections();
        }
    }

    // --- Params ---

    pub fn add_param(&mut self, key: String, value: String) {
        self.current_request.params.push((key, value));
    }

    pub fn set_param(&mut self, index: usize, key: String, value: String) {
        if index < self.current_request.params.len() {
            self.current_request.params[index] = (key, value);
        }
    }

    pub fn remove_param(&mut self, index: usize) {
        if index < self.current_request.params.len() {
            self.current_request.params.remove(index);
        }
    }

    // --- Auth ---

    pub fn set_auth(&mut self, auth: Option<Auth>) {
        self.current_request.auth = auth;
    }

    // --- Environments ---

    pub fn save_environments(&self) {
        let _ = self.environments.save(&self.settings.environments_file);
    }

    /// Clone la requete et substitue les variables de l'environnement actif
    pub fn apply_environment(&self, request: &Request) -> Request {
        let env = match self.active_environment {
            Some(idx) => match self.environments.items.get(idx) {
                Some(env) => env,
                None => return request.clone(),
            },
            None => return request.clone(),
        };

        let mut req = request.clone();
        req.url = substitute_variables(&req.url, env);
        req.headers = req
            .headers
            .iter()
            .map(|(k, v)| {
                (
                    substitute_variables(k, env),
                    substitute_variables(v, env),
                )
            })
            .collect();
        if let Some(body) = &req.body {
            req.body = Some(substitute_variables(body, env));
        }
        if let Some(auth) = &req.auth {
            req.auth = Some(auth.substitute_env(env));
        }
        req.params = req
            .params
            .iter()
            .map(|(k, v)| {
                (
                    substitute_variables(k, env),
                    substitute_variables(v, env),
                )
            })
            .collect();
        req
    }

    /// Retourne le message de notice si moins de 3 secondes se sont écoulées.
    pub fn status_notice_text(&self) -> Option<&str> {
        if let Some((msg, instant)) = &self.status_notice
            && instant.elapsed().as_secs() < 3
        {
            return Some(msg.as_str());
        }
        None
    }

    fn set_notice(&mut self, msg: String) {
        self.status_notice = Some((msg, std::time::Instant::now()));
    }

    /// Copie le body de la réponse dans le presse-papiers système.
    pub fn copy_response_to_clipboard(&mut self) {
        let Some(ref response) = self.current_response else {
            return;
        };
        let text = serde_json::to_string_pretty(&response.body).unwrap_or_default();
        match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(text)) {
            Ok(_) => self.set_notice("Copied to clipboard!".to_string()),
            Err(e) => self.set_notice(format!("Clipboard error: {}", e)),
        }
    }

    /// Sauvegarde le body de la réponse dans un fichier JSON dans le répertoire courant.
    pub fn save_response_to_file(&mut self) {
        let Some(ref response) = self.current_response else {
            return;
        };
        let text = serde_json::to_string_pretty(&response.body).unwrap_or_default();
        let filename = "response.json";
        match std::fs::write(filename, &text) {
            Ok(_) => self.set_notice(format!("Saved: {}", filename)),
            Err(e) => self.set_notice(format!("Error: {}", e)),
        }
    }

    pub fn load_collection_entry(&mut self, col_index: usize, entry_index: usize) {
        if let Some(col) = self.collections.items.get(col_index)
            && let Some(entry) = col.entries.get(entry_index)
            && let Ok(request) = entry.to_request()
        {
            self.current_request = request;
            self.current_response = None;
            self.error_message = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        App::new().unwrap()
    }

    #[test]
    fn add_header() {
        let mut app = test_app();
        app.add_header("Accept".into(), "application/json".into());
        assert_eq!(app.current_request.headers.len(), 1);
        assert_eq!(app.current_request.headers[0].0, "Accept");
        assert_eq!(app.current_request.headers[0].1, "application/json");
    }

    #[test]
    fn set_header_updates_existing() {
        let mut app = test_app();
        app.add_header("Accept".into(), "text/html".into());
        app.set_header(0, "Accept".into(), "application/json".into());
        assert_eq!(app.current_request.headers.len(), 1);
        assert_eq!(app.current_request.headers[0].1, "application/json");
    }

    #[test]
    fn set_header_out_of_bounds_does_nothing() {
        let mut app = test_app();
        app.set_header(5, "X-Key".into(), "val".into());
        assert!(app.current_request.headers.is_empty());
    }

    #[test]
    fn remove_header() {
        let mut app = test_app();
        app.add_header("A".into(), "1".into());
        app.add_header("B".into(), "2".into());
        app.remove_header(0);
        assert_eq!(app.current_request.headers.len(), 1);
        assert_eq!(app.current_request.headers[0].0, "B");
    }

    #[test]
    fn remove_header_out_of_bounds_does_nothing() {
        let mut app = test_app();
        app.remove_header(0);
        assert!(app.current_request.headers.is_empty());
    }
}
