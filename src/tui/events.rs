use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::App;
use crate::models::auth::{Auth, AUTH_TYPE_NAMES};
use crate::models::Method;

use super::state::{DrawerPanel, FocusedPanel, RequestTab, ResponseTab, TuiState};

pub enum AppEvent {
    SendRequest,
    Quit,
    None,
}

pub fn handle_events(
    app: &mut App,
    tui_state: &mut TuiState,
    timeout: Duration,
) -> std::io::Result<AppEvent> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            // Ctrl+C always quits
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(AppEvent::Quit);
            }

            if tui_state.is_editing() {
                return Ok(handle_editing(app, tui_state, key));
            } else {
                return Ok(handle_navigation(app, tui_state, key));
            }
        }
    }
    Ok(AppEvent::None)
}

// ── Top-level navigation ──────────────────────────────────────────────────────

fn handle_navigation(app: &mut App, tui_state: &mut TuiState, key: KeyEvent) -> AppEvent {
    let code = key.code;

    // Global shortcuts — always available
    match code {
        KeyCode::Char('q') => return AppEvent::Quit,

        KeyCode::Char('s') => {
            if !app.is_loading {
                return AppEvent::SendRequest;
            }
        }

        // Toggle drawers
        KeyCode::Char('h') => {
            tui_state.drawer = if tui_state.drawer == Some(DrawerPanel::History) {
                None
            } else {
                tui_state.history_search_active = false;
                Some(DrawerPanel::History)
            };
            return AppEvent::None;
        }

        KeyCode::Char('c') => {
            tui_state.drawer = if tui_state.drawer == Some(DrawerPanel::Collections) {
                None
            } else {
                Some(DrawerPanel::Collections)
            };
            return AppEvent::None;
        }

        // e → toggle Environments drawer
        KeyCode::Char('e') => {
            if tui_state.drawer.is_none() || tui_state.drawer == Some(DrawerPanel::Environments) {
                tui_state.drawer = if tui_state.drawer == Some(DrawerPanel::Environments) {
                    None
                } else {
                    Some(DrawerPanel::Environments)
                };
                return AppEvent::None;
            }
        }

        // Esc closes drawer if open
        KeyCode::Esc => {
            if tui_state.drawer.is_some() {
                // If in sub-navigation, go up a level first
                if tui_state.in_collection_requests {
                    tui_state.in_collection_requests = false;
                    tui_state.collection_request_index = 0;
                } else if tui_state.in_environment_vars {
                    tui_state.in_environment_vars = false;
                    tui_state.environment_variable_index = 0;
                } else {
                    tui_state.drawer = None;
                }
                return AppEvent::None;
            }
        }

        // Tab: cycle Request ↔ Response
        KeyCode::Tab => {
            if tui_state.drawer.is_none() {
                tui_state.focused_panel = match tui_state.focused_panel {
                    FocusedPanel::Request => FocusedPanel::Response,
                    FocusedPanel::Response => FocusedPanel::Request,
                };
                return AppEvent::None;
            }
        }

        // Shift+Tab: reverse cycle
        KeyCode::BackTab => {
            if tui_state.drawer.is_none() {
                tui_state.focused_panel = match tui_state.focused_panel {
                    FocusedPanel::Request => FocusedPanel::Response,
                    FocusedPanel::Response => FocusedPanel::Request,
                };
                return AppEvent::None;
            }
        }

        // u → edit URL (only when drawer closed)
        KeyCode::Char('u') => {
            if tui_state.drawer.is_none() {
                tui_state.editing_url = true;
                tui_state.url_input = app.current_request.url.clone();
                tui_state.url_cursor = tui_state.url_input.len();
                return AppEvent::None;
            }
        }

        // Cycle method (only when drawer closed)
        KeyCode::Char('m') => {
            if tui_state.drawer.is_none() {
                let next = match app.current_request.method {
                    Method::GET => Method::POST,
                    Method::POST => Method::PUT,
                    Method::PUT => Method::DELETE,
                    Method::DELETE => Method::PATCH,
                    Method::PATCH => Method::GET,
                };
                app.set_method(next);
                return AppEvent::None;
            }
        }

        _ => {}
    }

    // Drawer active — route to drawer handler
    if let Some(panel) = tui_state.drawer {
        return handle_drawer(app, tui_state, panel, code);
    }

    // Panel active
    match tui_state.focused_panel {
        FocusedPanel::Request => handle_request_panel(app, tui_state, key),
        FocusedPanel::Response => handle_response_panel(app, tui_state, code),
    }
}

// ── Editing dispatcher ────────────────────────────────────────────────────────

fn handle_editing(app: &mut App, tui_state: &mut TuiState, key: KeyEvent) -> AppEvent {
    if tui_state.history_search_active {
        return handle_history_search(tui_state, app, key.code);
    }
    if tui_state.editing_url {
        return handle_url_editing(app, tui_state, key.code);
    }
    if tui_state.is_editing_body {
        return handle_body_editing(app, tui_state, key);
    }
    if tui_state.editing_header {
        return handle_header_editing(app, tui_state, key.code);
    }
    if tui_state.editing_param {
        return handle_param_editing(app, tui_state, key.code);
    }
    if tui_state.editing_auth {
        return handle_auth_editing(app, tui_state, key.code);
    }
    if tui_state.editing_variable {
        return handle_variable_editing(app, tui_state, key.code);
    }
    if tui_state.editing_collection_name {
        return handle_collection_name_editing(app, tui_state, key.code);
    }
    if tui_state.editing_environment_name {
        return handle_environment_name_editing(app, tui_state, key.code);
    }
    AppEvent::None
}

