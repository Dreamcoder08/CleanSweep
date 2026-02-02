use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::tui;
use crate::tui::app::{App, AppState, Tab};

/// Dibuja la UI principal
pub fn draw<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    match app.state {
        AppState::Splash => {
            tui::draw_splash_screen(frame);
        }
        _ => {
            draw_main_ui(frame, app);
        }
    }
}

/// Dibuja la UI principal
fn draw_main_ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    let size = frame.size();

    // Layout principal
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Status bar
        ])
        .split(size);

    // Header
    draw_header(frame, app, chunks[0]);

    // Tabs
    draw_tabs(frame, app, chunks[1]);

    // Content según tab
    match app.current_tab {
        Tab::Dashboard => draw_dashboard(frame, app, chunks[2]),
        Tab::Modules => draw_modules(frame, app, chunks[2]),
        Tab::Backups => draw_backups(frame, app, chunks[2]),
        Tab::Settings => draw_settings(frame, app, chunks[2]),
    }

    // Status bar
    draw_status_bar(frame, app, chunks[3]);

    // Progress overlay if installing
    if app.state == AppState::Installing {
        draw_progress_overlay(frame, app);
    }
}

/// Dibuja el header
fn draw_header<B: Backend>(frame: &mut Frame<B>, _app: &App, area: Rect) {
    let header = Paragraph::new("🎩 Dreamcoder Dotfiles Manager v2.0")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, area);
}

/// Dibuja las tabs
fn draw_tabs<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    let titles = app
        .tab_titles()
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::White))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Navigation"))
        .select(app.current_tab as usize)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw("|"));

    frame.render_widget(tabs, area);
}

/// Dibuja el dashboard
fn draw_dashboard<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Panel izquierdo: System info
    let sys_info = format!(
        "OS: {}\nArch: {}\nModules: {} available\nStatus: Ready",
        "Linux", // TODO: Get from app
        "x86_64",
        app.modules.len()
    );

    let left = Paragraph::new(sys_info)
        .block(Block::default().title("System Info").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(left, chunks[0]);

    // Panel derecho: Logs
    let logs_text = app.logs.join("\n");
    let right = Paragraph::new(logs_text)
        .block(Block::default().title("Activity Log").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(right, chunks[1]);
}

/// Dibuja la lista de módulos
fn draw_modules<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .modules
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let style = if i == app.selected_module {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = format!(
                "{} {} ({} files)",
                if i == app.selected_module {
                    "▶"
                } else {
                    "  "
                },
                m.name,
                m.symlinks.len()
            );

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Modules (j/k to navigate, Enter to install)")
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("▶");

    frame.render_widget(list, area);
}

/// Dibuja la sección de backups
fn draw_backups<B: Backend>(frame: &mut Frame<B>, _app: &App, area: Rect) {
    let text = Paragraph::new("Backup management\n\nNo backups yet.")
        .block(Block::default().title("Backups").borders(Borders::ALL));

    frame.render_widget(text, area);
}

/// Dibuja la sección de settings
fn draw_settings<B: Backend>(frame: &mut Frame<B>, _app: &App, area: Rect) {
    let text = Paragraph::new("Settings\n\n• Auto-backup: Enabled\n• Non-interactive: Disabled")
        .block(Block::default().title("Settings").borders(Borders::ALL));

    frame.render_widget(text, area);
}

/// Dibuja la barra de estado
fn draw_status_bar<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    let status = match app.state {
        AppState::Installing => "Installing...",
        _ => "Ready",
    };

    let text = format!(
        "{} | Press 'q' to quit | [d]ashboard [m]odules [b]ackups [s]ettings",
        status
    );

    let status_bar = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));

    frame.render_widget(status_bar, area);
}

/// Dibuja overlay de progreso
fn draw_progress_overlay<B: Backend>(frame: &mut Frame<B>, app: &App) {
    let size = frame.size();
    let area = centered_rect(60, 20, size);

    // Clear background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title("Installing...")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    frame.render_widget(block, area);

    // Progress gauge
    let gauge_area = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width - 4,
        height: 3,
    };

    let gauge = Gauge::default()
        .percent((app.progress * 100.0) as u16)
        .label(format!("{:.0}%", app.progress * 100.0))
        .style(Style::default().fg(Color::Cyan))
        .gauge_style(Style::default().fg(Color::Cyan));

    frame.render_widget(gauge, gauge_area);
}

/// Helper para centrar rectángulo
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
