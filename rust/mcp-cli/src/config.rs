//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// MCP CLI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Global MCP settings
    pub global: GlobalConfig,

    /// List of configured MCP servers
    pub servers: Vec<ServerConfig>,

    /// UI settings
    pub ui: UiConfig,

    /// Storage settings
    pub storage: StorageConfig,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            global: GlobalConfig::default(),
            servers: Vec::new(),
            ui: UiConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

/// Global configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// API endpoint for MCP servers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_endpoint: Option<String>,

    /// API key for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Default response timeout (seconds)
    pub timeout: u64,

    /// Maximum concurrent connections
    pub max_connections: usize,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            api_endpoint: None,
            api_key: None,
            timeout: 30,
            max_connections: 10,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme name
    pub theme: String,

    /// Font size for messages
    pub font_size: usize,

    /// Show typing indicator
    pub show_typing: bool,

    /// Compact mode (smaller UI)
    pub compact: bool,

    /// Maximum message history per session
    pub max_history: usize,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            font_size: 12,
            show_typing: true,
            compact: false,
            max_history: 100,
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database path (SQLite file)
    pub database_path: Option<PathBuf>,

    /// Save chat history
    pub save_history: bool,

    /// Log file path
    pub log_path: Option<PathBuf>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_path: None,
            save_history: false,
            log_path: None,
        }
    }
}

/// Configuration error
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Missing configuration file: {path}")]
    MissingFile { path: PathBuf },

    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Get the config file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(std::path::PathBuf::new)
            .join("mcp-cli")
            .join("config.toml")
    }

    /// Load configuration from file
    pub fn load() -> Result<McpConfig, ConfigError> {
        let path = Self::config_path();

        if !path.exists() {
            return Ok(McpConfig::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let config: McpConfig = toml::from_str(&content)?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(config: &McpConfig) -> Result<(), ConfigError> {
        let path = Self::config_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(config)?;
        std::fs::write(&path, content)?;

        Ok(())
    }

    /// Create a default configuration at the specified path
    pub fn create_default(config_path: PathBuf) -> Result<McpConfig, ConfigError> {
        let config = McpConfig::default();
        let content = toml::to_string_pretty(&config)?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&config_path, content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = McpConfig::default();
        assert_eq!(config.global.timeout, 30);
        assert_eq!(config.ui.font_size, 12);
    }

    #[test]
    fn test_config_serialization() {
        let config = McpConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let _parsed: McpConfig = serde_json::from_str(&json).unwrap();
    }
}