// ── Drawer handler ────────────────────────────────────────────────────────────

fn handle_drawer(
    app: &mut App,
    tui_state: &mut TuiState,
    panel: DrawerPanel,
    code: KeyCode,
) -> AppEvent {
    match panel {
        DrawerPanel::History => handle_drawer_history(app, tui_state, code),
        DrawerPanel::Collections => handle_drawer_collections(app, tui_state, code),
        DrawerPanel::Environments => handle_drawer_environments(app, tui_state, code),
    }
}

fn handle_drawer_history(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    if tui_state.history_search_active {
        return handle_history_search(tui_state, app, code);
    }

    let filtered = tui_state.history_filtered_indices(&app.history.entries);

    match code {
        KeyCode::Up => {
            tui_state.history_index = tui_state.history_index.saturating_sub(1);
        }
        KeyCode::Down => {
            if !filtered.is_empty() {
                let max = filtered.len() - 1;
                if tui_state.history_index < max {
                    tui_state.history_index += 1;
                }
            }
        }
        KeyCode::Enter => {
            if let Some(&original_idx) = filtered.get(tui_state.history_index) {
                app.load_history_entry(original_idx);
                tui_state.url_input = app.current_request.url.clone();
                tui_state.url_cursor = tui_state.url_input.len();
                tui_state.body_input = app.current_request.body.clone().unwrap_or_default();
                tui_state.body_cursor = tui_state.body_input.len();
                tui_state.response_scroll = 0;
                tui_state.focused_panel = FocusedPanel::Request;
                tui_state.drawer = None;
            }
        }
        KeyCode::Char('d') => {
            if let Some(&original_idx) = filtered.get(tui_state.history_index) {
                app.remove_history_entry(original_idx);
                let new_filtered = tui_state.history_filtered_indices(&app.history.entries);
                if tui_state.history_index > 0 && tui_state.history_index >= new_filtered.len() {
                    tui_state.history_index -= 1;
                }
            }
        }
        KeyCode::Char('/') => {
            tui_state.history_search_active = true;
        }
        KeyCode::Esc => {
            if !tui_state.history_search_input.is_empty() {
                tui_state.history_search_input.clear();
                tui_state.history_index = 0;
            } else {
                tui_state.drawer = None;
            }
        }
        _ => {}
    }
    AppEvent::None
}

