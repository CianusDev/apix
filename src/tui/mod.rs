pub mod events;
pub mod state;
pub mod ui;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::panic;
use std::sync::mpsc;
use std::time::Duration;

use crate::app::App;
use crate::errors::{ApixError, Result};
use crate::models::{Request, Response};

use events::{handle_events, AppEvent};
use state::TuiState;

pub async fn run() -> Result<()> {
    // Panic hook pour restaurer le terminal
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App et state
    let mut app = App::new()?;
    app.initialize()?;
    let mut tui_state = TuiState::new();

    // Channels pour HTTP async
    let (request_tx, request_rx) = mpsc::channel::<Request>();
    let (response_tx, response_rx) =
        mpsc::channel::<std::result::Result<Response, ApixError>>();

    // Task en arriere-plan pour les requetes HTTP
    let http_client = app.http_client.clone();
    tokio::task::spawn(async move {
        while let Ok(request) = request_rx.recv() {
            let result = http_client.execute(&request).await;
            let _ = response_tx.send(result);
        }
    });

    // Boucle principale
    let result = run_loop(
        &mut terminal,
        &mut app,
        &mut tui_state,
        &request_tx,
        &response_rx,
    )
    .await;

    // Restaurer le terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    tui_state: &mut TuiState,
    request_tx: &mpsc::Sender<Request>,
    response_rx: &mpsc::Receiver<std::result::Result<Response, ApixError>>,
) -> Result<()> {
    loop {
        // Rendu
        terminal.draw(|f| ui::draw(f, app, tui_state))?;

        // Verifier les reponses HTTP (non-bloquant)
        if let Ok(response) = response_rx.try_recv() {
            app.finish_loading(response);
        }

        // Gestion des evenements clavier
        match handle_events(app, tui_state, Duration::from_millis(100))? {
            AppEvent::SendRequest => {
                if !app.current_request.url.is_empty() {
                    app.start_loading();

                    // Preparer la requete avec Content-Type auto
                    let mut request = app.current_request.clone();
                    if request.body.is_some() {
                        let has_ct = request
                            .headers
                            .iter()
                            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
                        if !has_ct {
                            request.headers.push((
                                "Content-Type".to_string(),
                                "application/json".to_string(),
                            ));
                        }
                    }

                    let _ = request_tx.send(request);
                }
            }
            AppEvent::Quit => break,
            AppEvent::None => {}
        }
    }

    Ok(())
}
