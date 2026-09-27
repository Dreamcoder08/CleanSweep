use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, error};

pub mod config;
pub mod state;
pub mod modules;
pub mod backup;

pub use state::{State, StateManager};
pub use modules::ModuleManager;
pub use dreamcoder_types::{
    Config, DreamcoderError, Result, Operation, OperationResult, Status, 
    ActionType, InputRequest, InputType, ApplyOptions, Module, Symlink,
    OsSpecific, ModuleVariant, ModuleState
};

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
        
        // 2. Pre-install hooks e Instalación
        for module in &options.modules {
            // Run pre-install hook if it exists and not skipped
            if !options.skip_hooks {
                if let Some(hook) = &module.pre_install_hook {
                    debug!("Running pre-install hook for {}", module.name);
                    let hook_op = self.run_hook(hook, module).await?;
                    operations.push(hook_op);
                    
                    if matches!(operations.last().unwrap().status, Status::Failed) {
                        error!("Pre-install hook failed for {}", module.name);
                    }
                }
            }

            debug!("Installing module: {}", module.name);
            let install_op = self.install_module(module).await?;
            operations.push(install_op);
        }
        
        // 3. Post-install hooks
        if !options.skip_hooks {
            for module in &options.modules {
                if let Some(hook) = &module.post_install_hook {
                    debug!("Running post-install hook for {}", module.name);
                    let hook_op = self.run_hook(hook, module).await?;
                    operations.push(hook_op);
                }
            }
        }
        
        let all_success = operations.iter().all(|op| matches!(op.status, Status::Success) || matches!(op.status, Status::Skipped));
        
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
        
        // Registrar en el estado
        let module_state = ModuleState {
            name: module.name.clone(),
            installed: true,
            install_date: Some(chrono::Utc::now()),
            symlinks: module.symlinks.iter().map(|s| s.target.clone()).collect(),
            checksum: String::new(), // TODO: Calcular checksum real
        };
        self._state.register_module(module_state)?;
        
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