fn handle_drawer_collections(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    if tui_state.editing_collection_name {
        return handle_collection_name_editing(app, tui_state, code);
    }

    if tui_state.in_collection_requests {
        let col_idx = tui_state.collection_index;
        let col_len = app.collections.items.get(col_idx).map(|c| c.entries.len()).unwrap_or(0);

        match code {
            KeyCode::Up => {
                tui_state.collection_request_index =
                    tui_state.collection_request_index.saturating_sub(1);
            }
            KeyCode::Down => {
                if col_len > 0 && tui_state.collection_request_index < col_len - 1 {
                    tui_state.collection_request_index += 1;
                }
            }
            KeyCode::Enter => {
                app.load_collection_entry(col_idx, tui_state.collection_request_index);
                tui_state.url_input = app.current_request.url.clone();
                tui_state.url_cursor = tui_state.url_input.len();
                tui_state.body_input = app.current_request.body.clone().unwrap_or_default();
                tui_state.body_cursor = tui_state.body_input.len();
                tui_state.response_scroll = 0;
                tui_state.focused_panel = FocusedPanel::Request;
                tui_state.drawer = None;
            }
            KeyCode::Char('a') => {
                if !app.current_request.url.is_empty() {
                    let name = format!(
                        "{} {}",
                        app.current_request.method, app.current_request.url
                    );
                    app.add_request_to_collection(col_idx, name);
                }
            }
            KeyCode::Char('d') => {
                let should_adjust = if let Some(col) = app.collections.items.get_mut(col_idx) {
                    if !col.entries.is_empty() {
                        col.remove_entry(tui_state.collection_request_index);
                        let adjust = tui_state.collection_request_index > 0
                            && tui_state.collection_request_index >= col.entries.len();
                        Some(adjust)
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(adjust) = should_adjust {
                    app.save_collections();
                    if adjust {
                        tui_state.collection_request_index -= 1;
                    }
                }
            }
            KeyCode::Esc => {
                tui_state.in_collection_requests = false;
                tui_state.collection_request_index = 0;
            }
            _ => {}
        }
    } else {
        let col_count = app.collections.items.len();

        match code {
            KeyCode::Up => {
                tui_state.collection_index = tui_state.collection_index.saturating_sub(1);
            }
            KeyCode::Down => {
                if col_count > 0 && tui_state.collection_index < col_count - 1 {
                    tui_state.collection_index += 1;
                }
            }
            KeyCode::Enter => {
                if !app.collections.items.is_empty() {
                    tui_state.in_collection_requests = true;
                    tui_state.collection_request_index = 0;
                }
            }
            KeyCode::Char('n') => {
                tui_state.editing_collection_name = true;
                tui_state.collection_name_input.clear();
                tui_state.collection_name_cursor = 0;
            }
            KeyCode::Char('d') => {
                if !app.collections.items.is_empty() {
                    app.collections.remove_collection(tui_state.collection_index);
                    app.save_collections();
                    if tui_state.collection_index > 0
                        && tui_state.collection_index >= app.collections.items.len()
                    {
                        tui_state.collection_index -= 1;
                    }
                }
            }
            KeyCode::Esc => {
                tui_state.drawer = None;
            }
            _ => {}
        }
    }
    AppEvent::None
}

fn handle_drawer_environments(
    app: &mut App,
    tui_state: &mut TuiState,
    code: KeyCode,
) -> AppEvent {
    if tui_state.editing_environment_name {
        return handle_environment_name_editing(app, tui_state, code);
    }
    if tui_state.editing_variable {
        return handle_variable_editing(app, tui_state, code);
    }

    if tui_state.in_environment_vars {
        let env_idx = tui_state.environment_index;
        let keys = app
            .environments
            .items
            .get(env_idx)
            .map(|e| e.sorted_keys())
            .unwrap_or_default();

        match code {
            KeyCode::Up => {
                tui_state.environment_variable_index =
                    tui_state.environment_variable_index.saturating_sub(1);
            }
            KeyCode::Down => {
                if !keys.is_empty() && tui_state.environment_variable_index < keys.len() - 1 {
                    tui_state.environment_variable_index += 1;
                }
            }
            KeyCode::Enter => {
                if !keys.is_empty() {
                    let key = &keys[tui_state.environment_variable_index];
                    let env = &app.environments.items[env_idx];
                    let value = env.variables.get(key).cloned().unwrap_or_default();
                    tui_state.variable_key_input = key.clone();
                    tui_state.variable_value_input = value;
                    tui_state.variable_key_cursor = tui_state.variable_key_input.len();
                    tui_state.variable_value_cursor = tui_state.variable_value_input.len();
                    tui_state.editing_variable_key = true;
                    tui_state.editing_variable = true;
                }
            }
            KeyCode::Char('a') => {
                tui_state.variable_key_input.clear();
                tui_state.variable_value_input.clear();
                tui_state.variable_key_cursor = 0;
                tui_state.variable_value_cursor = 0;
                tui_state.editing_variable_key = true;
                tui_state.editing_variable = true;
            }
            KeyCode::Char('d') => {
                if !keys.is_empty() {
                    let key = keys[tui_state.environment_variable_index].clone();
                    if let Some(env) = app.environments.items.get_mut(env_idx) {
                        env.remove_variable(&key);
                    }
                    app.save_environments();
                    let new_len = app.environments.items.get(env_idx)
                        .map(|e| e.variables.len())
                        .unwrap_or(0);
                    if tui_state.environment_variable_index > 0
                        && tui_state.environment_variable_index >= new_len
                    {
                        tui_state.environment_variable_index -= 1;
                    }
                }
            }
            KeyCode::Esc => {
                tui_state.in_environment_vars = false;
                tui_state.environment_variable_index = 0;
            }
            _ => {}
        }
    } else {
        let env_count = app.environments.items.len();

        match code {
            KeyCode::Up => {
                tui_state.environment_index = tui_state.environment_index.saturating_sub(1);
            }
            KeyCode::Down => {
                if env_count > 0 && tui_state.environment_index < env_count - 1 {
                    tui_state.environment_index += 1;
                }
            }
            KeyCode::Enter => {
                if !app.environments.items.is_empty() {
                    let idx = tui_state.environment_index;
                    if app.active_environment == Some(idx) {
                        app.active_environment = None;
                    } else {
                        app.active_environment = Some(idx);
                    }
                }
            }
            KeyCode::Char('v') => {
                if !app.environments.items.is_empty() {
                    tui_state.in_environment_vars = true;
                    tui_state.environment_variable_index = 0;
                }
            }
            KeyCode::Char('n') => {
                tui_state.editing_environment_name = true;
                tui_state.environment_name_input.clear();
                tui_state.environment_name_cursor = 0;
            }
            KeyCode::Char('d') => {
                if !app.environments.items.is_empty() {
                    let idx = tui_state.environment_index;
                    if app.active_environment == Some(idx) {
                        app.active_environment = None;
                    } else if let Some(active) = app.active_environment {
                        if idx < active {
                            app.active_environment = Some(active - 1);
                        }
                    }
                    app.environments.remove_environment(idx);
                    app.save_environments();
                    if tui_state.environment_index > 0
                        && tui_state.environment_index >= app.environments.items.len()
                    {
                        tui_state.environment_index -= 1;
                    }
                }
            }
            KeyCode::Esc => {
                tui_state.drawer = None;
            }
            _ => {}
        }
    }
    AppEvent::None
}

// ── Request panel handler ─────────────────────────────────────────────────────

fn handle_request_panel(app: &mut App, tui_state: &mut TuiState, key: KeyEvent) -> AppEvent {
    let code = key.code;

    // Numbered tab shortcuts
    match code {
        KeyCode::Char('1') => {
            tui_state.request_tab = RequestTab::Params;
            return AppEvent::None;
        }
        KeyCode::Char('2') => {
            tui_state.request_tab = RequestTab::Headers;
            return AppEvent::None;
        }
        KeyCode::Char('3') => {
            tui_state.request_tab = RequestTab::Body;
            return AppEvent::None;
        }
        KeyCode::Char('4') => {
            tui_state.request_tab = RequestTab::Auth;
            return AppEvent::None;
        }
        // Legacy [ / ] cycle tabs
        KeyCode::Char('[') => {
            tui_state.prev_request_tab();
            return AppEvent::None;
        }
        KeyCode::Char(']') => {
            tui_state.next_request_tab();
            return AppEvent::None;
        }
        _ => {}
    }

    // Dispatch to tab-specific handler
    match tui_state.request_tab {
        RequestTab::Params => handle_params_keys(app, tui_state, code),
        RequestTab::Headers => handle_headers_keys(app, tui_state, code),
        RequestTab::Body => handle_body_keys(app, tui_state, code),
        RequestTab::Auth => handle_auth_keys(app, tui_state, code),
    }
}

fn handle_params_keys(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Up => {
            tui_state.param_index = tui_state.param_index.saturating_sub(1);
        }
        KeyCode::Down => {
            if !app.current_request.params.is_empty() {
                let max = app.current_request.params.len() - 1;
                if tui_state.param_index < max {
                    tui_state.param_index += 1;
                }
            }
        }
        KeyCode::Enter => {
            if app.current_request.params.is_empty() {
                app.add_param(String::new(), String::new());
                tui_state.param_index = 0;
            }
            if !app.current_request.params.is_empty() {
                let (k, v) = &app.current_request.params[tui_state.param_index];
                tui_state.param_key_input = k.clone();
                tui_state.param_value_input = v.clone();
                tui_state.param_key_cursor = tui_state.param_key_input.len();
                tui_state.param_value_cursor = tui_state.param_value_input.len();
                tui_state.editing_param_key = true;
                tui_state.editing_param = true;
            }
        }
        KeyCode::Char('a') => {
            app.add_param(String::new(), String::new());
            tui_state.param_index = app.current_request.params.len() - 1;
            tui_state.param_key_input.clear();
            tui_state.param_value_input.clear();
            tui_state.param_key_cursor = 0;
            tui_state.param_value_cursor = 0;
            tui_state.editing_param_key = true;
            tui_state.editing_param = true;
        }
        KeyCode::Char('d') => {
            if !app.current_request.params.is_empty() {
                app.remove_param(tui_state.param_index);
                if tui_state.param_index > 0
                    && tui_state.param_index >= app.current_request.params.len()
                {
                    tui_state.param_index -= 1;
                }
            }
        }
        _ => {}
    }
    AppEvent::None
}

fn handle_headers_keys(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Up => {
            tui_state.header_index = tui_state.header_index.saturating_sub(1);
        }
        KeyCode::Down => {
            if !app.current_request.headers.is_empty() {
                let max = app.current_request.headers.len() - 1;
                if tui_state.header_index < max {
                    tui_state.header_index += 1;
                }
            }
        }
        KeyCode::Enter => {
            if app.current_request.headers.is_empty() {
                app.add_header(String::new(), String::new());
                tui_state.header_index = 0;
            }
            if !app.current_request.headers.is_empty() {
                let (k, v) = &app.current_request.headers[tui_state.header_index];
                tui_state.header_key_input = k.clone();
                tui_state.header_value_input = v.clone();
                tui_state.header_key_cursor = tui_state.header_key_input.len();
                tui_state.header_value_cursor = tui_state.header_value_input.len();
                tui_state.editing_header_key = true;
                tui_state.editing_header = true;
            }
        }
        KeyCode::Char('a') => {
            app.add_header(String::new(), String::new());
            tui_state.header_index = app.current_request.headers.len() - 1;
            tui_state.header_key_input.clear();
            tui_state.header_value_input.clear();
            tui_state.header_key_cursor = 0;
            tui_state.header_value_cursor = 0;
            tui_state.editing_header_key = true;
            tui_state.editing_header = true;
        }
        KeyCode::Char('d') => {
            if !app.current_request.headers.is_empty() {
                app.remove_header(tui_state.header_index);
                if tui_state.header_index > 0
                    && tui_state.header_index >= app.current_request.headers.len()
                {
                    tui_state.header_index -= 1;
                }
            }
        }
        _ => {}
    }
    AppEvent::None
}

