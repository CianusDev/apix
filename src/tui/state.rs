#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedPanel {
    Request,
    Response,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RequestField {
    Method,
    Url,
    Headers,
    Body,
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
        }
    }

    pub fn next_request_field(&mut self) {
        self.focused_request_field = match self.focused_request_field {
            RequestField::Method => RequestField::Url,
            RequestField::Url => RequestField::Headers,
            RequestField::Headers => RequestField::Body,
            RequestField::Body => RequestField::Method,
        };
    }

    pub fn prev_request_field(&mut self) {
        self.focused_request_field = match self.focused_request_field {
            RequestField::Method => RequestField::Body,
            RequestField::Url => RequestField::Method,
            RequestField::Headers => RequestField::Url,
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
