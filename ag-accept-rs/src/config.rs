use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use directories::ProjectDirs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub interval: f64,
    pub target_window_title: String,
    pub search_texts_ide: Vec<String>,
    pub search_texts_agent_manager: Vec<String>,
    pub context_text_agent_manager: Vec<String>,
    pub mode: String,
    pub debug_enabled: bool,
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            interval: 1.0,
            target_window_title: "Antigravity".to_string(),
            search_texts_ide: vec!["Run command?".to_string(), "Reject".to_string(), "Accept".to_string()],
            search_texts_agent_manager: vec!["Accept".to_string()],
            context_text_agent_manager: vec!["Run command?".to_string()],
            mode: "AgentManager".to_string(),
            debug_enabled: false,
            window_width: 600,
            window_height: 700,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path();
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: AppConfig = serde_json::from_str(&content)?;
            // In a real app we might want to merge with defaults to handle new keys,
            // but for now strict loading is fine, or we can fallback.
            Ok(config)
        } else {
            // Create default
            let config = Self::default();
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(&config)?;
            fs::write(&config_path, content)?;
            Ok(config)
        }
    }

    pub fn get_config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("", "RyosukeMondo", "ag-accept") {
            proj_dirs.config_dir().join("config.json")
        } else {
            PathBuf::from("config.json")
        }
    }
}
