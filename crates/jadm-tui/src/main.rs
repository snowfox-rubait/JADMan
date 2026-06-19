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

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Use XDG runtime directory for socket path (fix from /tmp/jadm.sock)
    let uid = unsafe { libc::getuid() };
    let socket_path = format!("/run/user/{}/jadm/jadm.sock", uid);

    // Setup app and RPC
    let mut app = App::new();
    let rpc = RpcClient::new(socket_path);

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
                    Err(_) => {
                        app.connected = false;
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
