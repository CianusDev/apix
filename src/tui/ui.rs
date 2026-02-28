use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::app::App;
use crate::models::auth::AUTH_TYPE_NAMES;
use crate::models::Method;

use super::state::{DrawerPanel, FocusedPanel, RequestTab, ResponseTab, TuiState};

/// Color associated with each HTTP method
fn method_color(method: Method) -> Color {
    match method {
        Method::GET => Color::Green,
        Method::POST => Color::Blue,
        Method::PUT => Color::Yellow,
        Method::DELETE => Color::Red,
        Method::PATCH => Color::Magenta,
    }
}

pub fn draw(f: &mut Frame, app: &App, tui_state: &TuiState) {
    let has_drawer = tui_state.drawer.is_some();
    let drawer_h = if has_drawer {
        (f.area().height * 35 / 100).max(5)
    } else {
        0
    };

    let [topbar_rect, main_rect, bottom_rect] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(if has_drawer { drawer_h } else { 1 }),
    ])
    .areas(f.area());

    draw_top_bar(f, topbar_rect, app, tui_state);

    let [request_rect, response_rect] = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .areas(main_rect);

    draw_request_panel(f, request_rect, app, tui_state);
    draw_response_panel(f, response_rect, app, tui_state);

    if let Some(panel) = tui_state.drawer {
        draw_drawer(f, bottom_rect, panel, app, tui_state);
    } else {
        draw_status_bar(f, bottom_rect, app, tui_state);
    }
}

// ── Top bar (URL bar) ─────────────────────────────────────────────────────────

