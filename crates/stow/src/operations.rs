//! Motor de operaciones stow
//! 
//! Implementa la lógica de creación/eliminación de symlinks
//! con manejo de directorios y backups.

use std::path::Path;
use tokio::fs;
use tracing::{debug, info, error};
use walkdir::WalkDir;

use crate::{StowConfig, SymlinkOperation, Result, StowError};
use crate::types::{OperationType, OperationResult};

pub struct StowEngine {
    config: StowConfig,
}

impl StowEngine {
    pub fn new(config: StowConfig) -> Self {
        Self { config }
    }
    
    /// Planifica operaciones para stow (instalar)
    pub async fn plan_stow(&self, package_path: &Path) -> Result<Vec<SymlinkOperation>> {
        let mut operations = Vec::new();
        
        // Recorrer todos los archivos en el paquete
        for entry in WalkDir::new(package_path)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let source_path = entry.path();
            let relative_path = source_path.strip_prefix(package_path)
                .map_err(|_| StowError::InvalidPath("Cannot strip prefix".to_string()))?;
            
            let target_path = self.config.target_dir.join(relative_path);
            let is_dir = entry.file_type().is_dir();
            
            // Si es directorio, solo crear si no existe
            if is_dir {
                if !target_path.exists() {
                    operations.push(SymlinkOperation {
                        op_type: OperationType::CreateDir,
                        source: source_path.to_path_buf(),
                        target: target_path,
                        is_dir: true,
                        description: format!("Create directory {:?}", relative_path),
                    });
                }
                continue;
            }
            
            // Verificar si existe algo en el destino
            if target_path.exists() {
                if target_path.is_symlink() {
                    let existing_target = fs::read_link(&target_path).await?;
                    if existing_target == source_path {
                        // Ya está linkeado correctamente
                        debug!("Already linked: {:?}", target_path);
                        continue;
                    } else {
                        // Symlink a otro lugar, remover y recrear
                        operations.push(SymlinkOperation {
                            op_type: OperationType::RemoveSymlink,
                            source: source_path.to_path_buf(),
                            target: target_path.clone(),
                            is_dir: false,
                            description: format!("Remove existing symlink {:?}", target_path),
                        });
                    }
                } else {
                    // Archivo regular, hacer backup
                    if self.config.backup_existing {
                        operations.push(SymlinkOperation {
                            op_type: OperationType::Backup,
                            source: target_path.clone(),
                            target: target_path.clone(),
                            is_dir: false,
                            description: format!("Backup existing file {:?}", target_path),
                        });
                    }
                }
            }
            
            // Crear symlink
            operations.push(SymlinkOperation {
                op_type: OperationType::CreateSymlink,
                source: source_path.to_path_buf(),
                target: target_path,
                is_dir: false,
                description: format!("Create symlink for {:?}", relative_path),
            });
        }
        
        Ok(operations)
    }
    
    /// Planifica operaciones para unstow (desinstalar)
    pub async fn plan_unstow(&self, package_path: &Path) -> Result<Vec<SymlinkOperation>> {
        let mut operations = Vec::new();
        
        // Encontrar todos los symlinks que apuntan a este paquete
        for entry in WalkDir::new(&self.config.target_dir)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let target_path = entry.path();
            
            if target_path.is_symlink() {
                let link_target = fs::read_link(target_path).await?;
                
                // Si apunta a nuestro paquete
                if link_target.starts_with(package_path) {
                    operations.push(SymlinkOperation {
                        op_type: OperationType::RemoveSymlink,
                        source: link_target,
                        target: target_path.to_path_buf(),
                        is_dir: entry.file_type().is_dir(),
                        description: format!("Remove symlink {:?}", target_path),
                    });
                    
                    // Intentar eliminar directorios padre vacíos
                    if let Some(parent) = target_path.parent() {
                        if parent != self.config.target_dir {
                            operations.push(SymlinkOperation {
                                op_type: OperationType::RemoveDir,
                                source: parent.to_path_buf(),
                                target: parent.to_path_buf(),
                                is_dir: true,
                                description: format!("Remove empty dir {:?}", parent),
                            });
                        }
                    }
                }
            }
        }
        
        // Ordenar inversamente para eliminar de adentro hacia afuera
        operations.reverse();
        
        Ok(operations)
    }
    
    /// Ejecuta una lista de operaciones
    pub async fn execute_operations(&self, operations: Vec<SymlinkOperation>) -> Result<Vec<OperationResult>> {
        let mut results = Vec::new();
        
        for op in operations {
            let result = self.execute_operation(&op).await;
            let error_msg = result.as_ref().err().map(|e| e.to_string());
            
            results.push(OperationResult {
                success: result.is_ok(),
                operation: op.clone(),
                error: error_msg,
            });
            
            if let Err(ref e) = result {
                error!("Operation failed: {} - {}", op.description, e);
            }
        }
        
        Ok(results)
    }
    
    /// Ejecuta una operación individual
    async fn execute_operation(&self, op: &SymlinkOperation) -> Result<()> {
        if self.config.simulate {
            info!("[SIMULATE] {}", op.description);
            return Ok(());
        }
        
        match op.op_type {
            OperationType::CreateSymlink => {
                self.create_symlink(&op.source, &op.target, op.is_dir).await
            }
            OperationType::RemoveSymlink => {
                self.remove_symlink(&op.target).await
            }
            OperationType::CreateDir => {
                fs::create_dir_all(&op.target).await?;
                Ok(())
            }
            OperationType::RemoveDir => {
                // Solo eliminar si está vacío
                if let Ok(mut entries) = fs::read_dir(&op.target).await {
                    if entries.next_entry().await?.is_none() {
                        fs::remove_dir(&op.target).await?;
                    }
                }
                Ok(())
            }
            OperationType::Backup => {
                self.backup_file(&op.target).await
            }
        }
    }
    
    /// Crea un symlink (cross-platform)
    async fn create_symlink(&self, source: &Path, target: &Path, _is_dir: bool) -> Result<()> {
        // Asegurar que el directorio padre existe
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, target)?;
        }
        
        #[cfg(windows)]
        {
            if _is_dir {
                std::os::windows::fs::symlink_dir(source, target)?;
            } else {
                std::os::windows::fs::symlink_file(source, target)?;
            }
        }
        
        debug!("Created symlink: {:?} -> {:?}", target, source);
        Ok(())
    }
    
    /// Elimina un symlink
    async fn remove_symlink(&self, target: &Path) -> Result<()> {
        fs::remove_file(target).await?;
        debug!("Removed symlink: {:?}", target);
        Ok(())
    }
    
    /// Crea backup de archivo existente
    async fn backup_file(&self, target: &Path) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let file_name = target.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| StowError::InvalidPath("Invalid file name".to_string()))?;
        
        let backup_name = format!("{}.backup.{}", file_name, timestamp);
        let backup_path = target.with_file_name(backup_name);
        
        fs::rename(target, &backup_path).await?;
        info!("Backed up {:?} to {:?}", target, backup_path);
        
        Ok(())
    }
}
