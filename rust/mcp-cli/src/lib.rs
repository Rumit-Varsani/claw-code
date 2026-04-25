//! MCP CLI - Model Context Protocol CLI
//!
//! This crate provides a command-line interface for interacting with MCP servers
//! and AI models.

pub mod backend;
pub mod config;
pub mod error;
pub mod mcp;
pub mod storage;

pub use backend::Backend;
pub use backend::BackendClient;
pub use config::{ConfigError, ConfigLoader, McpConfig};
pub use error::{McpCliError, ProtocolError, Result};
pub use storage::{Storage, StorageError, StorageManager};

/// Default model name
pub const DEFAULT_MODEL: &str = "llama2";

/// Create a new application
pub fn create_app() -> clap::Command {
    clap::Command::new("mcp-cli")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Your Name <you@example.com>")
        .about("Model Context Protocol CLI for interacting with MCP servers")
        .subcommand_required(true)
}

/// Application exit codes
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_ERROR: i32 = 1;

/// Run the application
pub async fn run(cmd: Option<clap::arg::Matches>) -> Result<()> {
    // TODO: Parse and execute command
    tracing::info!("MCP CLI starting");

    // Load configuration
    let config = ConfigLoader::load()?;

    tracing::debug!("Loaded config: {:?}", config);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model() {
        assert_eq!(DEFAULT_MODEL, "llama2");
    }
}