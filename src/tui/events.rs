use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::App;
use crate::models::auth::{Auth, AUTH_TYPE_NAMES};
use crate::models::Method;

use super::state::{CollectionsView, EnvironmentsView, FocusedPanel, RequestField, TuiState};

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
                return Ok(handle_editing(app, tui_state, key));
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

    // Mode environnements : gestion separee
    if tui_state.show_environments {
        if tui_state.editing_environment_name {
            return handle_environment_name_editing(app, tui_state, code);
        }
        if tui_state.editing_variable {
            return handle_variable_editing(app, tui_state, code);
        }
        return handle_environments_navigation(app, tui_state, code);
    }

    match code {
        KeyCode::Char('q') => AppEvent::Quit,

        KeyCode::Char('h') => {
            tui_state.show_history = true;
            tui_state.show_collections = false;
            tui_state.show_environments = false;
            tui_state.history_index = 0;
            tui_state.focused_panel = FocusedPanel::Response;
            AppEvent::None
        }

        KeyCode::Char('c') => {
            tui_state.show_collections = true;
            tui_state.show_history = false;
            tui_state.show_environments = false;
            tui_state.collections_view = CollectionsView::CollectionList;
            tui_state.collection_index = 0;
            tui_state.focused_panel = FocusedPanel::Response;
            AppEvent::None
        }

        KeyCode::Char('e') => {
            tui_state.show_environments = true;
            tui_state.show_history = false;
            tui_state.show_collections = false;
            tui_state.environments_view = EnvironmentsView::EnvironmentList;
            tui_state.environment_index = 0;
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
                        let raw = app.current_request.body.clone().unwrap_or_default();
                        // Auto-formatter si le contenu est du JSON valide
                        tui_state.body_input =
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                                serde_json::to_string_pretty(&val).unwrap_or(raw)
                            } else {
                                raw
                            };
                        tui_state.body_cursor = tui_state.body_input.len();
                    }
                    RequestField::Auth => {
                        // Charger l'auth courante dans les inputs
                        match &app.current_request.auth {
                            None => {
                                tui_state.auth_type_index = 0;
                                tui_state.auth_token_input.clear();
                                tui_state.auth_token_cursor = 0;
                                tui_state.auth_username_input.clear();
                                tui_state.auth_password_input.clear();
                                tui_state.auth_username_cursor = 0;
                                tui_state.auth_password_cursor = 0;
                                tui_state.auth_key_name_input.clear();
                                tui_state.auth_key_value_input.clear();
                                tui_state.auth_key_name_cursor = 0;
                                tui_state.auth_key_value_cursor = 0;
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
                        tui_state.is_editing = true;
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

        // Copier le body dans le presse-papiers
        KeyCode::Char('y') => {
            app.copy_response_to_clipboard();
            AppEvent::None
        }

        // Sauvegarder le body dans response.json
        KeyCode::Char('w') => {
            app.save_response_to_file();
            AppEvent::None
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

fn handle_editing(app: &mut App, tui_state: &mut TuiState, key: KeyEvent) -> AppEvent {
    let code = key.code;

    if tui_state.focused_request_field == RequestField::Headers {
        return handle_header_editing(app, tui_state, code);
    }
    if tui_state.focused_request_field == RequestField::Auth {
        return handle_auth_editing(app, tui_state, code);
    }

    // ── Body : comportement textarea complet ──────────────────────────────
    if tui_state.focused_request_field == RequestField::Body {
        match code {
            // Esc = valider et sortir
            KeyCode::Esc => {
                let body = if tui_state.body_input.is_empty() {
                    None
                } else {
                    Some(tui_state.body_input.clone())
                };
                app.set_body(body);
                tui_state.is_editing = false;
            }

            // Enter = newline avec auto-indentation
            KeyCode::Enter => body_insert_newline(tui_state),

            // Tab = 2 espaces
            KeyCode::Tab => {
                tui_state.body_input.insert_str(tui_state.body_cursor, "  ");
                tui_state.body_cursor += 2;
            }

            // Ctrl+f = formatter le JSON
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                body_format_json(tui_state);
            }

            // Navigation verticale
            KeyCode::Up => body_move_up(tui_state),
            KeyCode::Down => body_move_down(tui_state),

            // Navigation horizontale
            KeyCode::Left => {
                tui_state.body_cursor = tui_state.body_cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                if tui_state.body_cursor < tui_state.body_input.len() {
                    tui_state.body_cursor += 1;
                }
            }

            // Suppression
            KeyCode::Backspace => {
                if tui_state.body_cursor > 0 {
                    tui_state.body_input.remove(tui_state.body_cursor - 1);
                    tui_state.body_cursor -= 1;
                }
            }

            // Saisie de caractère
            KeyCode::Char(c) => {
                tui_state.body_input.insert(tui_state.body_cursor, c);
                tui_state.body_cursor += 1;
            }

            _ => {}
        }
        return AppEvent::None;
    }

    // ── URL et autres champs ──────────────────────────────────────────────
    match code {
        KeyCode::Esc => {
            tui_state.is_editing = false;
        }

        KeyCode::Enter => {
            if tui_state.focused_request_field == RequestField::Url {
                app.set_url(tui_state.url_input.clone());
            }
            tui_state.is_editing = false;
        }

        KeyCode::Char(c) => {
            if tui_state.focused_request_field == RequestField::Url {
                tui_state.url_input.insert(tui_state.url_cursor, c);
                tui_state.url_cursor += 1;
            }
        }

        KeyCode::Backspace => {
            if tui_state.focused_request_field == RequestField::Url && tui_state.url_cursor > 0 {
                tui_state.url_input.remove(tui_state.url_cursor - 1);
                tui_state.url_cursor -= 1;
            }
        }

        KeyCode::Left => {
            if tui_state.focused_request_field == RequestField::Url {
                tui_state.url_cursor = tui_state.url_cursor.saturating_sub(1);
            }
        }

        KeyCode::Right => {
            if tui_state.focused_request_field == RequestField::Url
                && tui_state.url_cursor < tui_state.url_input.len()
            {
                tui_state.url_cursor += 1;
            }
        }

        _ => {}
    }
    AppEvent::None
}

// ── Helpers textarea body ─────────────────────────────────────────────────

/// Insère un '\n' avec auto-indentation en fonction du contexte JSON.
fn body_insert_newline(tui_state: &mut TuiState) {
    let cursor = tui_state.body_cursor;
    let line_start = tui_state.body_input[..cursor]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let line_before = &tui_state.body_input[line_start..cursor];

    // Indentation courante
    let indent: String = line_before.chars().take_while(|c| *c == ' ').collect();
    // Indentation supplémentaire si la ligne se termine par { ou [
    let trimmed = line_before.trim_end();
    let extra = if trimmed.ends_with('{') || trimmed.ends_with('[') {
        "  "
    } else {
        ""
    };
    let new_indent = format!("{}{}", indent, extra);

    // Si le caractère juste après le curseur est } ou ], insérer deux lignes
    let char_after = tui_state.body_input[cursor..].chars().next();
    let close_block =
        !extra.is_empty() && (char_after == Some('}') || char_after == Some(']'));

    if close_block {
        // \n<new_indent>   ← curseur ici
        // \n<indent>}
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

/// Formate le contenu du body si c'est du JSON valide.
fn body_format_json(tui_state: &mut TuiState) {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&tui_state.body_input) {
        if let Ok(pretty) = serde_json::to_string_pretty(&value) {
            tui_state.body_input = pretty;
            tui_state.body_cursor = tui_state.body_input.len();
        }
    }
}

/// Déplace le curseur d'une ligne vers le haut en conservant la colonne.
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

/// Déplace le curseur d'une ligne vers le bas en conservant la colonne.
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
    // Pas de ligne suivante : rester en place
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

fn handle_environments_navigation(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match tui_state.environments_view {
        EnvironmentsView::EnvironmentList => match code {
            KeyCode::Char('q') => AppEvent::Quit,

            KeyCode::Esc | KeyCode::Char('e') => {
                tui_state.show_environments = false;
                AppEvent::None
            }

            KeyCode::Up => {
                tui_state.environment_index = tui_state.environment_index.saturating_sub(1);
                AppEvent::None
            }

            KeyCode::Down => {
                if !app.environments.items.is_empty() {
                    let max = app.environments.items.len() - 1;
                    if tui_state.environment_index < max {
                        tui_state.environment_index += 1;
                    }
                }
                AppEvent::None
            }

            // Activer l'environnement
            KeyCode::Enter => {
                if !app.environments.items.is_empty() {
                    if app.active_environment == Some(tui_state.environment_index) {
                        // Desactiver si deja actif
                        app.active_environment = None;
                    } else {
                        app.active_environment = Some(tui_state.environment_index);
                    }
                }
                AppEvent::None
            }

            // Voir les variables
            KeyCode::Char('v') => {
                if !app.environments.items.is_empty() {
                    tui_state.environments_view = EnvironmentsView::VariableList;
                    tui_state.environment_variable_index = 0;
                }
                AppEvent::None
            }

            KeyCode::Char('n') => {
                tui_state.editing_environment_name = true;
                tui_state.environment_name_input.clear();
                tui_state.environment_name_cursor = 0;
                AppEvent::None
            }

            KeyCode::Char('d') => {
                if !app.environments.items.is_empty() {
                    // Si on supprime l'environnement actif, desactiver
                    if app.active_environment == Some(tui_state.environment_index) {
                        app.active_environment = None;
                    } else if let Some(active) = app.active_environment {
                        if tui_state.environment_index < active {
                            app.active_environment = Some(active - 1);
                        }
                    }
                    app.environments.remove_environment(tui_state.environment_index);
                    app.save_environments();
                    if tui_state.environment_index > 0
                        && tui_state.environment_index >= app.environments.items.len()
                    {
                        tui_state.environment_index -= 1;
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

        EnvironmentsView::VariableList => {
            let keys = match app.environments.items.get(tui_state.environment_index) {
                Some(env) => env.sorted_keys(),
                None => return AppEvent::None,
            };

            match code {
                KeyCode::Char('q') => AppEvent::Quit,

                KeyCode::Esc => {
                    tui_state.environments_view = EnvironmentsView::EnvironmentList;
                    AppEvent::None
                }

                KeyCode::Up => {
                    tui_state.environment_variable_index =
                        tui_state.environment_variable_index.saturating_sub(1);
                    AppEvent::None
                }

                KeyCode::Down => {
                    if !keys.is_empty() {
                        let max = keys.len() - 1;
                        if tui_state.environment_variable_index < max {
                            tui_state.environment_variable_index += 1;
                        }
                    }
                    AppEvent::None
                }

                // Editer la variable selectionnee
                KeyCode::Enter => {
                    if !keys.is_empty() {
                        let key = &keys[tui_state.environment_variable_index];
                        let env = &app.environments.items[tui_state.environment_index];
                        let value = env.variables.get(key).cloned().unwrap_or_default();
                        tui_state.variable_key_input = key.clone();
                        tui_state.variable_value_input = value;
                        tui_state.variable_key_cursor = tui_state.variable_key_input.len();
                        tui_state.variable_value_cursor = tui_state.variable_value_input.len();
                        tui_state.editing_variable_key = true;
                        tui_state.editing_variable = true;
                    }
                    AppEvent::None
                }

                // Ajouter une variable
                KeyCode::Char('a') => {
                    tui_state.variable_key_input.clear();
                    tui_state.variable_value_input.clear();
                    tui_state.variable_key_cursor = 0;
                    tui_state.variable_value_cursor = 0;
                    tui_state.editing_variable_key = true;
                    tui_state.editing_variable = true;
                    AppEvent::None
                }

                // Supprimer la variable
                KeyCode::Char('d') => {
                    if !keys.is_empty() {
                        let key = keys[tui_state.environment_variable_index].clone();
                        if let Some(env) = app.environments.items.get_mut(tui_state.environment_index) {
                            env.remove_variable(&key);
                        }
                        app.save_environments();
                        let new_len = app.environments.items[tui_state.environment_index].variables.len();
                        if tui_state.environment_variable_index > 0
                            && tui_state.environment_variable_index >= new_len
                        {
                            tui_state.environment_variable_index -= 1;
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
    }
}

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
                if let Some(env) = app.environments.items.get_mut(tui_state.environment_index) {
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
            } else {
                if tui_state.variable_value_cursor > 0 {
                    tui_state.variable_value_input.remove(tui_state.variable_value_cursor - 1);
                    tui_state.variable_value_cursor -= 1;
                }
            }
        }

        KeyCode::Left => {
            if tui_state.editing_variable_key {
                tui_state.variable_key_cursor = tui_state.variable_key_cursor.saturating_sub(1);
            } else {
                tui_state.variable_value_cursor = tui_state.variable_value_cursor.saturating_sub(1);
            }
        }

        KeyCode::Right => {
            if tui_state.editing_variable_key {
                if tui_state.variable_key_cursor < tui_state.variable_key_input.len() {
                    tui_state.variable_key_cursor += 1;
                }
            } else {
                if tui_state.variable_value_cursor < tui_state.variable_value_input.len() {
                    tui_state.variable_value_cursor += 1;
                }
            }
        }

        _ => {}
    }
    AppEvent::None
}

fn handle_auth_editing(app: &mut App, tui_state: &mut TuiState, code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Esc => {
            tui_state.is_editing = false;
        }

        // Cycle auth type
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
            tui_state.is_editing = false;
        }

        KeyCode::Tab => {
            match tui_state.auth_type_index {
                2 => tui_state.auth_editing_username = !tui_state.auth_editing_username,
                3 => tui_state.auth_editing_key_name = !tui_state.auth_editing_key_name,
                _ => {}
            }
        }

        KeyCode::Char(c) => {
            match tui_state.auth_type_index {
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
            }
        }

        KeyCode::Backspace => {
            match tui_state.auth_type_index {
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
                    } else {
                        if tui_state.auth_password_cursor > 0 {
                            tui_state.auth_password_input.remove(tui_state.auth_password_cursor - 1);
                            tui_state.auth_password_cursor -= 1;
                        }
                    }
                }
                3 => {
                    if tui_state.auth_editing_key_name {
                        if tui_state.auth_key_name_cursor > 0 {
                            tui_state.auth_key_name_input.remove(tui_state.auth_key_name_cursor - 1);
                            tui_state.auth_key_name_cursor -= 1;
                        }
                    } else {
                        if tui_state.auth_key_value_cursor > 0 {
                            tui_state.auth_key_value_input.remove(tui_state.auth_key_value_cursor - 1);
                            tui_state.auth_key_value_cursor -= 1;
                        }
                    }
                }
                _ => {}
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
