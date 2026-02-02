use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, error};

pub mod config;
pub mod state;
pub mod modules;
pub mod backup;

pub use config::Config;
pub use state::{State, StateManager};
pub use modules::{Module, ModuleManager};

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

pub type Result<T> = std::result::Result<T, DreamcoderError>;

/// Core engine que orquesta todas las operaciones
pub struct DreamcoderEngine {
    config: Config,
    _state: StateManager,
    module_manager: ModuleManager,
}

impl DreamcoderEngine {
    pub fn new(config: Config) -> Result<Self> {
        let state = StateManager::new(&config)?;
        let module_manager = ModuleManager::new(&config)?;
        
        info!("Dreamcoder Engine initialized v{}", env!("CARGO_PKG_VERSION"));
        
        Ok(Self {
            config,
            _state: state,
            module_manager,
        })
    }
    
    /// Detecta módulos disponibles en el repositorio
    pub fn detect_modules(&self) -> Result<Vec<Module>> {
        self.module_manager.discover_modules()
    }
    
    /// Ejecuta una operación completa (AI-First protocol)
    pub async fn apply(&self, options: ApplyOptions) -> Result<OperationResult> {
        let mut operations = Vec::new();
        
        // 1. Backup si está habilitado
        if options.backup {
            debug!("Creating backup before apply");
            let backup_op = self.create_backup().await?;
            operations.push(backup_op);
        }
        
        // 2. Instalar módulos
        for module in &options.modules {
            debug!("Installing module: {}", module.name);
            let install_op = self.install_module(module).await?;
            operations.push(install_op);
        }
        
        // 3. Post-install hooks
        for module in &options.modules {
            if let Some(hook) = &module.post_install_hook {
                debug!("Running post-install hook for {}", module.name);
                let hook_op = self.run_hook(hook, module).await?;
                operations.push(hook_op);
            }
        }
        
        let all_success = operations.iter().all(|op| matches!(op.status, Status::Success));
        
        Ok(OperationResult {
            success: all_success,
            operations,
            timestamp: chrono::Utc::now(),
        })
    }
    
    /// Crea backup inteligente (solo si hay cambios)
    pub async fn create_backup(&self) -> Result<Operation> {
        let backup = backup::BackupManager::new(&self.config);
        
        // Verificar si hay cambios
        if !backup.has_changes().await? {
            return Ok(Operation {
                id: uuid::Uuid::new_v4().to_string(),
                action: ActionType::Backup,
                status: Status::Skipped,
                message: "No changes detected".to_string(),
                requires_input: None,
                metadata: serde_json::json!({}),
            });
        }
        
        let backup_path = backup.create().await?;
        
        Ok(Operation {
            id: uuid::Uuid::new_v4().to_string(),
            action: ActionType::Backup,
            status: Status::Success,
            message: format!("Backup created at {:?}", backup_path),
            requires_input: None,
            metadata: serde_json::json!({
                "path": backup_path,
                "size": 0, // TODO: Calculate size
            }),
        })
    }
    
    /// Instala un módulo específico
    pub async fn install_module(&self, module: &Module) -> Result<Operation> {
        // Pre-install checks
        if let Some(conflicts) = self.module_manager.check_conflicts(module).await? {
            return Ok(Operation {
                id: uuid::Uuid::new_v4().to_string(),
                action: ActionType::Install,
                status: Status::PendingInput,
                message: format!("Conflicts detected for module {}", module.name),
                requires_input: Some(InputRequest {
                    input_type: InputType::Confirm,
                    message: format!("Overwrite existing files? {:?}", conflicts),
                    default: Some("false".to_string()),
                }),
                metadata: serde_json::json!({
                    "module": module.name,
                    "conflicts": conflicts,
                }),
            });
        }
        
        // Realizar instalación
        self.module_manager.install(module).await?;
        
        Ok(Operation {
            id: uuid::Uuid::new_v4().to_string(),
            action: ActionType::Install,
            status: Status::Success,
            message: format!("Module {} installed successfully", module.name),
            requires_input: None,
            metadata: serde_json::json!({
                "module": module.name,
                "symlinks_created": module.symlinks.len(),
            }),
        })
    }
    
    /// Ejecuta un hook (shell script o comando)
    async fn run_hook(&self, hook: &str, module: &Module) -> Result<Operation> {
        use tokio::process::Command;
        
        let output = Command::new("sh")
            .arg("-c")
            .arg(hook)
            .env("DREAMCODER_MODULE", &module.name)
            .env("DREAMCODER_ROOT", &self.config.dotfiles_root)
            .output()
            .await?;
        
        let success = output.status.success();
        
        Ok(Operation {
            id: uuid::Uuid::new_v4().to_string(),
            action: ActionType::Hook,
            status: if success { Status::Success } else { Status::Failed },
            message: String::from_utf8_lossy(&output.stdout).to_string(),
            requires_input: if success { None } else {
                Some(InputRequest {
                    input_type: InputType::Confirm,
                    message: "Hook failed, continue?".to_string(),
                    default: Some("true".to_string()),
                })
            },
            metadata: serde_json::json!({
                "module": module.name,
                "hook": hook,
                "exit_code": output.status.code(),
            }),
        })
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyOptions {
    pub modules: Vec<Module>,
    pub backup: bool,
    pub dry_run: bool,
    pub skip_hooks: bool,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub username: String,
    pub home_dir: PathBuf,
}

impl SystemInfo {
    pub fn detect() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        
        // Get username from environment variable
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_default();
        
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            hostname: sysinfo::System::host_name().unwrap_or_default(),
            username,
            home_dir: home,
        }
    }
}