fn handle_body_keys(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Enter => {
            let raw = app.current_request.body.clone().unwrap_or_default();
            tui_state.body_input =
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                    serde_json::to_string_pretty(&val).unwrap_or(raw)
                } else {
                    raw
                };
            tui_state.body_cursor = tui_state.body_input.len();
            tui_state.is_editing_body = true;
        }
        _ => {}
    }
    AppEvent::None
}

fn handle_auth_keys(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Enter => {
            match &app.current_request.auth {
                None => {
                    tui_state.auth_type_index = 0;
                }
                Some(Auth::BearerToken { token }) => {
                    tui_state.auth_type_index = 1;
                    tui_state.auth_token_input = token.clone();
                    tui_state.auth_token_cursor = token.len();
                }
                Some(Auth::BasicAuth { username, password }) => {
                    tui_state.auth_type_index = 2;
                    tui_state.auth_username_input = username.clone();
                    tui_state.auth_password_input = password.clone();
                    tui_state.auth_username_cursor = username.len();
                    tui_state.auth_password_cursor = password.len();
                    tui_state.auth_editing_username = true;
                }
                Some(Auth::ApiKey { header, value }) => {
                    tui_state.auth_type_index = 3;
                    tui_state.auth_key_name_input = header.clone();
                    tui_state.auth_key_value_input = value.clone();
                    tui_state.auth_key_name_cursor = header.len();
                    tui_state.auth_key_value_cursor = value.len();
                    tui_state.auth_editing_key_name = true;
                }
            }
            tui_state.editing_auth = true;
        }
        _ => {}
    }
    AppEvent::None
}

