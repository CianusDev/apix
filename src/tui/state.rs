#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedPanel {
    Request,
    Response,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RequestField {
    Method,
    Url,
    Auth,
    Headers,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectionsView {
    CollectionList,
    RequestList,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvironmentsView {
    EnvironmentList,
    VariableList,
}

#[derive(Debug)]
pub struct TuiState {
    pub focused_panel: FocusedPanel,
    pub focused_request_field: RequestField,
    pub url_input: String,
    pub url_cursor: usize,
    pub body_input: String,
    pub body_cursor: usize,
    pub response_scroll: usize,
    pub is_editing: bool,
    // Header editing
    pub header_index: usize,
    pub header_key_input: String,
    pub header_value_input: String,
    pub header_key_cursor: usize,
    pub header_value_cursor: usize,
    pub editing_header_key: bool, // true = editing key, false = editing value
    // History
    pub show_history: bool,
    pub history_index: usize,
    pub history_search_active: bool,
    pub history_search_input: String,
    // Collections
    pub show_collections: bool,
    pub collections_view: CollectionsView,
    pub collection_index: usize,
    pub collection_request_index: usize,
    pub editing_collection_name: bool,
    pub collection_name_input: String,
    pub collection_name_cursor: usize,
    // Environments
    pub show_environments: bool,
    pub environments_view: EnvironmentsView,
    pub environment_index: usize,
    pub environment_variable_index: usize,
    pub editing_environment_name: bool,
    pub environment_name_input: String,
    pub environment_name_cursor: usize,
    // Edition de variable (cle/valeur)
    pub editing_variable: bool,
    pub variable_key_input: String,
    pub variable_value_input: String,
    pub variable_key_cursor: usize,
    pub variable_value_cursor: usize,
    pub editing_variable_key: bool,
    // Auth editing
    pub auth_type_index: usize, // 0=None, 1=Bearer, 2=Basic, 3=ApiKey
    pub auth_token_input: String,
    pub auth_token_cursor: usize,
    pub auth_username_input: String,
    pub auth_password_input: String,
    pub auth_username_cursor: usize,
    pub auth_password_cursor: usize,
    pub auth_editing_username: bool, // true=username, false=password
    pub auth_key_name_input: String,
    pub auth_key_value_input: String,
    pub auth_key_name_cursor: usize,
    pub auth_key_value_cursor: usize,
    pub auth_editing_key_name: bool, // true=header name, false=value
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            focused_panel: FocusedPanel::Request,
            focused_request_field: RequestField::Url,
            url_input: String::new(),
            url_cursor: 0,
            body_input: String::new(),
            body_cursor: 0,
            response_scroll: 0,
            is_editing: false,
            header_index: 0,
            header_key_input: String::new(),
            header_value_input: String::new(),
            header_key_cursor: 0,
            header_value_cursor: 0,
            editing_header_key: true,
            show_history: false,
            history_index: 0,
            history_search_active: false,
            history_search_input: String::new(),
            show_collections: false,
            collections_view: CollectionsView::CollectionList,
            collection_index: 0,
            collection_request_index: 0,
            editing_collection_name: false,
            collection_name_input: String::new(),
            collection_name_cursor: 0,
            show_environments: false,
            environments_view: EnvironmentsView::EnvironmentList,
            environment_index: 0,
            environment_variable_index: 0,
            editing_environment_name: false,
            environment_name_input: String::new(),
            environment_name_cursor: 0,
            editing_variable: false,
            variable_key_input: String::new(),
            variable_value_input: String::new(),
            variable_key_cursor: 0,
            variable_value_cursor: 0,
            editing_variable_key: true,
            auth_type_index: 0,
            auth_token_input: String::new(),
            auth_token_cursor: 0,
            auth_username_input: String::new(),
            auth_password_input: String::new(),
            auth_username_cursor: 0,
            auth_password_cursor: 0,
            auth_editing_username: true,
            auth_key_name_input: String::new(),
            auth_key_value_input: String::new(),
            auth_key_name_cursor: 0,
            auth_key_value_cursor: 0,
            auth_editing_key_name: true,
        }
    }

    /// Retourne les indices (dans la liste complète) des entrées d'historique
    /// qui correspondent à la recherche courante.
    pub fn history_filtered_indices(
        &self,
        entries: &[crate::models::HistoryEntry],
    ) -> Vec<usize> {
        if self.history_search_input.is_empty() {
            return (0..entries.len()).collect();
        }
        let q = self.history_search_input.to_lowercase();
        entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.url.to_lowercase().contains(&q)
                    || e.method.to_lowercase().contains(&q)
                    || e.status
                        .map(|s| s.to_string().contains(&q))
                        .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn next_request_field(&mut self) {
        self.focused_request_field = match self.focused_request_field {
            RequestField::Method => RequestField::Url,
            RequestField::Url => RequestField::Auth,
            RequestField::Auth => RequestField::Headers,
            RequestField::Headers => RequestField::Body,
            RequestField::Body => RequestField::Method,
        };
    }

    pub fn prev_request_field(&mut self) {
        self.focused_request_field = match self.focused_request_field {
            RequestField::Method => RequestField::Body,
            RequestField::Url => RequestField::Method,
            RequestField::Auth => RequestField::Url,
            RequestField::Headers => RequestField::Auth,
            RequestField::Body => RequestField::Headers,
        };
    }

    pub fn scroll_response_up(&mut self) {
        self.response_scroll = self.response_scroll.saturating_sub(1);
    }

    pub fn scroll_response_down(&mut self) {
        self.response_scroll += 1;
    }
}
