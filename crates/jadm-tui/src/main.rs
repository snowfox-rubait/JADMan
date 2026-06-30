mod app;
mod ui;
mod input;
mod client;
mod preview;

use std::io;
use ratatui::{backend::CrosstermBackend, Terminal};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::app::App;
use crate::client::rpc::RpcClient;
use jadm_common::protocol::{Request, Response};
use anyhow::Result;
use tokio::time::{interval, Duration};

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, crossterm::cursor::Show);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup panic hook to restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, crossterm::cursor::Show);
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    let _guard = TerminalGuard;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Resolve socket path dynamically in a cross-platform manner
    let proj_dirs = directories::ProjectDirs::from("com", "jadm", "jadm").unwrap();
    let socket_path = proj_dirs.runtime_dir()
        .map(|d| d.join("jadm.sock"))
        .unwrap_or_else(|| {
            #[cfg(unix)]
            { std::path::PathBuf::from("/tmp/jadm.sock") }
            #[cfg(not(unix))]
            { std::env::temp_dir().join("jadm.sock") }
        });

    // Setup app and RPC
    let mut app = App::new();
    let rpc = RpcClient::new(socket_path.to_string_lossy().to_string());

    let mut tick_rate = interval(Duration::from_millis(500));

    while app.running {
        terminal.draw(|f| ui::layout::render(f, &mut app))?;

        tokio::select! {
            _ = tick_rate.tick() => {
                // Refresh queue with connection resilience
                match rpc.send(Request::GetQueue).await {
                    Ok(Response::Queue { downloads }) => {
                        app.downloads = downloads;
                        app.connected = true;
                        app.check_bounds();
                    }
                    Ok(_) => {
                        // Unexpected response variant, still connected
                        app.connected = true;
                    }
                    Err(e) => {
                        app.connected = false;
                        eprintln!("TUI RPC connection error: {:?}", e);
                        // Don't crash — keep running so user sees "Disconnected"
                    }
                }
            }
            res = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(100))) => {
                if let Ok(Ok(true)) = res && let Event::Key(key) = event::read()? {
                    input::handler::handle_key_event(key, &mut app, &rpc).await;
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
