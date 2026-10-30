use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Represents a change event from the filesystem or fsnotifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsEventType {
    Create,
    Change,
    Delete,
    Stats,
    Reset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FsEvent {
    pub event_type: FsEventType,
    pub path: PathBuf,
}

/// A node in the Virtual File System.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualFile {
    pub id: u64,
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub size: usize,
    pub content: Option<String>,
    pub children: Vec<u64>,
    pub parent: Option<u64>,
    pub modification_seq: u64,
}

/// The Virtual File System (VFS), replicating IntelliJ's VFS architecture.
/// Caches the directory hierarchy and file contents in-memory for zero-latency IDE operations.
pub struct VirtualFileSystem {
    next_id: AtomicU64,
    nodes: HashMap<u64, VirtualFile>,
    path_to_id: HashMap<PathBuf, u64>,
    watched_roots: Vec<PathBuf>,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            nodes: HashMap::new(),
            path_to_id: HashMap::new(),
            watched_roots: Vec::new(),
        }
    }

    /// Add a root directory to be indexed and tracked by the VFS
    pub fn add_root<P: AsRef<Path>>(&mut self, root_path: P) -> Result<u64, String> {
        let root_path = root_path.as_ref().to_path_buf();
        if !root_path.exists() {
            return Err(format!("Root path does not exist: {:?}", root_path));
        }

        self.watched_roots.push(root_path.clone());
        self.index_path_recursive(&root_path, None)
    }

    /// Recursively scan and load files into the VFS
    fn index_path_recursive(&mut self, path: &Path, parent_id: Option<u64>) -> Result<u64, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());
        let is_dir = path.is_dir();

        let (size, content) = if is_dir {
            (0, None)
        } else {
            match std::fs::read_to_string(path) {
                Ok(text) => (text.len(), Some(text)),
                Err(_) => (0, None), // Non-utf8 or binary file
            }
        };

        let mut child_ids = Vec::new();
        if is_dir {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let child_path = entry.path();
                    // Skip hidden files like .git or __pycache__
                    if let Some(file_name) = child_path.file_name() {
                        let name_str = file_name.to_string_lossy();
                        if name_str.starts_with('.') || name_str == "__pycache__" {
                            continue;
                        }
                    }
                    if let Ok(child_id) = self.index_path_recursive(&child_path, Some(id)) {
                        child_ids.push(child_id);
                    }
                }
            }
        }

        let vfile = VirtualFile {
            id,
            path: path.to_path_buf(),
            name,
            is_directory: is_dir,
            size,
            content,
            children: child_ids,
            parent: parent_id,
            modification_seq: 1,
        };

        self.path_to_id.insert(path.to_path_buf(), id);
        self.nodes.insert(id, vfile);

        Ok(id)
    }

    /// Find a VirtualFile by its path
    pub fn find_file<P: AsRef<Path>>(&self, path: P) -> Option<&VirtualFile> {
        self.path_to_id
            .get(path.as_ref())
            .and_then(|id| self.nodes.get(id))
    }

    /// Find a VirtualFile by its ID
    pub fn get_file(&self, id: u64) -> Option<&VirtualFile> {
        self.nodes.get(&id)
    }

    /// Find a mutable VirtualFile by its ID
    pub fn get_file_mut(&mut self, id: u64) -> Option<&mut VirtualFile> {
        self.nodes.get_mut(&id)
    }

    /// Get all indexed Python files
    pub fn find_all_python_files(&self) -> Vec<&VirtualFile> {
        self.nodes
            .values()
            .filter(|f| {
                !f.is_directory && f.path.extension().and_then(|e| e.to_str()) == Some("py")
            })
            .collect()
    }

    /// Process an fsnotifier wire-protocol stream.
    /// Format:
    /// <EVENT_TYPE>\n
    /// <PATH>\n
    pub fn apply_fsnotifier_event(&mut self, event_type_str: &str, path_str: &str) {
        let path = PathBuf::from(path_str);
        match event_type_str {
            "CREATE" | "CHANGE" => {
                if path.exists() {
                    if let Some(&id) = self.path_to_id.get(&path) {
                        if let Some(vfile) = self.nodes.get_mut(&id) {
                            if !vfile.is_directory {
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    vfile.size = content.len();
                                    vfile.content = Some(content);
                                    vfile.modification_seq += 1;
                                }
                            }
                        }
                    } else {
                        // Brand new file
                        let parent_id = path.parent().and_then(|p| self.path_to_id.get(p).copied());
                        if let Ok(new_id) = self.index_path_recursive(&path, parent_id) {
                            if let Some(pid) = parent_id {
                                if let Some(parent_node) = self.nodes.get_mut(&pid) {
                                    if !parent_node.children.contains(&new_id) {
                                        parent_node.children.push(new_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "DELETE" => {
                if let Some(id) = self.path_to_id.remove(&path) {
                    if let Some(node) = self.nodes.remove(&id) {
                        if let Some(pid) = node.parent {
                            if let Some(parent_node) = self.nodes.get_mut(&pid) {
                                parent_node.children.retain(|&c| c != id);
                            }
                        }
                    }
                }
            }
            "STATS" => {
                // Metadata update (timestamp, permissions)
            }
            "RESET" => {
                // Queue overflow: reload all watched roots
                let roots = self.watched_roots.clone();
                self.nodes.clear();
                self.path_to_id.clear();
                self.watched_roots.clear();
                for r in roots {
                    let _ = self.add_root(r);
                }
            }
            _ => {}
        }
    }

    /// Generate the fsnotifier command to watch all roots
    pub fn format_fsnotifier_watch_command(&self) -> String {
        let mut cmd = String::from("ROOTS\n");
        for root in &self.watched_roots {
            cmd.push_str(&root.to_string_lossy());
            cmd.push('\n');
        }
        cmd.push_str("#\n");
        cmd
    }
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}
