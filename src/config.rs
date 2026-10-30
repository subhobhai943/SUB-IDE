use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Persisted IDE configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeConfig {
    pub ui_scale: f32,
    pub last_workspace: Option<PathBuf>,
}

impl Default for IdeConfig {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            last_workspace: None,
        }
    }
}

impl IdeConfig {
    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&content) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sub-ide")
            .join("config.json")
    }

    pub fn set_ui_scale(&mut self, scale: f32) {
        self.ui_scale = scale.clamp(0.5, 2.5);
    }

    pub fn set_last_workspace<P: AsRef<Path>>(&mut self, path: P) {
        self.last_workspace = Some(path.as_ref().to_path_buf());
    }
}
