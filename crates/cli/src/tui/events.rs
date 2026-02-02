use crossterm::event::{self, Event as CEvent, KeyEvent, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error};

/// Eventos de la aplicación
#[derive(Debug, Clone)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

/// Handler de eventos en background
pub struct EventHandler {
    tx: Sender<Event>,
}

impl EventHandler {
    pub fn new(tx: Sender<Event>) -> Self {
        Self { tx }
    }

    pub async fn run(self) {
        let mut interval = tokio::time::interval(Duration::from_millis(250));

        loop {
            interval.tick().await;

            // Try to read event with timeout
            if let Ok(true) = crossterm::event::poll(Duration::from_millis(100)) {
                match crossterm::event::read() {
                    Ok(CEvent::Key(key)) => {
                        if self.tx.send(Event::Key(key)).await.is_err() {
                            break;
                        }
                    }
                    Ok(CEvent::Mouse(mouse)) => {
                        if self.tx.send(Event::Mouse(mouse)).await.is_err() {
                            break;
                        }
                    }
                    Ok(CEvent::Resize(w, h)) => {
                        if self.tx.send(Event::Resize(w, h)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                    _ => {}
                }
            }

            // Send tick event
            if self.tx.send(Event::Tick).await.is_err() {
                break;
            }
        }
    }
}
