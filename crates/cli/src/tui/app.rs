use crate::tui::events::{Event, EventHandler};
use dreamcoder_core::{Config, DreamcoderEngine, Module, Operation, Status};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Estado de la aplicación TUI
pub struct App {
    pub engine: DreamcoderEngine,
    pub state: AppState,
    pub modules: Vec<Module>,
    pub selected_module: usize,
    pub current_tab: Tab,
    pub progress: f64,
    pub operations: Vec<Operation>,
    pub logs: Vec<String>,
    pub should_quit: bool,
    pub show_splash: bool,
    pub splash_start: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Modules,
    Backups,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Splash,
    Main,
    Installing,
    Error,
}

impl App {
    pub async fn new(engine: DreamcoderEngine) -> anyhow::Result<Self> {
        let modules = engine.detect_modules()?;
        
        Ok(Self {
            engine,
            state: AppState::Splash,
            modules,
            selected_module: 0,
            current_tab: Tab::Dashboard,
            progress: 0.0,
            operations: Vec::new(),
            logs: vec!["Welcome to Dreamcoder v2.0".to_string()],
            should_quit: false,
            show_splash: true,
            splash_start: Instant::now(),
        })
    }

    /// Procesa eventos de teclado
    pub async fn on_key(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<bool> {
        use crossterm::event::KeyCode;

        // Si estamos en splash, cualquier tecla lo cierra
        if self.state == AppState::Splash {
            self.state = AppState::Main;
            return Ok(false);
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return Ok(true);
            }
            KeyCode::Char('m') => self.current_tab = Tab::Modules,
            KeyCode::Char('d') => self.current_tab = Tab::Dashboard,
            KeyCode::Char('b') => self.current_tab = Tab::Backups,
            KeyCode::Char('s') => self.current_tab = Tab::Settings,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_module < self.modules.len().saturating_sub(1) {
                    self.selected_module += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_module > 0 {
                    self.selected_module -= 1;
                }
            }
            KeyCode::Enter => {
                // Install selected module
                if let Some(module) = self.modules.get(self.selected_module) {
                    self.install_module(module.clone()).await?;
                }
            }
            _ => {}
        }

        Ok(false)
    }

    /// Tick del loop principal
    pub async fn on_tick(&mut self) -> anyhow::Result<()> {
        // Splash screen timeout
        if self.state == AppState::Splash && self.splash_start.elapsed() > Duration::from_secs(3) {
            self.state = AppState::Main;
        }

        // Update progress if installing
        if self.state == AppState::Installing {
            self.progress = (self.progress + 0.01).min(1.0);
            if self.progress >= 1.0 {
                self.state = AppState::Main;
                self.progress = 0.0;
            }
        }

        Ok(())
    }

    /// Instala un módulo
    async fn install_module(&mut self, module: Module) -> anyhow::Result<()> {
        self.state = AppState::Installing;
        self.progress = 0.0;
        self.logs.push(format!("Installing {}...", module.name));

        // TODO: Actual installation logic
        // For now, just simulate
        tokio::time::sleep(Duration::from_secs(1)).await;

        self.logs.push(format!("✓ {} installed", module.name));
        
        Ok(())
    }

    /// Obtiene el módulo seleccionado
    pub fn selected_module(&self) -> Option<&Module> {
        self.modules.get(self.selected_module)
    }

    /// Lista de tabs para mostrar
    pub fn tab_titles(&self) -> Vec<&'static str> {
        vec!["Dashboard", "Modules", "Backups", "Settings"]
    }
}
