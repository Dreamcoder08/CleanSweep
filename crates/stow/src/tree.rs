//! Gestión del árbol de directorios stow
//!
//! Maneja la estructura de directorios y la fusión de árboles.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Representa un nodo en el árbol de directorios
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: HashMap<String, TreeNode>,
    pub source_package: Option<String>,
}

/// Árbol de directorios para stow
pub struct StowTree {
    root: TreeNode,
}

impl StowTree {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root: TreeNode {
                name: "".to_string(),
                path: root_path,
                is_dir: true,
                children: HashMap::new(),
                source_package: None,
            },
        }
    }

    /// Inserta un archivo en el árbol
    pub fn insert(&mut self, relative_path: &Path, package: &str) {
        let components: Vec<_> = relative_path.components().collect();
        Self::insert_recursive(&mut self.root, &components, 0, package);
    }

    fn insert_recursive(
        node: &mut TreeNode,
        components: &[std::path::Component],
        index: usize,
        package: &str,
    ) {
        if index >= components.len() {
            return;
        }

        let component = &components[index];
        let name = component.as_os_str().to_string_lossy().to_string();

        let is_last = index == components.len() - 1;

        // Insert or get child
        if !node.children.contains_key(&name) {
            let child_path = node.path.join(&name);
            let child = TreeNode {
                name: name.clone(),
                path: child_path,
                is_dir: !is_last,
                children: HashMap::new(),
                source_package: if is_last {
                    Some(package.to_string())
                } else {
                    None
                },
            };
            node.children.insert(name.clone(), child);
        }

        // Get mutable reference to child and continue recursively
        if let Some(child) = node.children.get_mut(&name) {
            if !is_last {
                Self::insert_recursive(child, components, index + 1, package);
            } else {
                // Mark as file from this package
                child.source_package = Some(package.to_string());
            }
        }
    }

    /// Obtiene todos los archivos (hojas) del árbol
    pub fn get_files(&self) -> Vec<(PathBuf, String)> {
        let mut files = Vec::new();
        Self::collect_files(&self.root, &mut files);
        files
    }

    fn collect_files(node: &TreeNode, files: &mut Vec<(PathBuf, String)>) {
        if !node.is_dir {
            if let Some(ref package) = node.source_package {
                files.push((node.path.clone(), package.clone()));
            }
        } else {
            for child in node.children.values() {
                Self::collect_files(child, files);
            }
        }
    }

    /// Detecta solapamientos entre paquetes
    pub fn detect_overlaps(&self) -> Vec<Overlap> {
        let mut overlaps = Vec::new();
        let files = self.get_files();

        // Agrupar por ruta
        let mut by_path: HashMap<PathBuf, Vec<String>> = HashMap::new();
        for (path, package) in files {
            by_path.entry(path).or_default().push(package);
        }

        // Encontrar rutas con múltiples paquetes
        for (path, packages) in by_path {
            if packages.len() > 1 {
                overlaps.push(Overlap { path, packages });
            }
        }

        overlaps
    }
}

/// Solapamiento detectado entre paquetes
#[derive(Debug, Clone)]
pub struct Overlap {
    pub path: PathBuf,
    pub packages: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_insert() {
        let mut tree = StowTree::new(PathBuf::from("/target"));

        tree.insert(Path::new(".config/nvim/init.lua"), "nvim");
        tree.insert(Path::new(".config/git/config"), "git");
        tree.insert(Path::new(".zshrc"), "shell");

        let files = tree.get_files();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_detect_overlaps() {
        let mut tree = StowTree::new(PathBuf::from("/target"));

        tree.insert(Path::new(".config/shared/config"), "package1");
        tree.insert(Path::new(".config/shared/config"), "package2");

        let overlaps = tree.detect_overlaps();
        assert_eq!(overlaps.len(), 1);
        assert_eq!(overlaps[0].packages.len(), 2);
    }
}
