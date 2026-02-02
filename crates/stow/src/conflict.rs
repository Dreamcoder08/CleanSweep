//! Resolución de conflictos stow
//! 
//! Detecta y maneja conflictos entre symlinks y archivos existentes.

use std::path::PathBuf;
use tokio::fs;
use tracing::warn;

use crate::{SymlinkOperation, Result, StowError};
use crate::types::OperationType;

/// Representa un conflicto detectado
#[derive(Debug, Clone)]
pub struct Conflict {
    pub path: PathBuf,
    pub existing_target: Option<PathBuf>,
    pub new_target: PathBuf,
    pub conflict_type: ConflictType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictType {
    /// Archivo regular bloquea symlink
    RegularFile,
    /// Directorio no vacío bloquea symlink
    NonEmptyDir,
    /// Symlink a otro paquete
    WrongSymlinkTarget,
    /// Symlink circular
    CircularSymlink,
}

/// Resolutor de conflictos
pub struct ConflictResolver;

impl ConflictResolver {
    /// Detecta conflictos en un conjunto de operaciones
    pub async fn detect_conflicts(operations: &[SymlinkOperation]) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();
        
        for op in operations {
            // Solo verificar operaciones de creación
            if !matches!(op.op_type, OperationType::CreateSymlink) {
                continue;
            }
            
            // Verificar si el target existe
            if fs::metadata(&op.target).await.is_ok() {
                let metadata = fs::symlink_metadata(&op.target).await?;
                
                if metadata.is_file() {
                    // Archivo regular
                    conflicts.push(Conflict {
                        path: op.target.clone(),
                        existing_target: None,
                        new_target: op.source.clone(),
                        conflict_type: ConflictType::RegularFile,
                    });
                } else if metadata.is_dir() {
                    // Verificar si directorio está vacío
                    let mut entries = fs::read_dir(&op.target).await?;
                    if entries.next_entry().await?.is_some() {
                        conflicts.push(Conflict {
                            path: op.target.clone(),
                            existing_target: None,
                            new_target: op.source.clone(),
                            conflict_type: ConflictType::NonEmptyDir,
                        });
                    }
                } else if metadata.file_type().is_symlink() {
                    // Es un symlink, verificar a dónde apunta
                    let existing_target = fs::read_link(&op.target).await?;
                    
                    if existing_target != op.source {
                        conflicts.push(Conflict {
                            path: op.target.clone(),
                            existing_target: Some(existing_target),
                            new_target: op.source.clone(),
                            conflict_type: ConflictType::WrongSymlinkTarget,
                        });
                    }
                }
            }
        }
        
        Ok(conflicts)
    }
    
    /// Intenta resolver conflictos automáticamente
    pub async fn resolve_conflicts(
        conflicts: &[Conflict],
        strategy: ResolutionStrategy,
    ) -> Result<Vec<SymlinkOperation>> {
        let mut operations = Vec::new();
        
        for conflict in conflicts {
            match strategy {
                ResolutionStrategy::Backup => {
                    // Crear operación de backup
                    operations.push(SymlinkOperation {
                        op_type: OperationType::Backup,
                        source: conflict.path.clone(),
                        target: conflict.path.clone(),
                        is_dir: false,
                        description: format!("Backup conflicting file {:?}", conflict.path),
                    });
                }
                ResolutionStrategy::Skip => {
                    warn!("Skipping conflict: {:?}", conflict.path);
                }
                ResolutionStrategy::Abort => {
                    return Err(StowError::Conflict {
                        message: format!("Cannot resolve conflict at {:?}", conflict.path),
                        path: conflict.path.clone(),
                        existing: conflict.existing_target.clone(),
                    });
                }
            }
        }
        
        Ok(operations)
    }
}

/// Estrategia de resolución de conflictos
#[derive(Debug, Clone, Copy)]
pub enum ResolutionStrategy {
    /// Hacer backup de archivos existentes
    Backup,
    /// Omitir archivos conflictivos
    Skip,
    /// Abortar si hay conflictos
    Abort,
}
