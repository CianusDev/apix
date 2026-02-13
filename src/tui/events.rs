use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;

use crate::app::App;
use crate::models::Method;

use super::state::{FocusedPanel, RequestField, TuiState};

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
            // Ctrl+C toujours quitter
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(AppEvent::Quit);
            }

            if tui_state.is_editing {
                return Ok(handle_editing(app, tui_state, key.code));
            } else {
                return Ok(handle_navigation(app, tui_state, key.code));
            }
        }
    }
    Ok(AppEvent::None)
}

fn handle_navigation(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Char('q') => AppEvent::Quit,

        KeyCode::Tab => {
            tui_state.focused_panel = match tui_state.focused_panel {
                FocusedPanel::Request => FocusedPanel::Response,
                FocusedPanel::Response => FocusedPanel::Request,
            };
            AppEvent::None
        }

        KeyCode::Up => {
            if tui_state.focused_panel == FocusedPanel::Request {
                tui_state.prev_request_field();
            } else {
                tui_state.scroll_response_up();
            }
            AppEvent::None
        }

        KeyCode::Down => {
            if tui_state.focused_panel == FocusedPanel::Request {
                tui_state.next_request_field();
            } else {
                tui_state.scroll_response_down();
            }
            AppEvent::None
        }

        KeyCode::Enter => {
            if tui_state.focused_panel == FocusedPanel::Request {
                match tui_state.focused_request_field {
                    RequestField::Method => {
                        let next = match app.current_request.method {
                            Method::GET => Method::POST,
                            Method::POST => Method::PUT,
                            Method::PUT => Method::DELETE,
                            Method::DELETE => Method::PATCH,
                            Method::PATCH => Method::GET,
                        };
                        app.set_method(next);
                    }
                    RequestField::Url => {
                        tui_state.is_editing = true;
                        tui_state.url_input = app.current_request.url.clone();
                        tui_state.url_cursor = tui_state.url_input.len();
                    }
                    RequestField::Body => {
                        tui_state.is_editing = true;
                        tui_state.body_input =
                            app.current_request.body.clone().unwrap_or_default();
                        tui_state.body_cursor = tui_state.body_input.len();
                    }
                    RequestField::Headers => {
                        if app.current_request.headers.is_empty() {
                            // Ajouter un nouveau header et entrer en edition
                            app.add_header(String::new(), String::new());
                            tui_state.header_index = 0;
                        }
                        // Editer le header selectionne
                        let (key, value) = &app.current_request.headers[tui_state.header_index];
                        tui_state.header_key_input = key.clone();
                        tui_state.header_value_input = value.clone();
                        tui_state.header_key_cursor = tui_state.header_key_input.len();
                        tui_state.header_value_cursor = tui_state.header_value_input.len();
                        tui_state.editing_header_key = true;
                        tui_state.is_editing = true;
                    }
                }
            }
            AppEvent::None
        }

        KeyCode::Char('s') => {
            if !app.is_loading {
                AppEvent::SendRequest
            } else {
                AppEvent::None
            }
        }

        // Headers : ajouter
        KeyCode::Char('a') => {
            if tui_state.focused_panel == FocusedPanel::Request
                && tui_state.focused_request_field == RequestField::Headers
            {
                app.add_header(String::new(), String::new());
                tui_state.header_index = app.current_request.headers.len() - 1;
                tui_state.header_key_input.clear();
                tui_state.header_value_input.clear();
                tui_state.header_key_cursor = 0;
                tui_state.header_value_cursor = 0;
                tui_state.editing_header_key = true;
                tui_state.is_editing = true;
            }
            AppEvent::None
        }

        // Headers : supprimer
        KeyCode::Char('d') => {
            if tui_state.focused_panel == FocusedPanel::Request
                && tui_state.focused_request_field == RequestField::Headers
                && !app.current_request.headers.is_empty()
            {
                app.remove_header(tui_state.header_index);
                if tui_state.header_index > 0
                    && tui_state.header_index >= app.current_request.headers.len()
                {
                    tui_state.header_index -= 1;
                }
            }
            AppEvent::None
        }

        // Headers : naviguer entre headers
        KeyCode::Left => {
            if tui_state.focused_panel == FocusedPanel::Request
                && tui_state.focused_request_field == RequestField::Headers
                && !app.current_request.headers.is_empty()
            {
                tui_state.header_index = tui_state.header_index.saturating_sub(1);
            }
            AppEvent::None
        }

        KeyCode::Right => {
            if tui_state.focused_panel == FocusedPanel::Request
                && tui_state.focused_request_field == RequestField::Headers
                && !app.current_request.headers.is_empty()
            {
                let max = app.current_request.headers.len() - 1;
                if tui_state.header_index < max {
                    tui_state.header_index += 1;
                }
            }
            AppEvent::None
        }

        _ => AppEvent::None,
    }
}

