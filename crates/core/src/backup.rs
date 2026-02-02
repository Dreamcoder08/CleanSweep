use std::path::{Path, PathBuf};
use crate::Result;
use sha2::{Sha256, Digest};
use tokio::fs;

pub struct BackupManager {
    backup_dir: PathBuf,
    max_backups: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Backup {
    pub path: PathBuf,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub hash: String,
}

impl BackupManager {
    pub fn new(config: &crate::Config) -> Self {
        Self {
            backup_dir: config.backup_dir.clone(),
            max_backups: config.max_backups,
        }
    }
    
    /// Verifica si hay cambios comparando hashes
    pub async fn has_changes(&self) -> Result<bool> {
        let current_hash = self.calculate_state_hash().await?;
        let hash_file = self.backup_dir.join(".last-hash");
        
        if !hash_file.exists() {
            return Ok(true);
        }
        
        let last_hash = fs::read_to_string(&hash_file).await?;
        Ok(current_hash != last_hash)
    }
    
    /// Crea un nuevo backup
    pub async fn create(&self) -> Result<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let backup_path = self.backup_dir.join(format!("backup-{}", timestamp));
        
        fs::create_dir_all(&backup_path).await?;
        
        // Backup de archivos comunes
        let items_to_backup = vec![
            ".config/dreamcoder*",
            ".zshrc",
            ".bashrc",
            ".tmux.conf",
            ".gitconfig",
        ];
        
        for _pattern in items_to_backup {
            // TODO: Implementar glob matching
            // Por ahora es un stub
        }
        
        // Guardar hash del estado
        let hash = self.calculate_state_hash().await?;
        fs::write(self.backup_dir.join(".last-hash"), hash).await?;
        
        // Rotar backups antiguos
        self.rotate_backups().await?;
        
        Ok(backup_path)
    }
    
    /// Lista backups existentes
    pub async fn list(&self) -> Result<Vec<Backup>> {
        let mut backups = Vec::new();
        
        let mut entries = fs::read_dir(&self.backup_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            
            if name_str.starts_with("backup-") {
                // Parsear timestamp del nombre
                let timestamp_str = name_str.trim_start_matches("backup-");
                if let Ok(timestamp) = chrono::NaiveDateTime::parse_from_str(
                    timestamp_str,
                    "%Y%m%d-%H%M%S"
                ) {
                    let datetime = chrono::DateTime::from_naive_utc_and_offset(
                        timestamp,
                        chrono::Utc
                    );
                    
                    backups.push(Backup {
                        path: entry.path(),
                        timestamp: datetime,
                        hash: String::new(), // TODO: Leer hash del backup
                    });
                }
            }
        }
        
        // Ordenar por fecha (más reciente primero)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(backups)
    }
    
    /// Elimina backups antiguos manteniendo solo max_backups
    async fn rotate_backups(&self) -> Result<()> {
        let backups = self.list().await?;
        
        if backups.len() > self.max_backups {
            for backup in &backups[self.max_backups..] {
                fs::remove_dir_all(&backup.path).await?;
            }
        }
        
        Ok(())
    }
    
    /// Calcula hash del estado actual
    async fn calculate_state_hash(&self) -> Result<String> {
        // TODO: Implementar hash de archivos de config
        let mut hasher = Sha256::new();
        hasher.update("placeholder");
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }
}

/// Utilidades de filesystem
pub async fn get_dir_size(_path: &Path) -> Result<u64> {
    // TODO: Implementar async recursivo con Box::pin para evitar recursion infinita
    Ok(0)
}