// ── Response panel handler ────────────────────────────────────────────────────

fn handle_response_panel(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        // Numbered tab shortcuts
        KeyCode::Char('1') => {
            tui_state.response_tab = ResponseTab::Body;
        }
        KeyCode::Char('2') => {
            tui_state.response_tab = ResponseTab::Headers;
        }
        KeyCode::Char('3') => {
            tui_state.response_tab = ResponseTab::Cookies;
        }
        // Legacy [ / ] cycle tabs
        KeyCode::Char('[') => {
            tui_state.prev_response_tab();
        }
        KeyCode::Char(']') => {
            tui_state.next_response_tab();
        }
        KeyCode::Up => match tui_state.response_tab {
            ResponseTab::Body => tui_state.scroll_response_up(),
            ResponseTab::Headers => {
                tui_state.response_headers_scroll =
                    tui_state.response_headers_scroll.saturating_sub(1);
            }
            ResponseTab::Cookies => {}
        },
        KeyCode::Down => match tui_state.response_tab {
            ResponseTab::Body => tui_state.scroll_response_down(),
            ResponseTab::Headers => {
                tui_state.response_headers_scroll += 1;
            }
            ResponseTab::Cookies => {}
        },
        KeyCode::Char('y') => {
            app.copy_response_to_clipboard();
        }
        KeyCode::Char('w') => {
            app.save_response_to_file();
        }
        _ => {}
    }
    AppEvent::None
}

// ── URL editing ───────────────────────────────────────────────────────────────

fn handle_url_editing(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Esc => {
            tui_state.editing_url = false;
        }
        KeyCode::Enter => {
            app.set_url(tui_state.url_input.clone());
            tui_state.editing_url = false;
        }
        KeyCode::Char(c) => {
            tui_state.url_input.insert(tui_state.url_cursor, c);
            tui_state.url_cursor += 1;
        }
        KeyCode::Backspace => {
            if tui_state.url_cursor > 0 {
                tui_state.url_input.remove(tui_state.url_cursor - 1);
                tui_state.url_cursor -= 1;
            }
        }
        KeyCode::Left => {
            tui_state.url_cursor = tui_state.url_cursor.saturating_sub(1);
        }
        KeyCode::Right => {
            if tui_state.url_cursor < tui_state.url_input.len() {
                tui_state.url_cursor += 1;
            }
        }
        _ => {}
    }
    AppEvent::None
}

// ── Body editing (textarea) ───────────────────────────────────────────────────

fn handle_body_editing(app: &mut App, tui_state: &mut TuiState, key: KeyEvent) -> AppEvent {
    let code = key.code;
    match code {
        KeyCode::Esc => {
            let body = if tui_state.body_input.is_empty() {
                None
            } else {
                Some(tui_state.body_input.clone())
            };
            app.set_body(body);
            tui_state.is_editing_body = false;
        }
        KeyCode::Enter => body_insert_newline(tui_state),
        KeyCode::Tab => {
            tui_state.body_input.insert_str(tui_state.body_cursor, "  ");
            tui_state.body_cursor += 2;
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            body_format_json(tui_state);
        }
        KeyCode::Up => body_move_up(tui_state),
        KeyCode::Down => body_move_down(tui_state),
        KeyCode::Left => {
            tui_state.body_cursor = tui_state.body_cursor.saturating_sub(1);
        }
        KeyCode::Right => {
            if tui_state.body_cursor < tui_state.body_input.len() {
                tui_state.body_cursor += 1;
            }
        }
        KeyCode::Backspace => {
            if tui_state.body_cursor > 0 {
                tui_state.body_input.remove(tui_state.body_cursor - 1);
                tui_state.body_cursor -= 1;
            }
        }
        KeyCode::Char(c) => {
            tui_state.body_input.insert(tui_state.body_cursor, c);
            tui_state.body_cursor += 1;
        }
        _ => {}
    }
    AppEvent::None
}

fn body_insert_newline(tui_state: &mut TuiState) {
    let cursor = tui_state.body_cursor;
    let line_start = tui_state.body_input[..cursor]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let line_before = &tui_state.body_input[line_start..cursor];
    let indent: String = line_before.chars().take_while(|c| *c == ' ').collect();
    let trimmed = line_before.trim_end();
    let extra = if trimmed.ends_with('{') || trimmed.ends_with('[') {
        "  "
    } else {
        ""
    };
    let new_indent = format!("{}{}", indent, extra);
    let char_after = tui_state.body_input[cursor..].chars().next();
    let close_block = !extra.is_empty() && (char_after == Some('}') || char_after == Some(']'));

    if close_block {
        let insert = format!("\n{}\n{}", new_indent, indent);
        tui_state.body_input.insert_str(cursor, &insert);
        tui_state.body_cursor = cursor + 1 + new_indent.len();
    } else {
        let insert = format!("\n{}", new_indent);
        let len = insert.len();
        tui_state.body_input.insert_str(cursor, &insert);
        tui_state.body_cursor = cursor + len;
    }
}

fn body_format_json(tui_state: &mut TuiState) {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&tui_state.body_input) {
        if let Ok(pretty) = serde_json::to_string_pretty(&value) {
            tui_state.body_input = pretty;
            tui_state.body_cursor = tui_state.body_input.len();
        }
    }
}

