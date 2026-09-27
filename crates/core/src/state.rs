use crate::{DreamcoderError, Result};
use dreamcoder_types::ModuleState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Gestiona el estado persistente del sistema
#[derive(Clone)]
pub struct StateManager {
    inner: Arc<Mutex<StateInner>>,
}

struct StateInner {
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

impl StateManager {
    pub fn new(config: &crate::Config) -> Result<Self> {
        let state_dir = config.state_dir.clone();
        fs::create_dir_all(&state_dir)?;

        let state_file = state_dir.join("state.json");
        let state = if state_file.exists() {
            let content = fs::read_to_string(&state_file)?;
            serde_json::from_str(&content).unwrap_or_else(|_| State::default())
        } else {
            State {
                version: "2.0.0".to_string(), // env!("CARGO_PKG_VERSION") might not be available here if not careful
                ..State::default()
            }
        };

        Ok(Self { 
            inner: Arc::new(Mutex::new(StateInner { state_dir, state }))
        })
    }

    pub fn save(&self) -> Result<()> {
        let inner = self.inner.lock().map_err(|_| DreamcoderError::Filesystem("State lock poisoned".to_string()))?;
        let state_file = inner.state_dir.join("state.json");
        let json = serde_json::to_string_pretty(&inner.state)
            .map_err(|e| DreamcoderError::Serialization(e))?;
        fs::write(state_file, json)?;
        Ok(())
    }

    pub fn is_module_installed(&self, name: &str) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.state.installed_modules.contains(&name.to_string())
    }

    pub fn register_module(&self, module_state: ModuleState) -> Result<()> {
        {
            let mut inner = self.inner.lock().map_err(|_| DreamcoderError::Filesystem("State lock poisoned".to_string()))?;
            if !inner.state.installed_modules.contains(&module_state.name) {
                inner.state.installed_modules.push(module_state.name.clone());
            }
            inner.state
                .module_states
                .insert(module_state.name.clone(), module_state);
        }
        self.save()
    }

    pub fn get_module_state(&self, name: &str) -> Option<ModuleState> {
        let inner = self.inner.lock().unwrap();
        inner.state.module_states.get(name).cloned()
    }
}