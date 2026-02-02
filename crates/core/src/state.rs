use crate::{DreamcoderError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Gestiona el estado persistente del sistema
pub struct StateManager {
    state_dir: PathBuf,
    state: State,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    pub version: String,
    pub installed_modules: Vec<String>,
    pub last_backup: Option<chrono::DateTime<chrono::Utc>>,
    pub module_states: HashMap<String, ModuleState>,
    pub system_info: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleState {
    pub name: String,
    pub installed: bool,
    pub install_date: Option<chrono::DateTime<chrono::Utc>>,
    pub symlinks: Vec<PathBuf>,
    pub checksum: String, // Hash del módulo para detectar cambios
}

impl StateManager {
    pub fn new(config: &crate::Config) -> Result<Self> {
        let state_dir = config.state_dir.clone();
        fs::create_dir_all(&state_dir)?;

        let state_file = state_dir.join("state.json");
        let state = if state_file.exists() {
            let content = fs::read_to_string(&state_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            State::default()
        };

        Ok(Self { state_dir, state })
    }

    pub fn save(&self) -> Result<()> {
        let state_file = self.state_dir.join("state.json");
        let json = serde_json::to_string_pretty(&self.state)
            .map_err(|e| DreamcoderError::Serialization(e))?;
        fs::write(state_file, json)?;
        Ok(())
    }

    pub fn is_module_installed(&self, name: &str) -> bool {
        self.state.installed_modules.contains(&name.to_string())
    }

    pub fn register_module(&mut self, module_state: ModuleState) -> Result<()> {
        self.state.installed_modules.push(module_state.name.clone());
        self.state
            .module_states
            .insert(module_state.name.clone(), module_state);
        self.save()
    }

    pub fn get_module_state(&self, name: &str) -> Option<&ModuleState> {
        self.state.module_states.get(name)
    }
}