fn body_move_up(tui_state: &mut TuiState) {
    let cursor = tui_state.body_cursor;
    let line_start = tui_state.body_input[..cursor]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let col = cursor - line_start;
    if line_start == 0 {
        tui_state.body_cursor = 0;
        return;
    }
    let prev_end = line_start - 1;
    let prev_start = tui_state.body_input[..prev_end]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let prev_len = prev_end - prev_start;
    tui_state.body_cursor = prev_start + col.min(prev_len);
}

fn body_move_down(tui_state: &mut TuiState) {
    let cursor = tui_state.body_cursor;
    let line_start = tui_state.body_input[..cursor]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let col = cursor - line_start;
    if let Some(rel) = tui_state.body_input[cursor..].find('\n') {
        let next_start = cursor + rel + 1;
        let next_end = tui_state.body_input[next_start..]
            .find('\n')
            .map(|i| next_start + i)
            .unwrap_or(tui_state.body_input.len());
        let next_len = next_end - next_start;
        tui_state.body_cursor = next_start + col.min(next_len);
    }
}

// ── Param editing ─────────────────────────────────────────────────────────────

fn handle_param_editing(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Esc => {
            let key = tui_state.param_key_input.trim();
            let val = tui_state.param_value_input.trim();
            if key.is_empty() && val.is_empty() {
                app.remove_param(tui_state.param_index);
                if tui_state.param_index > 0
                    && tui_state.param_index >= app.current_request.params.len()
                {
                    tui_state.param_index = tui_state.param_index.saturating_sub(1);
                }
            }
            tui_state.editing_param = false;
        }
        KeyCode::Enter => {
            let key = tui_state.param_key_input.trim().to_string();
            let val = tui_state.param_value_input.trim().to_string();
            if key.is_empty() {
                app.remove_param(tui_state.param_index);
                if tui_state.param_index > 0
                    && tui_state.param_index >= app.current_request.params.len()
                {
                    tui_state.param_index = tui_state.param_index.saturating_sub(1);
                }
            } else {
                app.set_param(tui_state.param_index, key, val);
            }
            tui_state.editing_param = false;
        }
        KeyCode::Tab => {
            tui_state.editing_param_key = !tui_state.editing_param_key;
        }
        KeyCode::Char(c) => {
            if tui_state.editing_param_key {
                tui_state.param_key_input.insert(tui_state.param_key_cursor, c);
                tui_state.param_key_cursor += 1;
            } else {
                tui_state.param_value_input.insert(tui_state.param_value_cursor, c);
                tui_state.param_value_cursor += 1;
            }
        }
        KeyCode::Backspace => {
            if tui_state.editing_param_key {
                if tui_state.param_key_cursor > 0 {
                    tui_state.param_key_input.remove(tui_state.param_key_cursor - 1);
                    tui_state.param_key_cursor -= 1;
                }
            } else if tui_state.param_value_cursor > 0 {
                tui_state.param_value_input.remove(tui_state.param_value_cursor - 1);
                tui_state.param_value_cursor -= 1;
            }
        }
        KeyCode::Left => {
            if tui_state.editing_param_key {
                tui_state.param_key_cursor = tui_state.param_key_cursor.saturating_sub(1);
            } else {
                tui_state.param_value_cursor = tui_state.param_value_cursor.saturating_sub(1);
            }
        }
        KeyCode::Right => {
            if tui_state.editing_param_key {
                if tui_state.param_key_cursor < tui_state.param_key_input.len() {
                    tui_state.param_key_cursor += 1;
                }
            } else if tui_state.param_value_cursor < tui_state.param_value_input.len() {
                tui_state.param_value_cursor += 1;
            }
        }
        _ => {}
    }
    AppEvent::None
}

// ── Header editing ────────────────────────────────────────────────────────────

fn handle_header_editing(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Esc => {
            let key = tui_state.header_key_input.trim();
            let val = tui_state.header_value_input.trim();
            if key.is_empty() && val.is_empty() {
                app.remove_header(tui_state.header_index);
                if tui_state.header_index > 0
                    && tui_state.header_index >= app.current_request.headers.len()
                {
                    tui_state.header_index = tui_state.header_index.saturating_sub(1);
                }
            }
            tui_state.editing_header = false;
        }
        KeyCode::Enter => {
            let key = tui_state.header_key_input.trim().to_string();
            let val = tui_state.header_value_input.trim().to_string();
            if key.is_empty() {
                app.remove_header(tui_state.header_index);
                if tui_state.header_index > 0
                    && tui_state.header_index >= app.current_request.headers.len()
                {
                    tui_state.header_index = tui_state.header_index.saturating_sub(1);
                }
            } else {
                app.set_header(tui_state.header_index, key, val);
            }
            tui_state.editing_header = false;
        }
        KeyCode::Tab => {
            tui_state.editing_header_key = !tui_state.editing_header_key;
        }
        KeyCode::Char(c) => {
            if tui_state.editing_header_key {
                tui_state.header_key_input.insert(tui_state.header_key_cursor, c);
                tui_state.header_key_cursor += 1;
            } else {
                tui_state.header_value_input.insert(tui_state.header_value_cursor, c);
                tui_state.header_value_cursor += 1;
            }
        }
        KeyCode::Backspace => {
            if tui_state.editing_header_key {
                if tui_state.header_key_cursor > 0 {
                    tui_state.header_key_input.remove(tui_state.header_key_cursor - 1);
                    tui_state.header_key_cursor -= 1;
                }
            } else if tui_state.header_value_cursor > 0 {
                tui_state.header_value_input.remove(tui_state.header_value_cursor - 1);
                tui_state.header_value_cursor -= 1;
            }
        }
        KeyCode::Left => {
            if tui_state.editing_header_key {
                tui_state.header_key_cursor = tui_state.header_key_cursor.saturating_sub(1);
            } else {
                tui_state.header_value_cursor = tui_state.header_value_cursor.saturating_sub(1);
            }
        }
        KeyCode::Right => {
            if tui_state.editing_header_key {
                if tui_state.header_key_cursor < tui_state.header_key_input.len() {
                    tui_state.header_key_cursor += 1;
                }
            } else if tui_state.header_value_cursor < tui_state.header_value_input.len() {
                tui_state.header_value_cursor += 1;
            }
        }
        _ => {}
    }
    AppEvent::None
}

