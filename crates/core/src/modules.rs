use std::path::{Path, PathBuf};
use dreamcoder_types::{Result, DreamcoderError, Module, Symlink};
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct ModuleMetadata {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    hooks: Option<ModuleHooks>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModuleHooks {
    pre_install: Option<String>,
    post_install: Option<String>,
}

pub struct ModuleManager {
    root: PathBuf,
}

impl ModuleManager {
    pub fn new(config: &dreamcoder_types::Config) -> Result<Self> {
        Ok(Self {
            root: config.dotfiles_root.clone(),
        })
    }
    
    /// Descubre todos los módulos disponibles
    pub fn discover_modules(&self) -> Result<Vec<Module>> {
        let mut modules = Vec::new();
        
        if !self.root.exists() {
            return Ok(modules);
        }

        // Buscar directorios que empiecen con "Dreamcoder"
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            
            if name_str.starts_with("Dreamcoder") && entry.file_type()?.is_dir() {
                if let Some(module) = self.load_module(&name_str)? {
                    modules.push(module);
                }
            }
        }
        
        // Ordenar por nombre
        modules.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(modules)
    }
    
    /// Carga un módulo específico desde disco
    fn load_module(&self, name: &str) -> Result<Option<Module>> {
        let path = self.root.join(name);
        
        if !path.exists() {
            return Ok(None);
        }
        
        // Buscar metadata (dreamcoder.toml o similar)
        let meta_path = path.join(".dreamcoder.toml");
        let metadata: Option<ModuleMetadata> = if meta_path.exists() {
            let content = std::fs::read_to_string(&meta_path)?;
            toml::from_str(&content).map_err(|e| DreamcoderError::Module(format!("Failed to parse metadata: {}", e)))?
        } else {
            None
        };
        
        let (description, hooks, deps) = if let Some(meta) = metadata {
            (meta.description, meta.hooks, meta.dependencies)
        } else {
            (None, None, Vec::new())
        };
        
        // Escanear symlinks potenciales
        let symlinks = self.scan_symlinks(&path, name)?;
        
        Ok(Some(Module {
            name: name.to_string(),
            path,
            description,
            symlinks,
            dependencies: deps,
            pre_install_hook: hooks.as_ref().and_then(|h| h.pre_install.clone()),
            post_install_hook: hooks.as_ref().and_then(|h| h.post_install.clone()),
            os_specific: None,
        }))
    }
    
    /// Escanea archivos para crear symlinks
    fn scan_symlinks(&self, module_path: &Path, _module_name: &str) -> Result<Vec<Symlink>> {
        let mut symlinks = Vec::new();
        let home = dirs::home_dir().ok_or_else(|| DreamcoderError::Config("No home dir".to_string()))?;
        
        for entry in WalkDir::new(module_path).min_depth(1) {
            let entry = entry?;
            // Ignorar archivos de metadata y hooks internos
            let file_name = entry.file_name().to_string_lossy();
            if file_name.starts_with(".dreamcoder") || file_name == ".hooks" {
                continue;
            }

            if entry.file_type().is_file() || entry.file_type().is_dir() {
                let source = entry.path().to_path_buf();
                let relative = source.strip_prefix(module_path)?;
                let target = home.join(relative);
                
                symlinks.push(Symlink {
                    source,
                    target,
                    backup_existing: true,
                });
            }
        }
        
        Ok(symlinks)
    }
    
    /// Verifica conflictos antes de instalar
    pub async fn check_conflicts(&self, module: &Module) -> Result<Option<Vec<PathBuf>>> {
        let mut conflicts = Vec::new();
        
        for symlink in &module.symlinks {
            if symlink.target.exists() && !symlink.target.is_symlink() {
                conflicts.push(symlink.target.clone());
            }
        }
        
        if conflicts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(conflicts))
        }
    }
    
    /// Instala el módulo (crea symlinks)
    pub async fn install(&self, module: &Module) -> Result<()> {
        use tokio::fs;
        
        for symlink in &module.symlinks {
            // Backup si existe y es archivo regular
            if symlink.target.exists() && !symlink.target.is_symlink() && symlink.backup_existing {
                let backup_name = format!(
                    "{}.backup.{}",
                    symlink.target.file_name().unwrap_or_default().to_string_lossy(),
                    chrono::Utc::now().timestamp()
                );
                let backup_path = symlink.target.with_file_name(backup_name);
                fs::rename(&symlink.target, &backup_path).await?;
            }
            
            // Crear directorio padre si no existe
            if let Some(parent) = symlink.target.parent() {
                fs::create_dir_all(parent).await?;
            }
            
            // Eliminar symlink existente si hay
            if symlink.target.is_symlink() {
                fs::remove_file(&symlink.target).await?;
            }
            
            // Crear symlink
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&symlink.source, &symlink.target)?;
            }
            
            #[cfg(windows)]
            {
                if symlink.source.is_dir() {
                    std::os::windows::fs::symlink_dir(&symlink.source, &symlink.target)?;
                } else {
                    std::os::windows::fs::symlink_file(&symlink.source, &symlink.target)?;
                }
            }
        }
        
        Ok(())
    }
}