fn draw_top_bar(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let border_style = if tui_state.editing_url {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let block = Block::default()
        .title(" APIX ")
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Layout: method badge (9) | url (rest)
    let [method_area, url_area] = Layout::horizontal([
        Constraint::Length(9),
        Constraint::Min(0),
    ])
    .areas(inner);

    // Method badge
    let method = app.current_request.method;
    let method_color = method_color(method);
    let method_widget = Paragraph::new(Line::from(vec![Span::styled(
        format!(" {:>5} ", method),
        Style::default()
            .fg(Color::Black)
            .bg(method_color)
            .add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(method_widget, method_area);

    // URL + env indicator
    let env_text = if let Some(idx) = app.active_environment {
        if let Some(env) = app.environments.items.get(idx) {
            format!(" [{}]", env.name)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let notice_text = if let Some(notice) = app.status_notice_text() {
        format!(" [{}]", notice)
    } else {
        String::new()
    };

    let url_text = if tui_state.editing_url {
        tui_state.url_input.as_str()
    } else if app.current_request.url.is_empty() {
        "https://..."
    } else {
        app.current_request.url.as_str()
    };

    let url_style = if tui_state.editing_url || !app.current_request.url.is_empty() {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Build the URL line with right-side indicators
    let url_width = url_area.width as usize;
    let suffix = if !notice_text.is_empty() {
        notice_text.clone()
    } else {
        env_text.clone()
    };
    let url_display_len = url_text.len().min(url_width.saturating_sub(suffix.len()));
    let url_display = if url_text.len() > url_display_len {
        format!("{}…", &url_text[..url_display_len.saturating_sub(1)])
    } else {
        url_text.to_string()
    };
    let padding = url_width
        .saturating_sub(url_display.len())
        .saturating_sub(suffix.len());

    let mut spans = vec![Span::styled(url_display, url_style)];
    if !suffix.is_empty() {
        spans.push(Span::raw(" ".repeat(padding)));
        let suffix_style = if !notice_text.is_empty() {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };
        spans.push(Span::styled(suffix, suffix_style));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), url_area);

    if tui_state.editing_url {
        let cx = url_area.x + tui_state.url_cursor as u16;
        let cy = url_area.y;
        f.set_cursor_position((cx, cy));
    }
}

// ── Request panel ─────────────────────────────────────────────────────────────

fn draw_request_panel(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let is_focused = tui_state.focused_panel == FocusedPanel::Request;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Request ")
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Layout: tab bar (1) | content (rest)
    let [tab_bar_rect, content_rect] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
    ])
    .areas(inner);

    draw_request_tab_bar(f, tab_bar_rect, tui_state);

    match tui_state.request_tab {
        RequestTab::Params => draw_params_tab(f, content_rect, app, tui_state),
        RequestTab::Headers => draw_headers_tab(f, content_rect, app, tui_state),
        RequestTab::Body => draw_body_tab(f, content_rect, app, tui_state),
        RequestTab::Auth => draw_auth_tab(f, content_rect, app, tui_state),
    }
}

fn draw_request_tab_bar(f: &mut Frame, area: Rect, tui_state: &TuiState) {
    let tab_names = ["1:Params", "2:Headers", "3:Body", "4:Auth"];
    let active_idx = match tui_state.request_tab {
        RequestTab::Params => 0,
        RequestTab::Headers => 1,
        RequestTab::Body => 2,
        RequestTab::Auth => 3,
    };
    let titles: Vec<Line> = tab_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            if i == active_idx {
                Line::from(Span::styled(
                    format!(" {} ", name),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(
                    format!(" {} ", name),
                    Style::default().fg(Color::DarkGray),
                ))
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .divider("│")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(tabs, area);
}

fn draw_params_tab(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let is_focused = tui_state.focused_panel == FocusedPanel::Request;
    let editing = tui_state.editing_param;

    let items: Vec<ListItem> = if app.current_request.params.is_empty() && !editing {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  (no params)", Style::default().fg(Color::DarkGray)),
            if is_focused {
                Span::styled("  a:add", Style::default().fg(Color::DarkGray))
            } else {
                Span::raw("")
            },
        ]))]
    } else {
        app.current_request
            .params
            .iter()
            .enumerate()
            .map(|(i, (k, v))| {
                let is_selected = is_focused && i == tui_state.param_index;
                if editing && is_selected {
                    let key_style = if tui_state.editing_param_key {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    let val_style = if !tui_state.editing_param_key {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    let key_d = if tui_state.param_key_input.is_empty() { "key" } else { &tui_state.param_key_input };
                    let val_d = if tui_state.param_value_input.is_empty() { "value" } else { &tui_state.param_value_input };
                    ListItem::new(Line::from(vec![
                        Span::styled("▸ ", Style::default().fg(Color::Yellow)),
                        Span::styled(key_d, key_style),
                        Span::styled("=", Style::default().fg(Color::DarkGray)),
                        Span::styled(val_d, val_style),
                    ]))
                } else {
                    let marker = if is_selected { "▸ " } else { "  " };
                    let key_style = if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(marker, Style::default().fg(Color::Yellow)),
                        Span::styled(k.as_str(), key_style),
                        Span::styled("=", Style::default().fg(Color::DarkGray)),
                        Span::styled(v.as_str(), Style::default().fg(Color::White)),
                    ]))
                }
            })
            .collect()
    };

    let param_count = app.current_request.params.len();
    let title = if param_count > 0 {
        format!(" Query Params [{}] ", param_count)
    } else {
        " Query Params ".to_string()
    };

    let widget = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(widget, area);

    if editing {
        let cursor_x = if tui_state.editing_param_key {
            area.x + 1 + 2 + tui_state.param_key_cursor as u16
        } else {
            let key_len = if tui_state.param_key_input.is_empty() { 3 } else { tui_state.param_key_input.len() };
            area.x + 1 + 2 + key_len as u16 + 1 + tui_state.param_value_cursor as u16
        };
        let cursor_y = area.y + 1 + tui_state.param_index as u16;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_headers_tab(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let is_focused = tui_state.focused_panel == FocusedPanel::Request;
    let editing = tui_state.editing_header;

    let items: Vec<ListItem> = if app.current_request.headers.is_empty() && !editing {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  (no headers)", Style::default().fg(Color::DarkGray)),
            if is_focused {
                Span::styled("  a:add", Style::default().fg(Color::DarkGray))
            } else {
                Span::raw("")
            },
        ]))]
    } else {
        app.current_request
            .headers
            .iter()
            .enumerate()
            .map(|(i, (k, v))| {
                let is_selected = is_focused && i == tui_state.header_index;
                if editing && is_selected {
                    let key_style = if tui_state.editing_header_key {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    let val_style = if !tui_state.editing_header_key {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    let key_d = if tui_state.header_key_input.is_empty() { "key" } else { &tui_state.header_key_input };
                    let val_d = if tui_state.header_value_input.is_empty() { "value" } else { &tui_state.header_value_input };
                    ListItem::new(Line::from(vec![
                        Span::styled("▸ ", Style::default().fg(Color::Yellow)),
                        Span::styled(key_d, key_style),
                        Span::styled(": ", Style::default().fg(Color::DarkGray)),
                        Span::styled(val_d, val_style),
                    ]))
                } else {
                    let marker = if is_selected { "▸ " } else { "  " };
                    let key_style = if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(marker, Style::default().fg(Color::Yellow)),
                        Span::styled(k.as_str(), key_style),
                        Span::styled(": ", Style::default().fg(Color::DarkGray)),
                        Span::styled(v.as_str(), Style::default().fg(Color::White)),
                    ]))
                }
            })
            .collect()
    };

    let header_count = app.current_request.headers.len();
    let title = if header_count > 0 {
        format!(" Headers [{}] ", header_count)
    } else {
        " Headers ".to_string()
    };

    let widget = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(widget, area);

    if editing {
        let cursor_x = if tui_state.editing_header_key {
            area.x + 1 + 2 + tui_state.header_key_cursor as u16
        } else {
            let key_len = if tui_state.header_key_input.is_empty() { 3 } else { tui_state.header_key_input.len() };
            area.x + 1 + 2 + key_len as u16 + 2 + tui_state.header_value_cursor as u16
        };
        let cursor_y = area.y + 1 + tui_state.header_index as u16;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_body_tab(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let (text, style) = if tui_state.is_editing_body {
        (tui_state.body_input.as_str(), Style::default().fg(Color::White))
    } else {
        let body = app.current_request.body.as_deref().unwrap_or("");
        if body.is_empty() {
            ("{}", Style::default().fg(Color::DarkGray))
        } else {
            (body, Style::default().fg(Color::White))
        }
    };

    let widget = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Body ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(style)
        .wrap(Wrap { trim: false });
    f.render_widget(widget, area);

    if tui_state.is_editing_body {
        let inner_width = area.width.saturating_sub(2).max(1) as usize;
        let mut visual_row = 0usize;
        let mut visual_col = 0usize;
        for (i, ch) in tui_state.body_input.char_indices() {
            if i == tui_state.body_cursor {
                break;
            }
            if ch == '\n' {
                visual_row += 1;
                visual_col = 0;
            } else {
                visual_col += 1;
                if visual_col >= inner_width {
                    visual_row += 1;
                    visual_col = 0;
                }
            }
        }
        f.set_cursor_position((
            area.x + 1 + visual_col as u16,
            area.y + 1 + visual_row as u16,
        ));
    }
}

fn draw_auth_tab(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Auth ")
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if !tui_state.editing_auth {
        let text = match &app.current_request.auth {
            None => Line::from(vec![
                Span::styled(" (none)  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Enter", Style::default().fg(Color::DarkGray)),
                Span::styled(" to configure", Style::default().fg(Color::DarkGray)),
            ]),
            Some(auth) => Line::from(vec![Span::styled(
                format!(" {}", auth.display_summary()),
                Style::default().fg(Color::Cyan),
            )]),
        };
        f.render_widget(Paragraph::new(text), inner);
        return;
    }

    let type_name = AUTH_TYPE_NAMES[tui_state.auth_type_index];
    let mut lines = vec![Line::from(vec![
        Span::styled("◀ ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{:>7}", type_name),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ▶", Style::default().fg(Color::Cyan)),
    ])];

    match tui_state.auth_type_index {
        0 => {
            if let Some(auth) = &app.current_request.auth {
                lines.push(Line::from(Span::styled(
                    format!(" Current:{}", auth.display_summary()),
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    " (no auth)",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
        1 => {
            let token_d = if tui_state.auth_token_input.is_empty() { "token..." } else { &tui_state.auth_token_input };
            lines.push(Line::from(vec![
                Span::styled(" Token: ", Style::default().fg(Color::DarkGray)),
                Span::styled(token_d, Style::default().fg(Color::White).add_modifier(Modifier::UNDERLINED)),
            ]));
        }
        2 => {
            let u_style = if tui_state.auth_editing_username {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::White)
            };
            let p_style = if !tui_state.auth_editing_username {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::White)
            };
            let u_d = if tui_state.auth_username_input.is_empty() { "user" } else { &tui_state.auth_username_input };
            let p_d = if tui_state.auth_password_input.is_empty() { "pass" } else { &tui_state.auth_password_input };
            lines.push(Line::from(vec![
                Span::styled(" User: ", Style::default().fg(Color::DarkGray)),
                Span::styled(u_d, u_style),
                Span::styled("  Pass: ", Style::default().fg(Color::DarkGray)),
                Span::styled(p_d, p_style),
            ]));
        }
        3 => {
            let k_style = if tui_state.auth_editing_key_name {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::White)
            };
            let v_style = if !tui_state.auth_editing_key_name {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::White)
            };
            let k_d = if tui_state.auth_key_name_input.is_empty() { "Header" } else { &tui_state.auth_key_name_input };
            let v_d = if tui_state.auth_key_value_input.is_empty() { "value" } else { &tui_state.auth_key_value_input };
            lines.push(Line::from(vec![
                Span::styled(k_d, k_style),
                Span::styled(": ", Style::default().fg(Color::DarkGray)),
                Span::styled(v_d, v_style),
            ]));
        }
        _ => {}
    }

    lines.push(Line::from(Span::styled(
        " Enter=save  Esc=cancel  ←→=type  Tab=field",
        Style::default().fg(Color::DarkGray),
    )));

    f.render_widget(Paragraph::new(lines), inner);

    let prefix_len = 13u16;
    match tui_state.auth_type_index {
        1 => {
            let cx = inner.x + prefix_len + tui_state.auth_token_cursor as u16;
            f.set_cursor_position((cx, inner.y + 1));
        }
        2 => {
            if tui_state.auth_editing_username {
                let cx = inner.x + 7 + tui_state.auth_username_cursor as u16;
                f.set_cursor_position((cx, inner.y + 1));
            } else {
                let ul = if tui_state.auth_username_input.is_empty() { 4 } else { tui_state.auth_username_input.len() };
                let cx = inner.x + 7 + ul as u16 + 8 + tui_state.auth_password_cursor as u16;
                f.set_cursor_position((cx, inner.y + 1));
            }
        }
        3 => {
            if tui_state.auth_editing_key_name {
                let cx = inner.x + tui_state.auth_key_name_cursor as u16;
                f.set_cursor_position((cx, inner.y + 1));
            } else {
                let kl = if tui_state.auth_key_name_input.is_empty() { 6 } else { tui_state.auth_key_name_input.len() };
                let cx = inner.x + kl as u16 + 2 + tui_state.auth_key_value_cursor as u16;
                f.set_cursor_position((cx, inner.y + 1));
            }
        }
        _ => {}
    }
}

// ── Response panel ────────────────────────────────────────────────────────────

fn draw_response_panel(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let is_focused = tui_state.focused_panel == FocusedPanel::Response;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Response ")
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.is_loading {
        let spinner = Line::from(vec![
            Span::styled("  ● ", Style::default().fg(Color::Yellow)),
            Span::styled("Sending...", Style::default().fg(Color::Yellow)),
        ]);
        f.render_widget(Paragraph::new(spinner), inner);
        return;
    }

    if let Some(ref error) = app.error_message {
        let error_lines = vec![
            Line::from(Span::styled(
                "  Error",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", error),
                Style::default().fg(Color::Red),
            )),
        ];
        f.render_widget(
            Paragraph::new(error_lines).wrap(Wrap { trim: false }),
            inner,
        );
        return;
    }

    let Some(ref response) = app.current_response else {
        let placeholder = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No response yet.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Press ", Style::default().fg(Color::DarkGray)),
                Span::styled("s", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" to send the request.", Style::default().fg(Color::DarkGray)),
            ]),
        ];
        f.render_widget(Paragraph::new(placeholder), inner);
        return;
    };

    // Compact status line (1) | tab bar (1) | content (rest)
    let [status_rect, tab_bar_rect, content_rect] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(3),
    ])
    .areas(inner);

    // Compact status line
    let status_color = match response.status {
        200..=299 => Color::Green,
        300..=399 => Color::Yellow,
        _ => Color::Red,
    };
    let method = app.current_request.method;
    let status_line = Line::from(vec![
        Span::styled("● ", Style::default().fg(status_color)),
        Span::styled(
            format!("{}", response.status),
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}  ", status_text(response.status)),
            Style::default().fg(status_color),
        ),
        Span::styled(
            format!(" {} ", method),
            Style::default().fg(Color::Black).bg(method_color(method)).add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(status_line), status_rect);

    draw_response_tab_bar(f, tab_bar_rect, tui_state);

    match tui_state.response_tab {
        ResponseTab::Body => draw_response_body_tab(f, content_rect, response, tui_state),
        ResponseTab::Headers => draw_response_headers_tab(f, content_rect, response, tui_state),
        ResponseTab::Cookies => draw_response_cookies_tab(f, content_rect, response),
    }
}