// ── Auth editing ──────────────────────────────────────────────────────────────

fn handle_auth_editing(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Esc => {
            tui_state.editing_auth = false;
        }
        KeyCode::Left => {
            if tui_state.auth_type_index == 0 {
                tui_state.auth_type_index = AUTH_TYPE_NAMES.len() - 1;
            } else {
                tui_state.auth_type_index -= 1;
            }
        }
        KeyCode::Right => {
            tui_state.auth_type_index = (tui_state.auth_type_index + 1) % AUTH_TYPE_NAMES.len();
        }
        KeyCode::Enter => {
            let auth = match tui_state.auth_type_index {
                0 => None,
                1 => Some(Auth::BearerToken {
                    token: tui_state.auth_token_input.clone(),
                }),
                2 => Some(Auth::BasicAuth {
                    username: tui_state.auth_username_input.clone(),
                    password: tui_state.auth_password_input.clone(),
                }),
                3 => Some(Auth::ApiKey {
                    header: tui_state.auth_key_name_input.clone(),
                    value: tui_state.auth_key_value_input.clone(),
                }),
                _ => None,
            };
            app.set_auth(auth);
            tui_state.editing_auth = false;
        }
        KeyCode::Tab => match tui_state.auth_type_index {
            2 => tui_state.auth_editing_username = !tui_state.auth_editing_username,
            3 => tui_state.auth_editing_key_name = !tui_state.auth_editing_key_name,
            _ => {}
        },
        KeyCode::Char(c) => match tui_state.auth_type_index {
            1 => {
                tui_state.auth_token_input.insert(tui_state.auth_token_cursor, c);
                tui_state.auth_token_cursor += 1;
            }
            2 => {
                if tui_state.auth_editing_username {
                    tui_state.auth_username_input.insert(tui_state.auth_username_cursor, c);
                    tui_state.auth_username_cursor += 1;
                } else {
                    tui_state.auth_password_input.insert(tui_state.auth_password_cursor, c);
                    tui_state.auth_password_cursor += 1;
                }
            }
            3 => {
                if tui_state.auth_editing_key_name {
                    tui_state.auth_key_name_input.insert(tui_state.auth_key_name_cursor, c);
                    tui_state.auth_key_name_cursor += 1;
                } else {
                    tui_state.auth_key_value_input.insert(tui_state.auth_key_value_cursor, c);
                    tui_state.auth_key_value_cursor += 1;
                }
            }
            _ => {}
        },
        KeyCode::Backspace => match tui_state.auth_type_index {
            1 => {
                if tui_state.auth_token_cursor > 0 {
                    tui_state.auth_token_input.remove(tui_state.auth_token_cursor - 1);
                    tui_state.auth_token_cursor -= 1;
                }
            }
            2 => {
                if tui_state.auth_editing_username {
                    if tui_state.auth_username_cursor > 0 {
                        tui_state.auth_username_input.remove(tui_state.auth_username_cursor - 1);
                        tui_state.auth_username_cursor -= 1;
                    }
                } else if tui_state.auth_password_cursor > 0 {
                    tui_state.auth_password_input.remove(tui_state.auth_password_cursor - 1);
                    tui_state.auth_password_cursor -= 1;
                }
            }
            3 => {
                if tui_state.auth_editing_key_name {
                    if tui_state.auth_key_name_cursor > 0 {
                        tui_state.auth_key_name_input.remove(tui_state.auth_key_name_cursor - 1);
                        tui_state.auth_key_name_cursor -= 1;
                    }
                } else if tui_state.auth_key_value_cursor > 0 {
                    tui_state.auth_key_value_input.remove(tui_state.auth_key_value_cursor - 1);
                    tui_state.auth_key_value_cursor -= 1;
                }
            }
            _ => {}
        },
        _ => {}
    }
    AppEvent::None
}

// ── Variable editing ──────────────────────────────────────────────────────────