fn handle_editing(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    if tui_state.focused_request_field == RequestField::Headers {
        return handle_header_editing(app, tui_state, code);
    }

    match code {
        KeyCode::Esc => {
            tui_state.is_editing = false;
        }

        KeyCode::Enter => {
            match tui_state.focused_request_field {
                RequestField::Url => {
                    app.set_url(tui_state.url_input.clone());
                }
                RequestField::Body => {
                    let body = if tui_state.body_input.is_empty() {
                        None
                    } else {
                        Some(tui_state.body_input.clone())
                    };
                    app.set_body(body);
                }
                _ => {}
            }
            tui_state.is_editing = false;
        }

        KeyCode::Char(c) => match tui_state.focused_request_field {
            RequestField::Url => {
                tui_state.url_input.insert(tui_state.url_cursor, c);
                tui_state.url_cursor += 1;
            }
            RequestField::Body => {
                tui_state.body_input.insert(tui_state.body_cursor, c);
                tui_state.body_cursor += 1;
            }
            _ => {}
        },

        KeyCode::Backspace => match tui_state.focused_request_field {
            RequestField::Url => {
                if tui_state.url_cursor > 0 {
                    tui_state.url_input.remove(tui_state.url_cursor - 1);
                    tui_state.url_cursor -= 1;
                }
            }
            RequestField::Body => {
                if tui_state.body_cursor > 0 {
                    tui_state.body_input.remove(tui_state.body_cursor - 1);
                    tui_state.body_cursor -= 1;
                }
            }
            _ => {}
        },

        KeyCode::Left => match tui_state.focused_request_field {
            RequestField::Url => {
                tui_state.url_cursor = tui_state.url_cursor.saturating_sub(1);
            }
            RequestField::Body => {
                tui_state.body_cursor = tui_state.body_cursor.saturating_sub(1);
            }
            _ => {}
        },

        KeyCode::Right => match tui_state.focused_request_field {
            RequestField::Url => {
                if tui_state.url_cursor < tui_state.url_input.len() {
                    tui_state.url_cursor += 1;
                }
            }
            RequestField::Body => {
                if tui_state.body_cursor < tui_state.body_input.len() {
                    tui_state.body_cursor += 1;
                }
            }
            _ => {}
        },

        _ => {}
    }
    AppEvent::None
}

fn handle_header_editing(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Esc => {
            // Annuler : si le header est vide (cle et valeur vides), le supprimer
            let key = tui_state.header_key_input.trim();
            let value = tui_state.header_value_input.trim();
            if key.is_empty() && value.is_empty() {
                app.remove_header(tui_state.header_index);
                if tui_state.header_index > 0
                    && tui_state.header_index >= app.current_request.headers.len()
                {
                    tui_state.header_index = tui_state.header_index.saturating_sub(1);
                }
            }
            tui_state.is_editing = false;
        }

        KeyCode::Enter => {
            // Valider : sauvegarder le header
            let key = tui_state.header_key_input.trim().to_string();
            let value = tui_state.header_value_input.trim().to_string();
            if key.is_empty() {
                // Cle vide → supprimer le header
                app.remove_header(tui_state.header_index);
                if tui_state.header_index > 0
                    && tui_state.header_index >= app.current_request.headers.len()
                {
                    tui_state.header_index = tui_state.header_index.saturating_sub(1);
                }
            } else {
                app.set_header(tui_state.header_index, key, value);
            }
            tui_state.is_editing = false;
        }

        KeyCode::Tab => {
            // Basculer entre cle et valeur
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
            } else {
                if tui_state.header_value_cursor > 0 {
                    tui_state.header_value_input.remove(tui_state.header_value_cursor - 1);
                    tui_state.header_value_cursor -= 1;
                }
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
            } else {
                if tui_state.header_value_cursor < tui_state.header_value_input.len() {
                    tui_state.header_value_cursor += 1;
                }
            }
        }

        _ => {}
    }
    AppEvent::None
}
