use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::models::Method;

use super::state::{FocusedPanel, RequestField, TuiState};

/// Couleur associee a chaque methode HTTP
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
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Titre
            Constraint::Min(10),  // Contenu
            Constraint::Length(1), // Barre d'aide
        ])
        .split(f.area());

    // Titre
    let title = Line::from(vec![
        Span::styled(" APIX ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("- API eXecutor ", Style::default().fg(Color::DarkGray)),
        if tui_state.is_editing {
            Span::styled("[EDIT] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        } else if app.is_loading {
            Span::styled("[LOADING] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        } else {
            Span::raw("")
        },
    ]);
    f.render_widget(Paragraph::new(title), main_chunks[0]);

    // Panneaux
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    draw_request_panel(f, content_chunks[0], app, tui_state);
    draw_response_panel(f, content_chunks[1], app, tui_state);

    // Barre d'aide
    draw_help_bar(f, main_chunks[2], tui_state);
}

fn draw_help_bar(f: &mut Frame, area: Rect, tui_state: &TuiState) {
    let help = if tui_state.is_editing && tui_state.focused_request_field == RequestField::Headers {
        Line::from(vec![
            Span::styled(" Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" cle/valeur  ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" valider  ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" annuler", Style::default().fg(Color::White)),
        ])
    } else if tui_state.is_editing {
        Line::from(vec![
            Span::styled(" Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" valider  ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" annuler", Style::default().fg(Color::White)),
        ])
    } else if tui_state.focused_panel == FocusedPanel::Request
        && tui_state.focused_request_field == RequestField::Headers
    {
        Line::from(vec![
            Span::styled(" a", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" ajouter  ", Style::default().fg(Color::White)),
            Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" supprimer  ", Style::default().fg(Color::White)),
            Span::styled("←→", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" naviguer  ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" editer  ", Style::default().fg(Color::White)),
            Span::styled("s", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" envoyer  ", Style::default().fg(Color::White)),
            Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" quitter", Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" panneaux  ", Style::default().fg(Color::White)),
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" naviguer  ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" editer  ", Style::default().fg(Color::White)),
            Span::styled("s", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" envoyer  ", Style::default().fg(Color::White)),
            Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" quitter", Style::default().fg(Color::White)),
        ])
    };
    f.render_widget(
        Paragraph::new(help).style(Style::default().bg(Color::DarkGray)),
        area,
    );
}

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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Method
            Constraint::Length(3), // URL
            Constraint::Length(5), // Headers
            Constraint::Min(3),   // Body
        ])
        .split(inner);

    draw_method_field(f, chunks[0], app, tui_state);
    draw_url_field(f, chunks[1], app, tui_state);
    draw_headers_field(f, chunks[2], app, tui_state);
    draw_body_field(f, chunks[3], app, tui_state);
}

fn is_field_active(tui_state: &TuiState, field: RequestField) -> bool {
    tui_state.focused_panel == FocusedPanel::Request
        && tui_state.focused_request_field == field
}

fn field_block<'a>(tui_state: &TuiState, field: RequestField, title: &'a str) -> Block<'a> {
    let active = is_field_active(tui_state, field);
    let border_style = if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style)
}

fn draw_method_field(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let active = is_field_active(tui_state, RequestField::Method);
    let method = app.current_request.method;
    let color = method_color(method);

    let text = Line::from(vec![
        Span::styled(
            format!(" {} ", method),
            Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD),
        ),
        if active {
            Span::styled("  ← Enter pour changer", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        },
    ]);

    let widget = Paragraph::new(text)
        .block(field_block(tui_state, RequestField::Method, " Method "));
    f.render_widget(widget, area);
}