fn draw_response_tab_bar(f: &mut Frame, area: Rect, tui_state: &TuiState) {
    let tab_names = ["1:Body", "2:Headers", "3:Cookies"];
    let active_idx = match tui_state.response_tab {
        ResponseTab::Body => 0,
        ResponseTab::Headers => 1,
        ResponseTab::Cookies => 2,
    };
    let titles: Vec<Line> = tab_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            if i == active_idx {
                Line::from(Span::styled(
                    format!(" {} ", name),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(
                    format!(" {} ", name),
                    Style::default().fg(Color::DarkGray),
                ))
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .divider("│")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(tabs, area);
}

fn draw_response_body_tab(
    f: &mut Frame,
    area: Rect,
    response: &crate::models::Response,
    tui_state: &TuiState,
) {
    let is_json = matches!(
        response.body,
        serde_json::Value::Object(_) | serde_json::Value::Array(_)
    );
    let body_text = serde_json::to_string_pretty(&response.body).unwrap_or_default();
    let all_lines: Vec<&str> = body_text.lines().collect();
    let scroll = tui_state.response_scroll.min(all_lines.len().saturating_sub(1));

    let visible_lines: Vec<Line> = if is_json {
        all_lines[scroll..]
            .iter()
            .map(|line| colorize_json_line(line))
            .collect()
    } else {
        all_lines[scroll..]
            .iter()
            .map(|line| Line::from(Span::styled(*line, Style::default().fg(Color::White))))
            .collect()
    };

    let scroll_info = if all_lines.len() > area.height as usize {
        format!(" Body [{}/{}] ", scroll + 1, all_lines.len())
    } else {
        " Body ".to_string()
    };
    let suffix = if !is_json { "(raw)" } else { "" };
    let full_title = format!("{}{}", scroll_info, suffix);

    let body_widget = Paragraph::new(visible_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(full_title)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(body_widget, area);
}

fn draw_response_headers_tab(
    f: &mut Frame,
    area: Rect,
    response: &crate::models::Response,
    tui_state: &TuiState,
) {
    let all_headers: Vec<(&reqwest::header::HeaderName, &reqwest::header::HeaderValue)> =
        response.headers.iter().collect();
    let scroll = tui_state.response_headers_scroll.min(all_headers.len().saturating_sub(1));

    let items: Vec<ListItem> = all_headers[scroll..]
        .iter()
        .map(|(k, v)| {
            let is_cookie = k.as_str().eq_ignore_ascii_case("set-cookie");
            let key_color = if is_cookie { Color::Yellow } else { Color::Cyan };
            let val_color = if is_cookie { Color::Yellow } else { Color::White };
            ListItem::new(Line::from(vec![
                Span::styled(k.as_str(), Style::default().fg(key_color)),
                Span::styled(": ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    v.to_str().unwrap_or("?"),
                    Style::default().fg(val_color),
                ),
            ]))
        })
        .collect();

    let title = format!(" Headers [{}/{}] ", scroll + 1, all_headers.len().max(1));
    let widget = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(widget, area);
}

fn draw_response_cookies_tab(
    f: &mut Frame,
    area: Rect,
    response: &crate::models::Response,
) {
    let cookies: Vec<(String, String)> = response
        .headers
        .iter()
        .filter(|(k, _)| k.as_str().eq_ignore_ascii_case("set-cookie"))
        .map(|(_, v)| {
            let raw = v.to_str().unwrap_or("");
            let cookie_part = raw.split(';').next().unwrap_or(raw).trim();
            if let Some(eq_pos) = cookie_part.find('=') {
                (cookie_part[..eq_pos].to_string(), cookie_part[eq_pos + 1..].to_string())
            } else {
                (cookie_part.to_string(), String::new())
            }
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Cookies [{}] ", cookies.len()))
        .border_style(Style::default().fg(Color::DarkGray));

    if cookies.is_empty() {
        let inner = block.inner(area);
        f.render_widget(block, area);
        f.render_widget(
            Paragraph::new(Span::styled(
                "  No Set-Cookie headers in this response.",
                Style::default().fg(Color::DarkGray),
            )),
            inner,
        );
        return;
    }

    let items: Vec<ListItem> = cookies
        .iter()
        .map(|(name, value)| {
            ListItem::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(name.as_str(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("=", Style::default().fg(Color::DarkGray)),
                Span::styled(value.as_str(), Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let widget = List::new(items).block(block);
    f.render_widget(widget, area);
}

// ── Drawer ────────────────────────────────────────────────────────────────────

fn draw_drawer(f: &mut Frame, area: Rect, panel: DrawerPanel, app: &App, tui_state: &TuiState) {
    match panel {
        DrawerPanel::History => draw_drawer_history(f, area, app, tui_state),
        DrawerPanel::Collections => draw_drawer_collections(f, area, app, tui_state),
        DrawerPanel::Environments => draw_drawer_environments(f, area, app, tui_state),
    }
}

fn draw_drawer_history(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let filtered = tui_state.history_filtered_indices(&app.history.entries);
    let total = app.history.entries.len();
    let title = if tui_state.history_search_input.is_empty() {
        format!(" HISTORY [{}]  /=search ", total)
    } else {
        format!(" HISTORY [{}/{}]  /=search ", filtered.len(), total)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    // Search bar (1 line) + list (rest) + help (1 line)
    let help_h = 1u16;
    let search_h = if tui_state.history_search_active || !tui_state.history_search_input.is_empty() { 1u16 } else { 0u16 };
    let list_h = inner.height.saturating_sub(help_h + search_h);

    let [search_rect, list_rect, help_rect] = Layout::vertical([
        Constraint::Length(search_h),
        Constraint::Length(list_h),
        Constraint::Length(help_h),
    ])
    .areas(inner);

    // Search bar
    if search_h > 0 {
        let cursor_indicator = if tui_state.history_search_active { "█" } else { "" };
        let search_line = Line::from(vec![
            Span::styled("/ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}{}", tui_state.history_search_input, cursor_indicator),
                Style::default().fg(Color::White),
            ),
        ]);
        f.render_widget(Paragraph::new(search_line), search_rect);
    }

    // List
    if filtered.is_empty() {
        let msg = if app.history.entries.is_empty() { " (empty)" } else { " (no results)" };
        f.render_widget(
            Paragraph::new(Span::styled(msg, Style::default().fg(Color::DarkGray))),
            list_rect,
        );
    } else {
        let visible_h = list_rect.height as usize;
        let offset = scroll_offset(tui_state.history_index, visible_h);
        let items: Vec<ListItem> = filtered
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_h)
            .map(|(display_i, &original_i)| {
                let entry = &app.history.entries[original_i];
                let is_selected = display_i == tui_state.history_index;
                let method = Method::from_str(&entry.method).unwrap_or(Method::GET);
                let color = method_color(method);

                let max_url = list_rect.width.saturating_sub(12) as usize;
                let url_display = if entry.url.len() > max_url {
                    format!("{}…", &entry.url[..max_url.saturating_sub(1)])
                } else {
                    entry.url.clone()
                };

                let status_span = if let Some(s) = entry.status {
                    let sc = match s {
                        200..=299 => Color::Green,
                        300..=399 => Color::Yellow,
                        _ => Color::Red,
                    };
                    Span::styled(format!(" {:3}", s), Style::default().fg(sc))
                } else {
                    Span::raw("    ")
                };

                let style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        if is_selected { " ▶ " } else { "   " },
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::styled(
                        format!("{:>4}", entry.method.chars().take(4).collect::<String>()),
                        Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(url_display, Style::default().fg(Color::White)),
                    status_span,
                ]))
                .style(style)
            })
            .collect();
        f.render_widget(List::new(items), list_rect);
    }

    // Help
    let help = Line::from(vec![
        Span::styled(" h", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled(":close  ", Style::default().fg(Color::White)),
        Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(":nav  ", Style::default().fg(Color::White)),
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(":load  ", Style::default().fg(Color::White)),
        Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled(":del  ", Style::default().fg(Color::White)),
        Span::styled("/", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled(":search  ", Style::default().fg(Color::White)),
        Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(":close", Style::default().fg(Color::White)),
    ]);
    f.render_widget(Paragraph::new(help), help_rect);
}

fn draw_drawer_collections(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let count = app.collections.items.len();
    let title = format!(" COLLECTIONS [{}] ", count);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let help_h = 1u16;
    let list_h = inner.height.saturating_sub(help_h);
    let [list_rect, help_rect] = Layout::vertical([
        Constraint::Length(list_h),
        Constraint::Length(help_h),
    ])
    .areas(inner);

    // Editing collection name
    if tui_state.editing_collection_name {
        let line = Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::White)),
            Span::styled(
                if tui_state.collection_name_input.is_empty() {
                    "..."
                } else {
                    &tui_state.collection_name_input
                },
                Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED),
            ),
        ]);
        f.render_widget(Paragraph::new(line), list_rect);
        let cx = list_rect.x + 5 + tui_state.collection_name_cursor as u16;
        f.set_cursor_position((cx, list_rect.y));
    } else if tui_state.in_collection_requests {
        let col_idx = tui_state.collection_index;
        let col = match app.collections.items.get(col_idx) {
            Some(c) => c,
            None => {
                f.render_widget(
                    Paragraph::new(Span::styled(" (error)", Style::default().fg(Color::Red))),
                    list_rect,
                );
                return;
            }
        };

        if col.entries.is_empty() {
            f.render_widget(
                Paragraph::new(Span::styled(" (empty)", Style::default().fg(Color::DarkGray))),
                list_rect,
            );
        } else {
            let visible_h = list_rect.height as usize;
            let offset = scroll_offset(tui_state.collection_request_index, visible_h);
            let items: Vec<ListItem> = col
                .entries
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible_h)
                .map(|(i, entry)| {
                    let is_sel = i == tui_state.collection_request_index;
                    let method = Method::from_str(&entry.method).unwrap_or(Method::GET);
                    let color = method_color(method);
                    let max_url = list_rect.width.saturating_sub(10) as usize;
                    let url_display = if entry.url.len() > max_url {
                        format!("{}…", &entry.url[..max_url.saturating_sub(1)])
                    } else {
                        entry.url.clone()
                    };
                    let style = if is_sel { Style::default().bg(Color::DarkGray) } else { Style::default() };
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            if is_sel { " ▶ " } else { "   " },
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(
                            format!("{:>4}", entry.method.chars().take(4).collect::<String>()),
                            Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(url_display, Style::default().fg(Color::White)),
                    ]))
                    .style(style)
                })
                .collect();
            f.render_widget(List::new(items), list_rect);
        }
    } else if app.collections.items.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(" (empty)  n:new", Style::default().fg(Color::DarkGray))),
            list_rect,
        );
    } else {
            let visible_h = list_rect.height as usize;
            let offset = scroll_offset(tui_state.collection_index, visible_h);
            let items: Vec<ListItem> = app
                .collections
                .items
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible_h)
                .map(|(i, col)| {
                    let is_sel = i == tui_state.collection_index;
                    let style = if is_sel { Style::default().bg(Color::DarkGray) } else { Style::default() };
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            if is_sel { " ▶ " } else { "   " },
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(&col.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled(
                            format!(" ({})", col.entries.len()),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]))
                    .style(style)
                })
                .collect();
            f.render_widget(List::new(items), list_rect);
    }

    // Help
    let help = if tui_state.in_collection_requests {
        Line::from(vec![
            Span::styled(" c", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":close  ", Style::default().fg(Color::White)),
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":nav  ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":load  ", Style::default().fg(Color::White)),
            Span::styled("a", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":save req  ", Style::default().fg(Color::White)),
            Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(":del  ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":back", Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" c", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":close  ", Style::default().fg(Color::White)),
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":nav  ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":open  ", Style::default().fg(Color::White)),
            Span::styled("n", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":new  ", Style::default().fg(Color::White)),
            Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(":del  ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":close", Style::default().fg(Color::White)),
        ])
    };
    f.render_widget(Paragraph::new(help), help_rect);
}

fn draw_drawer_environments(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let count = app.environments.items.len();
    let title = format!(" ENVIRONMENTS [{}]", count);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let help_h = 1u16;
    let list_h = inner.height.saturating_sub(help_h);
    let [list_rect, help_rect] = Layout::vertical([
        Constraint::Length(list_h),
        Constraint::Length(help_h),
    ])
    .areas(inner);

    // Editing environment name
    if tui_state.editing_environment_name {
        let line = Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::White)),
            Span::styled(
                if tui_state.environment_name_input.is_empty() {
                    "..."
                } else {
                    &tui_state.environment_name_input
                },
                Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED),
            ),
        ]);
        f.render_widget(Paragraph::new(line), list_rect);
        let cx = list_rect.x + 5 + tui_state.environment_name_cursor as u16;
        f.set_cursor_position((cx, list_rect.y));
    } else if tui_state.in_environment_vars {
        let env_idx = tui_state.environment_index;
        let env = match app.environments.items.get(env_idx) {
            Some(e) => e,
            None => return,
        };

        // Editing variable
        if tui_state.editing_variable {
            let key_style = if tui_state.editing_variable_key {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::Cyan)
            };
            let val_style = if !tui_state.editing_variable_key {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::White)
            };
            let key_d = if tui_state.variable_key_input.is_empty() { "key" } else { &tui_state.variable_key_input };
            let val_d = if tui_state.variable_value_input.is_empty() { "val" } else { &tui_state.variable_value_input };
            let line = Line::from(vec![
                Span::styled(key_d, key_style),
                Span::styled("=", Style::default().fg(Color::DarkGray)),
                Span::styled(val_d, val_style),
            ]);
            f.render_widget(Paragraph::new(line), list_rect);
            let cx = if tui_state.editing_variable_key {
                list_rect.x + tui_state.variable_key_cursor as u16
            } else {
                let kl = if tui_state.variable_key_input.is_empty() { 3 } else { tui_state.variable_key_input.len() };
                list_rect.x + kl as u16 + 1 + tui_state.variable_value_cursor as u16
            };
            f.set_cursor_position((cx, list_rect.y));
            return;
        }

        let keys = env.sorted_keys();
        if keys.is_empty() {
            f.render_widget(
                Paragraph::new(Span::styled(" (empty)  a:add", Style::default().fg(Color::DarkGray))),
                list_rect,
            );
        } else {
            let items: Vec<ListItem> = keys
                .iter()
                .enumerate()
                .take(list_rect.height as usize)
                .map(|(i, k)| {
                    let is_sel = i == tui_state.environment_variable_index;
                    let v = env.variables.get(k).unwrap();
                    let max_v = list_rect.width.saturating_sub(k.len() as u16 + 7) as usize;
                    let v_display = if v.len() > max_v { &v[..max_v.saturating_sub(1)] } else { v.as_str() };
                    let style = if is_sel { Style::default().bg(Color::DarkGray) } else { Style::default() };
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            if is_sel { " ▶ " } else { "   " },
                            Style::default().fg(Color::Green),
                        ),
                        Span::styled(k.as_str(), Style::default().fg(Color::Cyan)),
                        Span::styled("=", Style::default().fg(Color::DarkGray)),
                        Span::styled(v_display, Style::default().fg(Color::White)),
                    ]))
                    .style(style)
                })
                .collect();
            f.render_widget(List::new(items), list_rect);
        }
    } else if app.environments.items.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(" (empty)  n:new", Style::default().fg(Color::DarkGray))),
            list_rect,
        );
    } else {
            let visible_h = list_rect.height as usize;
            let offset = scroll_offset(tui_state.environment_index, visible_h);
            let items: Vec<ListItem> = app
                .environments
                .items
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible_h)
                .map(|(i, env)| {
                    let is_sel = i == tui_state.environment_index;
                    let is_active_env = app.active_environment == Some(i);
                    let style = if is_sel { Style::default().bg(Color::DarkGray) } else { Style::default() };
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            if is_sel { " ▶ " } else { "   " },
                            Style::default().fg(Color::Green),
                        ),
                        Span::styled(&env.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        if is_active_env {
                            Span::styled(" ★ active", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                        } else {
                            Span::raw("")
                        },
                    ]))
                    .style(style)
                })
                .collect();
            f.render_widget(List::new(items), list_rect);
    }

    // Help
    let help = if tui_state.in_environment_vars {
        Line::from(vec![
            Span::styled(" e", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":close  ", Style::default().fg(Color::White)),
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":nav  ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":edit  ", Style::default().fg(Color::White)),
            Span::styled("a", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":add  ", Style::default().fg(Color::White)),
            Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(":del  ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":back", Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" e", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":close  ", Style::default().fg(Color::White)),
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":nav  ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":activate  ", Style::default().fg(Color::White)),
            Span::styled("v", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":vars  ", Style::default().fg(Color::White)),
            Span::styled("n", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":new  ", Style::default().fg(Color::White)),
            Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(":del  ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":close", Style::default().fg(Color::White)),
        ])
    };
    f.render_widget(Paragraph::new(help), help_rect);
}

// ── Status bar ────────────────────────────────────────────────────────────────

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let help = if tui_state.is_editing() {
        // Context-sensitive editing hints
        if tui_state.editing_url {
            Line::from(vec![
                Span::styled(" Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":confirm  ", Style::default().fg(Color::White)),
                Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":cancel  URL editing", Style::default().fg(Color::White)),
            ])
        } else if tui_state.is_editing_body {
            Line::from(vec![
                Span::styled(" Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":newline  ", Style::default().fg(Color::White)),
                Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":indent  ", Style::default().fg(Color::White)),
                Span::styled("C-f", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":fmt JSON  ", Style::default().fg(Color::White)),
                Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":save & exit", Style::default().fg(Color::White)),
            ])
        } else if tui_state.history_search_active {
            Line::from(vec![
                Span::styled(" Type to filter  ", Style::default().fg(Color::White)),
                Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":nav  ", Style::default().fg(Color::White)),
                Span::styled("Enter/Esc", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled(":done", Style::default().fg(Color::White)),
            ])
        } else {
            Line::from(vec![
                Span::styled(" Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":field  ", Style::default().fg(Color::White)),
                Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":confirm  ", Style::default().fg(Color::White)),
                Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":cancel", Style::default().fg(Color::White)),
            ])
        }
    } else {
        let panel_hint = match tui_state.focused_panel {
            FocusedPanel::Request => match tui_state.request_tab {
                RequestTab::Params | RequestTab::Headers => "↑↓:nav  Enter:edit  a:add  d:del",
                RequestTab::Body => "Enter:edit body",
                RequestTab::Auth => "Enter:configure",
            },
            FocusedPanel::Response => "↑↓:scroll  y:copy  w:save",
        };

        let env_text = if let Some(idx) = app.active_environment {
            if let Some(env) = app.environments.items.get(idx) {
                format!("  [ENV:{}]", env.name)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let loading_text = if app.is_loading { "  ⏳" } else { "" };

        Line::from(vec![
            Span::styled(" h", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(":History  ", Style::default().fg(Color::White)),
            Span::styled("c", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":Collections  ", Style::default().fg(Color::White)),
            Span::styled("e", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":Envs  ", Style::default().fg(Color::White)),
            Span::styled("u", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(":URL  ", Style::default().fg(Color::White)),
            Span::styled("m", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(":method  ", Style::default().fg(Color::White)),
            Span::styled("s", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(":send  ", Style::default().fg(Color::White)),
            Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(":switch  ", Style::default().fg(Color::White)),
            Span::styled(panel_hint, Style::default().fg(Color::White)),
            Span::styled(env_text, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(loading_text, Style::default().fg(Color::Yellow)),
            Span::styled("  q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(":quit", Style::default().fg(Color::White)),
        ])
    };

    f.render_widget(
        Paragraph::new(help).style(Style::default().bg(Color::DarkGray)),
        area,
    );
}

// ── Scroll helper ─────────────────────────────────────────────────────────────

/// Compute scroll offset so that `selected` is always visible.
fn scroll_offset(selected: usize, visible: usize) -> usize {
    if selected >= visible {
        selected + 1 - visible
    } else {
        0
    }
}

// ── JSON colorization ─────────────────────────────────────────────────────────

fn colorize_json_line<'a>(line: &'a str) -> Line<'a> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let mut spans: Vec<Span<'a>> = vec![Span::raw(indent)];

    if let Some(colon_pos) = trimmed.find("\": ")
        && trimmed.starts_with('"')
    {
        let key = &trimmed[..colon_pos + 1];
        spans.push(Span::styled(key, Style::default().fg(Color::Cyan)));
        spans.push(Span::styled(": ", Style::default().fg(Color::DarkGray)));
        let value = &trimmed[colon_pos + 3..];
        spans.push(colorize_json_value(value));
        return Line::from(spans);
    }

    spans.push(colorize_json_value(trimmed));
    Line::from(spans)
}

fn colorize_json_value<'a>(value: &'a str) -> Span<'a> {
    let v = value.trim_end_matches(',');
    if v.starts_with('"') {
        Span::styled(value, Style::default().fg(Color::Green))
    } else if v == "true" || v == "false" {
        Span::styled(value, Style::default().fg(Color::Magenta))
    } else if v == "null" {
        Span::styled(value, Style::default().fg(Color::Red))
    } else if v.parse::<f64>().is_ok() {
        Span::styled(value, Style::default().fg(Color::Yellow))
    } else {
        Span::styled(value, Style::default().fg(Color::DarkGray))
    }
}

fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "",
    }
}
