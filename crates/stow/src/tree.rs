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
    pub source_packages: Vec<String>,
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
                source_packages: Vec::new(),
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
        let child = node.children.entry(name.clone()).or_insert_with(|| {
            let child_path = node.path.join(&name);
            TreeNode {
                name: name.clone(),
                path: child_path,
                is_dir: !is_last,
                children: HashMap::new(),
                source_packages: Vec::new(),
            }
        });

        if is_last {
            child.is_dir = false;
            if !child.source_packages.contains(&package.to_string()) {
                child.source_packages.push(package.to_string());
            }
        } else {
            child.is_dir = true;
            Self::insert_recursive(child, components, index + 1, package);
        }
    }

    /// Obtiene todos los archivos (hojas) del árbol
    pub fn get_files(&self) -> Vec<(PathBuf, Vec<String>)> {
        let mut files = Vec::new();
        Self::collect_files(&self.root, &mut files);
        files
    }

    fn collect_files(node: &TreeNode, files: &mut Vec<(PathBuf, Vec<String>)>) {
        if !node.is_dir {
            if !node.source_packages.is_empty() {
                files.push((node.path.clone(), node.source_packages.clone()));
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

        for (path, packages) in files {
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