fn draw_url_field(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let active = is_field_active(tui_state, RequestField::Url);

    let (text, style) = if tui_state.is_editing && active {
        (tui_state.url_input.as_str(), Style::default().fg(Color::White))
    } else if app.current_request.url.is_empty() {
        ("https://...", Style::default().fg(Color::DarkGray))
    } else {
        (app.current_request.url.as_str(), Style::default().fg(Color::White))
    };

    let widget = Paragraph::new(text)
        .block(field_block(tui_state, RequestField::Url, " URL "))
        .style(style);
    f.render_widget(widget, area);

    if tui_state.is_editing && active {
        let cursor_x = area.x + 1 + tui_state.url_cursor as u16;
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_headers_field(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let active = is_field_active(tui_state, RequestField::Headers);
    let editing = tui_state.is_editing && active;

    let items: Vec<ListItem> = if app.current_request.headers.is_empty() && !editing {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  (aucun header) ", Style::default().fg(Color::DarkGray)),
            if active {
                Span::styled("  a ajouter", Style::default().fg(Color::DarkGray))
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
                let is_selected = active && i == tui_state.header_index;

                if editing && is_selected {
                    // Afficher les inputs d'edition
                    let key_style = if tui_state.editing_header_key {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    let value_style = if !tui_state.editing_header_key {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let key_display = if tui_state.header_key_input.is_empty() {
                        "key"
                    } else {
                        &tui_state.header_key_input
                    };
                    let value_display = if tui_state.header_value_input.is_empty() {
                        "value"
                    } else {
                        &tui_state.header_value_input
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled("▸ ", Style::default().fg(Color::Yellow)),
                        Span::styled(key_display, key_style),
                        Span::styled(": ", Style::default().fg(Color::DarkGray)),
                        Span::styled(value_display, value_style),
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

    let widget = List::new(items)
        .block(field_block(tui_state, RequestField::Headers, &title));
    f.render_widget(widget, area);

    // Positionner le curseur en mode edition
    if editing {
        let cursor_x = if tui_state.editing_header_key {
            area.x + 1 + 2 + tui_state.header_key_cursor as u16
        } else {
            let key_len = if tui_state.header_key_input.is_empty() {
                3 // "key"
            } else {
                tui_state.header_key_input.len()
            };
            area.x + 1 + 2 + key_len as u16 + 2 + tui_state.header_value_cursor as u16
        };
        let cursor_y = area.y + 1 + tui_state.header_index as u16;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_body_field(f: &mut Frame, area: Rect, app: &App, tui_state: &TuiState) {
    let active = is_field_active(tui_state, RequestField::Body);

    let (text, style) = if tui_state.is_editing && active {
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
        .block(field_block(tui_state, RequestField::Body, " Body "))
        .style(style)
        .wrap(Wrap { trim: false });
    f.render_widget(widget, area);

    if tui_state.is_editing && active {
        let inner_width = area.width.saturating_sub(2).max(1);
        let cursor_x = area.x + 1 + (tui_state.body_cursor as u16 % inner_width);
        let cursor_y = area.y + 1 + (tui_state.body_cursor as u16 / inner_width);
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

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
            Span::styled("Envoi en cours...", Style::default().fg(Color::Yellow)),
        ]);
        f.render_widget(Paragraph::new(spinner), inner);
        return;
    }

    if let Some(ref error) = app.error_message {
        let error_lines = vec![
            Line::from(Span::styled("  Erreur", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled(format!("  {}", error), Style::default().fg(Color::Red))),
        ];
        f.render_widget(
            Paragraph::new(error_lines).wrap(Wrap { trim: false }),
            inner,
        );
        return;
    }

    if let Some(ref response) = app.current_response {
        draw_response_content(f, inner, response, app, tui_state);
        return;
    }

    let placeholder = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Aucune reponse.",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Appuyez sur ", Style::default().fg(Color::DarkGray)),
            Span::styled("s", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" pour envoyer.", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    f.render_widget(Paragraph::new(placeholder), inner);
}

fn draw_response_content(
    f: &mut Frame,
    area: Rect,
    response: &crate::models::Response,
    app: &App,
    tui_state: &TuiState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Status + method
            Constraint::Length(5), // Headers
            Constraint::Min(3),   // Body
        ])
        .split(area);

    // Status avec couleur methode
    let status_color = match response.status {
        200..=299 => Color::Green,
        300..=399 => Color::Yellow,
        _ => Color::Red,
    };
    let method = app.current_request.method;
    let status_line = Line::from(vec![
        Span::styled(
            format!(" {} ", method),
            Style::default().fg(Color::Black).bg(method_color(method)).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{}", response.status),
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}", status_text(response.status)),
            Style::default().fg(status_color),
        ),
    ]);
    let status_widget = Paragraph::new(status_line)
        .block(Block::default().borders(Borders::ALL).title(" Status ").border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(status_widget, chunks[0]);

    // Headers
    let header_items: Vec<ListItem> = response
        .headers
        .iter()
        .take(3)
        .map(|(k, v)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}", k), Style::default().fg(Color::Cyan)),
                Span::styled(": ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    v.to_str().unwrap_or("?"),
                    Style::default().fg(Color::White),
                ),
            ]))
        })
        .collect();
    let headers_widget = List::new(header_items)
        .block(Block::default().borders(Borders::ALL).title(" Headers ").border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(headers_widget, chunks[1]);

    // Body avec scroll securise et coloration JSON
    // Detecter si c'est du JSON structure ou du texte brut
    let is_json_object = matches!(response.body, serde_json::Value::Object(_) | serde_json::Value::Array(_));
    let body_text = serde_json::to_string_pretty(&response.body).unwrap_or_default();
    let all_lines: Vec<&str> = body_text.lines().collect();
    let scroll = tui_state.response_scroll.min(all_lines.len().saturating_sub(1));

    let visible_lines: Vec<Line> = if is_json_object {
        all_lines[scroll..]
            .iter()
            .map(|line| colorize_json_line(line))
            .collect()
    } else {
        // Texte brut (non-JSON) : afficher tel quel
        all_lines[scroll..]
            .iter()
            .map(|line| Line::from(Span::styled(*line, Style::default().fg(Color::White))))
            .collect()
    };

    let scroll_info = if all_lines.len() > chunks[2].height as usize {
        format!(" Body [{}/{}] ", scroll + 1, all_lines.len())
    } else {
        " Body ".to_string()
    };

    let title_suffix = if !is_json_object { " (raw) " } else { "" };
    let full_title = format!("{}{}", scroll_info, title_suffix);

    let body_widget = Paragraph::new(visible_lines)
        .block(Block::default().borders(Borders::ALL).title(full_title).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(body_widget, chunks[2]);
}

/// Colorise une ligne de JSON pretty-printed
fn colorize_json_line<'a>(line: &'a str) -> Line<'a> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let mut spans: Vec<Span<'a>> = vec![Span::raw(indent)];

    // Cle: "xxx":
    if let Some(colon_pos) = trimmed.find("\": ") {
        if trimmed.starts_with('"') {
            let key = &trimmed[..colon_pos + 1];
            spans.push(Span::styled(key, Style::default().fg(Color::Cyan)));
            spans.push(Span::styled(": ", Style::default().fg(Color::DarkGray)));
            let value = &trimmed[colon_pos + 3..];
            spans.push(colorize_json_value(value));
            return Line::from(spans);
        }
    }

    // Valeur seule (dans un array par exemple)
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
        // Ponctuation : {, }, [, ]
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
