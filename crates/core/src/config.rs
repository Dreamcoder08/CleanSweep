use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuración central del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub dotfiles_root: PathBuf,
    pub backup_dir: PathBuf,
    pub state_dir: PathBuf,
    pub max_backups: usize,
    pub detect_changes: bool,
    pub auto_backup: bool,
    pub generate_reports: bool,
    pub secrets_provider: SecretsProvider,
    pub package_manager: PackageManagerConfig,
    pub ai: AiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsProvider {
    pub provider_type: SecretProviderType,
    pub key_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretProviderType {
    Age,
    Sops,
    OnePassword,
    Bitwarden,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfig {
    pub auto_install: bool,
    pub preferred_manager: Option<String>, // pacman, apt, brew, etc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub structured_output: bool,
    pub non_interactive: bool,
    pub auto_confirm: bool,
    pub json_pretty: bool,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();

        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            dotfiles_root: home
                .join("Documents")
                .join("PROYECTOS")
                .join("Dreamcoder_dots"),
            backup_dir: home.join(".config").join("dreamcoder-backups"),
            state_dir: home.join(".config").join("dreamcoder-manager"),
            max_backups: 5,
            detect_changes: true,
            auto_backup: true,
            generate_reports: true,
            secrets_provider: SecretsProvider {
                provider_type: SecretProviderType::None,
                key_file: None,
            },
            package_manager: PackageManagerConfig {
                auto_install: false,
                preferred_manager: None,
            },
            ai: AiConfig {
                structured_output: true,
                non_interactive: false,
                auto_confirm: false,
                json_pretty: true,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // TODO: Load from file (TOML/JSON)
        Ok(Self::default())
    }

    pub fn save(&self) -> Result<()> {
        // TODO: Save to file
        Ok(())
    }
}
