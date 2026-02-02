//! Módulo TUI con Ratatui - Interfaz terminal interactiva
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use dreamcoder_core::{Config, DreamcoderEngine, Module, SystemInfo};

pub mod app;
pub mod events;
pub mod ui;

pub use app::App;
pub use events::{Event as AppEvent, EventHandler};

/// Ejecuta el modo TUI interactivo
pub async fn run_tui() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let config = Config::load().unwrap_or_default();
    let engine = DreamcoderEngine::new(config)?;
    let mut app = App::new(engine).await?;

    // Event handler
    let (tx, mut rx) = mpsc::channel(100);
    let event_handler = EventHandler::new(tx.clone());
    tokio::spawn(event_handler.run());

    // Main loop
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Handle events
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        
        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Tick => {
                    app.on_tick().await?;
                }
                AppEvent::Key(key) => {
                    if app.on_key(key).await? {
                        break;
                    }
                }
                AppEvent::Mouse(_) => {
                    // Handle mouse events
                }
                AppEvent::Resize(_, _) => {
                    // Handle resize
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await?;
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Pantalla de bienvenida splash
pub fn draw_splash_screen<B: ratatui::backend::Backend>(frame: &mut Frame<B>) {
    let size = frame.size();
    
    // Simple splash for now (tui-big-text can be added later)
    let splash = Paragraph::new("🎩\n\nDreamcoder\nv2.0\n\nPress any key...")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    
    let area = centered_rect(60, 40, size);
    frame.render_widget(splash, area);
}

/// Helper para centrar un rectángulo
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