fn handle_variable_editing(
    app: &mut App,
    tui_state: &mut TuiState,
    code: KeyCode,
) -> AppEvent {
    match code {
        KeyCode::Esc => {
            tui_state.editing_variable = false;
        }
        KeyCode::Enter => {
            let key = tui_state.variable_key_input.trim().to_string();
            let value = tui_state.variable_value_input.trim().to_string();
            if !key.is_empty() {
                let env_idx = tui_state.environment_index;
                if let Some(env) = app.environments.items.get_mut(env_idx) {
                    env.set_variable(key, value);
                }
                app.save_environments();
            }
            tui_state.editing_variable = false;
        }
        KeyCode::Tab => {
            tui_state.editing_variable_key = !tui_state.editing_variable_key;
        }
        KeyCode::Char(c) => {
            if tui_state.editing_variable_key {
                tui_state.variable_key_input.insert(tui_state.variable_key_cursor, c);
                tui_state.variable_key_cursor += 1;
            } else {
                tui_state.variable_value_input.insert(tui_state.variable_value_cursor, c);
                tui_state.variable_value_cursor += 1;
            }
        }
        KeyCode::Backspace => {
            if tui_state.editing_variable_key {
                if tui_state.variable_key_cursor > 0 {
                    tui_state.variable_key_input.remove(tui_state.variable_key_cursor - 1);
                    tui_state.variable_key_cursor -= 1;
                }
            } else if tui_state.variable_value_cursor > 0 {
                tui_state.variable_value_input.remove(tui_state.variable_value_cursor - 1);
                tui_state.variable_value_cursor -= 1;
            }
        }
        KeyCode::Left => {
            if tui_state.editing_variable_key {
                tui_state.variable_key_cursor = tui_state.variable_key_cursor.saturating_sub(1);
            } else {
                tui_state.variable_value_cursor =
                    tui_state.variable_value_cursor.saturating_sub(1);
            }
        }
        KeyCode::Right => {
            if tui_state.editing_variable_key {
                if tui_state.variable_key_cursor < tui_state.variable_key_input.len() {
                    tui_state.variable_key_cursor += 1;
                }
            } else if tui_state.variable_value_cursor < tui_state.variable_value_input.len() {
                tui_state.variable_value_cursor += 1;
            }
        }
        _ => {}
    }
    AppEvent::None
}

// ── Collection name editing ───────────────────────────────────────────────────

fn handle_collection_name_editing(
    app: &mut App,
    tui_state: &mut TuiState,
    code: KeyCode,
) -> AppEvent {
    match code {
        KeyCode::Esc => {
            tui_state.editing_collection_name = false;
        }
        KeyCode::Enter => {
            let name = tui_state.collection_name_input.trim().to_string();
            if !name.is_empty() {
                app.collections.add_collection(name);
                app.save_collections();
                tui_state.collection_index = app.collections.items.len() - 1;
            }
            tui_state.editing_collection_name = false;
        }
        KeyCode::Char(c) => {
            tui_state
                .collection_name_input
                .insert(tui_state.collection_name_cursor, c);
            tui_state.collection_name_cursor += 1;
        }
        KeyCode::Backspace => {
            if tui_state.collection_name_cursor > 0 {
                tui_state
                    .collection_name_input
                    .remove(tui_state.collection_name_cursor - 1);
                tui_state.collection_name_cursor -= 1;
            }
        }
        KeyCode::Left => {
            tui_state.collection_name_cursor =
                tui_state.collection_name_cursor.saturating_sub(1);
        }
        KeyCode::Right => {
            if tui_state.collection_name_cursor < tui_state.collection_name_input.len() {
                tui_state.collection_name_cursor += 1;
            }
        }
        _ => {}
    }
    AppEvent::None
}

// ── Environment name editing ──────────────────────────────────────────────────

fn handle_environment_name_editing(
    app: &mut App,
    tui_state: &mut TuiState,
    code: KeyCode,
) -> AppEvent {
    match code {
        KeyCode::Esc => {
            tui_state.editing_environment_name = false;
        }
        KeyCode::Enter => {
            let name = tui_state.environment_name_input.trim().to_string();
            if !name.is_empty() {
                app.environments.add_environment(name);
                app.save_environments();
                tui_state.environment_index = app.environments.items.len() - 1;
            }
            tui_state.editing_environment_name = false;
        }
        KeyCode::Char(c) => {
            tui_state
                .environment_name_input
                .insert(tui_state.environment_name_cursor, c);
            tui_state.environment_name_cursor += 1;
        }
        KeyCode::Backspace => {
            if tui_state.environment_name_cursor > 0 {
                tui_state
                    .environment_name_input
                    .remove(tui_state.environment_name_cursor - 1);
                tui_state.environment_name_cursor -= 1;
            }
        }
        KeyCode::Left => {
            tui_state.environment_name_cursor =
                tui_state.environment_name_cursor.saturating_sub(1);
        }
        KeyCode::Right => {
            if tui_state.environment_name_cursor < tui_state.environment_name_input.len() {
                tui_state.environment_name_cursor += 1;
            }
        }
        _ => {}
    }
    AppEvent::None
}

// ── History search ────────────────────────────────────────────────────────────

fn handle_history_search(tui_state: &mut TuiState, app: &App, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Enter | KeyCode::Esc => {
            tui_state.history_search_active = false;
            tui_state.history_index = 0;
        }
        KeyCode::Backspace => {
            tui_state.history_search_input.pop();
            tui_state.history_index = 0;
        }
        KeyCode::Up => {
            tui_state.history_index = tui_state.history_index.saturating_sub(1);
        }
        KeyCode::Down => {
            let filtered = tui_state.history_filtered_indices(&app.history.entries);
            if !filtered.is_empty() && tui_state.history_index < filtered.len() - 1 {
                tui_state.history_index += 1;
            }
        }
        KeyCode::Char(c) => {
            tui_state.history_search_input.push(c);
            tui_state.history_index = 0;
        }
        _ => {}
    }
    AppEvent::None
}
