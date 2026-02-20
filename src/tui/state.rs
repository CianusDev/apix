#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedPanel {
    Request,
    Response,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawerPanel {
    History,
    Collections,
    Environments,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RequestTab {
    Params,
    Headers,
    Body,
    Auth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseTab {
    Body,
    Headers,
    Cookies,
}

#[derive(Debug)]
pub struct TuiState {
    pub focused_panel: FocusedPanel,
    // URL
    pub url_input: String,
    pub url_cursor: usize,
    pub editing_url: bool,
    // Tabs
    pub request_tab: RequestTab,
    pub response_tab: ResponseTab,
    // Drawer (None = closed)
    pub drawer: Option<DrawerPanel>,
    // History drawer
    pub history_index: usize,
    // Collections drawer
    pub collection_index: usize,
    pub collection_request_index: usize,
    pub in_collection_requests: bool,
    // Environments drawer
    pub environment_index: usize,
    pub environment_variable_index: usize,
    pub in_environment_vars: bool,
    // Body editing
    pub body_input: String,
    pub body_cursor: usize,
    pub is_editing_body: bool,
    // Response scroll
    pub response_scroll: usize,
    pub response_headers_scroll: usize,
    // Header editing
    pub header_index: usize,
    pub header_key_input: String,
    pub header_value_input: String,
    pub header_key_cursor: usize,
    pub header_value_cursor: usize,
    pub editing_header_key: bool,
    pub editing_header: bool,
    // Param editing (mirrors headers)
    pub param_index: usize,
    pub param_key_input: String,
    pub param_value_input: String,
    pub param_key_cursor: usize,
    pub param_value_cursor: usize,
    pub editing_param_key: bool,
    pub editing_param: bool,
    // History search
    pub history_search_active: bool,
    pub history_search_input: String,
    // Collection name editing
    pub editing_collection_name: bool,
    pub collection_name_input: String,
    pub collection_name_cursor: usize,
    // Environment name editing
    pub editing_environment_name: bool,
    pub environment_name_input: String,
    pub environment_name_cursor: usize,
    // Variable editing (key/value)
    pub editing_variable: bool,
    pub variable_key_input: String,
    pub variable_value_input: String,
    pub variable_key_cursor: usize,
    pub variable_value_cursor: usize,
    pub editing_variable_key: bool,
    // Auth editing
    pub editing_auth: bool,
    pub auth_type_index: usize, // 0=None, 1=Bearer, 2=Basic, 3=ApiKey
    pub auth_token_input: String,
    pub auth_token_cursor: usize,
    pub auth_username_input: String,
    pub auth_password_input: String,
    pub auth_username_cursor: usize,
    pub auth_password_cursor: usize,
    pub auth_editing_username: bool,
    pub auth_key_name_input: String,
    pub auth_key_value_input: String,
    pub auth_key_name_cursor: usize,
    pub auth_key_value_cursor: usize,
    pub auth_editing_key_name: bool,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            focused_panel: FocusedPanel::Request,
            url_input: String::new(),
            url_cursor: 0,
            editing_url: false,
            request_tab: RequestTab::Params,
            response_tab: ResponseTab::Body,
            drawer: None,
            history_index: 0,
            collection_index: 0,
            collection_request_index: 0,
            in_collection_requests: false,
            environment_index: 0,
            environment_variable_index: 0,
            in_environment_vars: false,
            body_input: String::new(),
            body_cursor: 0,
            is_editing_body: false,
            response_scroll: 0,
            response_headers_scroll: 0,
            header_index: 0,
            header_key_input: String::new(),
            header_value_input: String::new(),
            header_key_cursor: 0,
            header_value_cursor: 0,
            editing_header_key: true,
            editing_header: false,
            param_index: 0,
            param_key_input: String::new(),
            param_value_input: String::new(),
            param_key_cursor: 0,
            param_value_cursor: 0,
            editing_param_key: true,
            editing_param: false,
            history_search_active: false,
            history_search_input: String::new(),
            editing_collection_name: false,
            collection_name_input: String::new(),
            collection_name_cursor: 0,
            editing_environment_name: false,
            environment_name_input: String::new(),
            environment_name_cursor: 0,
            editing_variable: false,
            variable_key_input: String::new(),
            variable_value_input: String::new(),
            variable_key_cursor: 0,
            variable_value_cursor: 0,
            editing_variable_key: true,
            editing_auth: false,
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

    /// Returns true if any editing mode is active (keyboard input captured).
    pub fn is_editing(&self) -> bool {
        self.editing_url
            || self.is_editing_body
            || self.editing_header
            || self.editing_param
            || self.editing_auth
            || self.editing_variable
            || self.editing_collection_name
            || self.editing_environment_name
            || self.history_search_active
    }

    pub fn next_request_tab(&mut self) {
        self.request_tab = match self.request_tab {
            RequestTab::Params => RequestTab::Headers,
            RequestTab::Headers => RequestTab::Body,
            RequestTab::Body => RequestTab::Auth,
            RequestTab::Auth => RequestTab::Params,
        };
    }

    pub fn prev_request_tab(&mut self) {
        self.request_tab = match self.request_tab {
            RequestTab::Params => RequestTab::Auth,
            RequestTab::Headers => RequestTab::Params,
            RequestTab::Body => RequestTab::Headers,
            RequestTab::Auth => RequestTab::Body,
        };
    }

    pub fn next_response_tab(&mut self) {
        self.response_tab = match self.response_tab {
            ResponseTab::Body => ResponseTab::Headers,
            ResponseTab::Headers => ResponseTab::Cookies,
            ResponseTab::Cookies => ResponseTab::Body,
        };
    }

    pub fn prev_response_tab(&mut self) {
        self.response_tab = match self.response_tab {
            ResponseTab::Body => ResponseTab::Cookies,
            ResponseTab::Headers => ResponseTab::Body,
            ResponseTab::Cookies => ResponseTab::Headers,
        };
    }

    pub fn scroll_response_up(&mut self) {
        self.response_scroll = self.response_scroll.saturating_sub(1);
    }

    pub fn scroll_response_down(&mut self) {
        self.response_scroll += 1;
    }

    /// Returns filtered history indices matching current search query.
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
}
