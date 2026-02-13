use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;

use crate::app::App;
use crate::models::Method;

use super::state::{CollectionsView, FocusedPanel, RequestField, TuiState};

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
    // Mode historique : gestion separee
    if tui_state.show_history {
        return handle_history_navigation(app, tui_state, code);
    }

    // Mode collections : gestion separee
    if tui_state.show_collections {
        if tui_state.editing_collection_name {
            return handle_collection_name_editing(app, tui_state, code);
        }
        return handle_collections_navigation(app, tui_state, code);
    }

    match code {
        KeyCode::Char('q') => AppEvent::Quit,

        KeyCode::Char('h') => {
            tui_state.show_history = true;
            tui_state.show_collections = false;
            tui_state.history_index = 0;
            tui_state.focused_panel = FocusedPanel::Response;
            AppEvent::None
        }

        KeyCode::Char('c') => {
            tui_state.show_collections = true;
            tui_state.show_history = false;
            tui_state.collections_view = CollectionsView::CollectionList;
            tui_state.collection_index = 0;
            tui_state.focused_panel = FocusedPanel::Response;
            AppEvent::None
        }

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

fn handle_history_navigation(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Char('q') => AppEvent::Quit,

        KeyCode::Esc | KeyCode::Char('h') => {
            tui_state.show_history = false;
            AppEvent::None
        }

        KeyCode::Up => {
            tui_state.history_index = tui_state.history_index.saturating_sub(1);
            AppEvent::None
        }

        KeyCode::Down => {
            if !app.history.entries.is_empty() {
                let max = app.history.entries.len() - 1;
                if tui_state.history_index < max {
                    tui_state.history_index += 1;
                }
            }
            AppEvent::None
        }

        KeyCode::Enter => {
            if !app.history.entries.is_empty() {
                app.load_history_entry(tui_state.history_index);
                tui_state.show_history = false;
                tui_state.response_scroll = 0;
            }
            AppEvent::None
        }

        KeyCode::Char('d') => {
            if !app.history.entries.is_empty() {
                app.remove_history_entry(tui_state.history_index);
                if tui_state.history_index > 0
                    && tui_state.history_index >= app.history.entries.len()
                {
                    tui_state.history_index -= 1;
                }
            }
            AppEvent::None
        }

        KeyCode::Tab => {
            tui_state.focused_panel = match tui_state.focused_panel {
                FocusedPanel::Request => FocusedPanel::Response,
                FocusedPanel::Response => FocusedPanel::Request,
            };
            AppEvent::None
        }

        _ => AppEvent::None,
    }
}

fn handle_collections_navigation(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match tui_state.collections_view {
        CollectionsView::CollectionList => match code {
            KeyCode::Char('q') => AppEvent::Quit,

            KeyCode::Esc | KeyCode::Char('c') => {
                tui_state.show_collections = false;
                AppEvent::None
            }

            KeyCode::Up => {
                tui_state.collection_index = tui_state.collection_index.saturating_sub(1);
                AppEvent::None
            }

            KeyCode::Down => {
                if !app.collections.items.is_empty() {
                    let max = app.collections.items.len() - 1;
                    if tui_state.collection_index < max {
                        tui_state.collection_index += 1;
                    }
                }
                AppEvent::None
            }

            KeyCode::Enter => {
                if !app.collections.items.is_empty() {
                    tui_state.collections_view = CollectionsView::RequestList;
                    tui_state.collection_request_index = 0;
                }
                AppEvent::None
            }

            KeyCode::Char('n') => {
                tui_state.editing_collection_name = true;
                tui_state.collection_name_input.clear();
                tui_state.collection_name_cursor = 0;
                AppEvent::None
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
                AppEvent::None
            }

            KeyCode::Tab => {
                tui_state.focused_panel = match tui_state.focused_panel {
                    FocusedPanel::Request => FocusedPanel::Response,
                    FocusedPanel::Response => FocusedPanel::Request,
                };
                AppEvent::None
            }

            _ => AppEvent::None,
        },

        CollectionsView::RequestList => match code {
            KeyCode::Char('q') => AppEvent::Quit,

            KeyCode::Esc => {
                tui_state.collections_view = CollectionsView::CollectionList;
                AppEvent::None
            }

            KeyCode::Up => {
                tui_state.collection_request_index =
                    tui_state.collection_request_index.saturating_sub(1);
                AppEvent::None
            }

            KeyCode::Down => {
                if let Some(col) = app.collections.items.get(tui_state.collection_index) {
                    if !col.entries.is_empty() {
                        let max = col.entries.len() - 1;
                        if tui_state.collection_request_index < max {
                            tui_state.collection_request_index += 1;
                        }
                    }
                }
                AppEvent::None
            }

            KeyCode::Enter => {
                app.load_collection_entry(
                    tui_state.collection_index,
                    tui_state.collection_request_index,
                );
                tui_state.show_collections = false;
                tui_state.response_scroll = 0;
                AppEvent::None
            }

            KeyCode::Char('a') => {
                // Ajouter la requete courante a la collection
                if !app.current_request.url.is_empty() {
                    let name = format!(
                        "{} {}",
                        app.current_request.method, app.current_request.url
                    );
                    app.add_request_to_collection(tui_state.collection_index, name);
                }
                AppEvent::None
            }

            KeyCode::Char('d') => {
                let should_adjust = if let Some(col) = app.collections.items.get_mut(tui_state.collection_index) {
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
                AppEvent::None
            }

            KeyCode::Tab => {
                tui_state.focused_panel = match tui_state.focused_panel {
                    FocusedPanel::Request => FocusedPanel::Response,
                    FocusedPanel::Response => FocusedPanel::Request,
                };
                AppEvent::None
            }

            _ => AppEvent::None,
        },
    }
}

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
