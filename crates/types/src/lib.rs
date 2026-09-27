use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DreamcoderError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Module error: {0}")]
    Module(String),
    
    #[error("Backup error: {0}")]
    Backup(String),
    
    #[error("Filesystem error: {0}")]
    Filesystem(String),
    
    #[error("Template error: {0}")]
    Template(String),
    
    #[error("Secret error: {0}")]
    Secret(String),
    
    #[error("Package manager error: {0}")]
    Package(String),
    
    #[error("AI Protocol error: {0}")]
    Ai(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Walkdir error: {0}")]
    Walkdir(String),
    
    #[error("Path error: {0}")]
    Path(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DreamcoderError>;

impl From<walkdir::Error> for DreamcoderError {
    fn from(err: walkdir::Error) -> Self {
        DreamcoderError::Walkdir(err.to_string())
    }
}

impl From<std::path::StripPrefixError> for DreamcoderError {
    fn from(_err: std::path::StripPrefixError) -> Self {
        DreamcoderError::Path("Failed to strip path prefix".to_string())
    }
}

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

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();

        Self {
            version: "2.0.0".to_string(),
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

/// Estructura para AI-First Protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub action: ActionType,
    pub status: Status,
    pub message: String,
    pub requires_input: Option<InputRequest>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub operations: Vec<Operation>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Backup,
    Install,
    Uninstall,
    Update,
    Template,
    Hook,
    PackageInstall,
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::Backup => write!(f, "backup"),
            ActionType::Install => write!(f, "install"),
            ActionType::Uninstall => write!(f, "uninstall"),
            ActionType::Update => write!(f, "update"),
            ActionType::Template => write!(f, "template"),
            ActionType::Hook => write!(f, "hook"),
            ActionType::PackageInstall => write!(f, "package-install"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
    PendingInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputRequest {
    pub input_type: InputType,
    pub message: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    Confirm,
    Text,
    Password,
    Select,
}

/// Representa un módulo de dotfiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub symlinks: Vec<Symlink>,
    pub dependencies: Vec<String>,
    pub pre_install_hook: Option<String>,
    pub post_install_hook: Option<String>,
    pub os_specific: Option<OsSpecific>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symlink {
    pub source: PathBuf,
    pub target: PathBuf,
    pub backup_existing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsSpecific {
    pub linux: Option<ModuleVariant>,
    pub macos: Option<ModuleVariant>,
    pub windows: Option<ModuleVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleVariant {
    pub path: PathBuf,
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleState {
    pub name: String,
    pub installed: bool,
    pub install_date: Option<chrono::DateTime<chrono::Utc>>,
    pub symlinks: Vec<PathBuf>,
    pub checksum: String, // Hash del módulo para detectar cambios
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyOptions {
    pub modules: Vec<Module>,
    pub backup: bool,
    pub dry_run: bool,
    pub skip_hooks: bool,
}
