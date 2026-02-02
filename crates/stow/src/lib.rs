//! Sistema Stow Nativo - Implementación pura en Rust
//! 
//! Reimplementación de GNU Stow sin dependencias externas.
//! Maneja symlinks de forma inteligente con detección de conflictos,
//! backups automáticos, y resolución de árboles de directorios.

use std::path::PathBuf;
use thiserror::Error;
use tracing::{info, warn, error};
use walkdir::WalkDir;

pub mod types;
pub mod operations;
pub mod conflict;
pub mod tree;

pub use types::{StowConfig, SymlinkOperation, StowResult};
pub use operations::StowEngine;
pub use conflict::ConflictResolver;

#[derive(Error, Debug)]
pub enum StowError {
    #[error("Source directory does not exist: {0}")]
    SourceNotFound(PathBuf),
    
    #[error("Target directory does not exist: {0}")]
    TargetNotFound(PathBuf),
    
    #[error("Conflict detected: {message}")]
    Conflict {
        message: String,
        path: PathBuf,
        existing: Option<PathBuf>,
    },
    
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

pub type Result<T> = std::result::Result<T, StowError>;

/// StowManager - API principal para operaciones stow
pub struct StowManager {
    engine: StowEngine,
    config: StowConfig,
}

impl StowManager {
    pub fn new(config: StowConfig) -> Self {
        let engine = StowEngine::new(config.clone());
        Self { engine, config }
    }
    
    /// Stow (instala) un paquete
    pub async fn stow(&self, package: &str) -> Result<StowResult> {
        let package_path = self.config.source_dir.join(package);
        
        if !package_path.exists() {
            return Err(StowError::SourceNotFound(package_path));
        }
        
        info!("Stowing package: {}", package);
        
        let operations = self.engine.plan_stow(&package_path).await?;
        
        // Verificar conflictos
        let conflicts = ConflictResolver::detect_conflicts(&operations).await?;
        if !conflicts.is_empty() {
            warn!("Detected {} conflicts", conflicts.len());
            return Err(StowError::Conflict {
                message: format!("{} conflicts detected", conflicts.len()),
                path: conflicts[0].path.clone(),
                existing: conflicts[0].existing_target.clone(),
            });
        }
        
        // Ejecutar operaciones
        let results = self.engine.execute_operations(operations).await?;
        
        Ok(StowResult {
            package: package.to_string(),
            operations: results.len(),
            symlinks_created: results.iter().filter(|r| r.success).count(),
            errors: results.iter().filter(|r| !r.success).count(),
        })
    }
    
    /// Unstow (desinstala) un paquete
    pub async fn unstow(&self, package: &str) -> Result<StowResult> {
        let package_path = self.config.source_dir.join(package);
        
        info!("Unstowing package: {}", package);
        
        let operations = self.engine.plan_unstow(&package_path).await?;
        let results = self.engine.execute_operations(operations).await?;
        
        Ok(StowResult {
            package: package.to_string(),
            operations: results.len(),
            symlinks_created: 0,
            errors: results.iter().filter(|r| !r.success).count(),
        })
    }
    
    /// Restow (desinstala y reinstala)
    pub async fn restow(&self, package: &str) -> Result<StowResult> {
        self.unstow(package).await?;
        self.stow(package).await
    }
    
    /// Planifica operaciones sin ejecutarlas (dry-run)
    pub async fn plan(&self, package: &str) -> Result<Vec<SymlinkOperation>> {
        let package_path = self.config.source_dir.join(package);
        self.engine.plan_stow(&package_path).await
    }
    
    /// Lista paquetes disponibles
    pub fn list_packages(&self) -> Result<Vec<String>> {
        let mut packages = Vec::new();
        
        for entry in std::fs::read_dir(&self.config.source_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| StowError::InvalidPath("Invalid package name".to_string()))?;
                
                packages.push(name);
            }
        }
        
        packages.sort();
        Ok(packages)
    }
    
    /// Verifica integridad de symlinks existentes
    pub async fn verify(&self) -> Result<Vec<BrokenSymlink>> {
        let mut broken = Vec::new();
        
        for entry in WalkDir::new(&self.config.target_dir)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_symlink() {
                let target = std::fs::read_link(path)?;
                
                // Verificar si apunta a nuestro source_dir
                if target.starts_with(&self.config.source_dir) {
                    if !target.exists() {
                        broken.push(BrokenSymlink {
                            link: path.to_path_buf(),
                            target,
                            reason: "Target does not exist".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(broken)
    }
}

#[derive(Debug, Clone)]
pub struct BrokenSymlink {
    pub link: PathBuf,
    pub target: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct LinkInfo {
    pub source: PathBuf,
    pub target: PathBuf,
    pub is_dir: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_stow_simple_file() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");
        
        std::fs::create_dir(&source).unwrap();
        std::fs::create_dir(&target).unwrap();
        
        // Crear estructura de prueba
        let package = source.join("test-package");
        std::fs::create_dir(&package).unwrap();
        std::fs::write(package.join(".bashrc"), "# test").unwrap();
        
        let config = StowConfig {
            source_dir: source.clone(),
            target_dir: target.clone(),
            backup_existing: true,
            simulate: false,
        };
        
        let manager = StowManager::new(config);
        let result = manager.stow("test-package").await.unwrap();
        
        assert_eq!(result.symlinks_created, 1);
        assert!(target.join(".bashrc").is_symlink());
    }
}
