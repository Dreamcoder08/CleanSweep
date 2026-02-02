use serde::{Deserialize, Serialize};
/// Tipos y estructuras de datos para stow
use std::path::PathBuf;

/// Configuración del sistema stow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StowConfig {
    /// Directorio fuente (donde están los paquetes)
    pub source_dir: PathBuf,

    /// Directorio destino (normalmente $HOME)
    pub target_dir: PathBuf,

    /// Hacer backup de archivos existentes
    pub backup_existing: bool,

    /// Modo simulación (no ejecutar)
    pub simulate: bool,
}

impl Default for StowConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();

        Self {
            source_dir: home
                .join("Documents")
                .join("PROYECTOS")
                .join("Dreamcoder_dots"),
            target_dir: home,
            backup_existing: true,
            simulate: false,
        }
    }
}

/// Tipo de operación stow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    /// Crear symlink
    CreateSymlink,

    /// Eliminar symlink
    RemoveSymlink,

    /// Crear directorio
    CreateDir,

    /// Eliminar directorio vacío
    RemoveDir,

    /// Backup archivo existente
    Backup,
}

/// Operación individual de stow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkOperation {
    pub op_type: OperationType,
    pub source: PathBuf,
    pub target: PathBuf,
    pub is_dir: bool,
    pub description: String,
}

/// Resultado de una operación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub operation: SymlinkOperation,
    pub error: Option<String>,
}

/// Resultado completo de stow/unstow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StowResult {
    pub package: String,
    pub operations: usize,
    pub symlinks_created: usize,
    pub errors: usize,
}

/// Información de un paquete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<PathBuf>,
    pub is_installed: bool,
}
