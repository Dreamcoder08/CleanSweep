//! Tests de integración para el sistema stow nativo

use std::path::PathBuf;
use tempfile::TempDir;
use dreamcoder_stow::{StowConfig, StowManager, StowResult};

#[tokio::test]
async fn test_stow_basic_file() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&target).unwrap();
    
    // Crear estructura de prueba
    let pkg = source.join("test-pkg");
    std::fs::create_dir(&pkg).unwrap();
    std::fs::write(pkg.join(".bashrc"), "# test bashrc").unwrap();
    
    let config = StowConfig {
        source_dir: source,
        target_dir: target.clone(),
        backup_existing: true,
        simulate: false,
    };
    
    let manager = StowManager::new(config);
    let result = manager.stow("test-pkg").await.unwrap();
    
    assert_eq!(result.symlinks_created, 1);
    assert!(target.join(".bashrc").is_symlink());
    
    // Verificar que apunta al source correcto
    let link_target = std::fs::read_link(target.join(".bashrc")).unwrap();
    assert!(link_target.to_string_lossy().contains("test-pkg"));
}

#[tokio::test]
async fn test_stow_nested_directories() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&target).unwrap();
    
    // Crear estructura anidada
    let pkg = source.join("nvim");
    std::fs::create_dir_all(pkg.join(".config").join("nvim")).unwrap();
    std::fs::write(pkg.join(".config").join("nvim").join("init.lua"), "-- config").unwrap();
    
    let config = StowConfig {
        source_dir: source,
        target_dir: target.clone(),
        backup_existing: true,
        simulate: false,
    };
    
    let manager = StowManager::new(config);
    let result = manager.stow("nvim").await.unwrap();
    
    assert!(result.symlinks_created >= 1);
    assert!(target.join(".config").join("nvim").join("init.lua").is_symlink());
}

#[tokio::test]
async fn test_stow_conflict_backup() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&target).unwrap();
    
    // Crear paquete
    let pkg = source.join("test-pkg");
    std::fs::create_dir(&pkg).unwrap();
    std::fs::write(pkg.join(".bashrc"), "# from package").unwrap();
    
    // Crear archivo existente (debería hacer backup)
    std::fs::write(target.join(".bashrc"), "# existing").unwrap();
    
    let config = StowConfig {
        source_dir: source,
        target_dir: target.clone(),
        backup_existing: true,
        simulate: false,
    };
    
    let manager = StowManager::new(config);
    let result = manager.stow("test-pkg").await.unwrap();
    
    // Debería haber creado symlink
    assert!(target.join(".bashrc").is_symlink());
    
    // Debería existir backup
    let backups: Vec<_> = std::fs::read_dir(&target)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(".bashrc.backup"))
        .collect();
    
    assert!(!backups.is_empty(), "Backup file should exist");
}

#[tokio::test]
async fn test_unstow_removes_links() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&target).unwrap();
    
    // Crear e instalar
    let pkg = source.join("test-pkg");
    std::fs::create_dir(&pkg).unwrap();
    std::fs::write(pkg.join(".bashrc"), "# test").unwrap();
    
    let config = StowConfig {
        source_dir: source.clone(),
        target_dir: target.clone(),
        backup_existing: true,
        simulate: false,
    };
    
    let manager = StowManager::new(config.clone());
    manager.stow("test-pkg").await.unwrap();
    
    // Verificar que existe
    assert!(target.join(".bashrc").is_symlink());
    
    // Unstow
    let result = manager.unstow("test-pkg").await.unwrap();
    assert!(result.operations > 0);
    
    // Verificar que se eliminó
    assert!(!target.join(".bashrc").exists());
}

#[tokio::test]
async fn test_verify_detects_broken_links() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&target).unwrap();
    
    // Crear e instalar
    let pkg = source.join("test-pkg");
    std::fs::create_dir(&pkg).unwrap();
    std::fs::write(pkg.join(".bashrc"), "# test").unwrap();
    
    let config = StowConfig {
        source_dir: source.clone(),
        target_dir: target.clone(),
        backup_existing: true,
        simulate: false,
    };
    
    let manager = StowManager::new(config);
    manager.stow("test-pkg").await.unwrap();
    
    // Romper el link eliminando el source
    std::fs::remove_file(pkg.join(".bashrc")).unwrap();
    
    // Verificar debería detectar el link roto
    let broken = manager.verify().await.unwrap();
    assert!(!broken.is_empty(), "Should detect broken symlink");
}

#[tokio::test]
async fn test_list_packages() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&target).unwrap();
    
    // Crear varios paquetes
    for name in ["shell", "nvim", "tmux"] {
        let pkg = source.join(format!("Dreamcoder{}", name));
        std::fs::create_dir(&pkg).unwrap();
    }
    
    let config = StowConfig {
        source_dir: source,
        target_dir: target,
        backup_existing: true,
        simulate: false,
    };
    
    let manager = StowManager::new(config);
    let packages = manager.list_packages().unwrap();
    
    assert_eq!(packages.len(), 3);
    assert!(packages.contains(&"Dreamcodershell".to_string()));
    assert!(packages.contains(&"Dreamcodernvim".to_string()));
    assert!(packages.contains(&"Dreamcodertmux".to_string()));
}
