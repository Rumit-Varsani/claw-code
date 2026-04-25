//! Error types

use thiserror::Error;

/// Main error type for MCP CLI
#[derive(Error, Debug)]
pub enum McpCliError {
    /// Generic error with message
    #[error("{0}")]
    Generic(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// HTTP error
    #[error("HTTP error: {0}")]
    Http(String),

    /// Backend error
    #[error("Backend error: {0}")]
    Backend(String),

    /// MCP protocol error
    #[error("MCP protocol error: {0}")]
    Protocol(String),

    /// Server error
    #[error("Server error ({}): {}", name, message)]
    Server {
        name: String,
        message: String,
    },

    /// Tool error
    #[error("Tool error ({name}): {message}")]
    Tool {
        name: String,
        message: String,
    },

    /// Resource error
    #[error("Resource error ({name}): {message}")]
    Resource {
        name: String,
        message: String,
    },

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Timeout
    #[error("Operation timed out after {0} seconds")]
    Timeout(u64),
}

impl McpCliError {
    /// Create a new generic error
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a connection error
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
}

/// Error types for MCP protocol
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid JSON-RPC: {0}")]
    JsonRpc(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Server error: {0}")]
    Server(String),
}

impl From<ProtocolError> for McpCliError {
    fn from(err: ProtocolError) -> Self {
        match err {
            ProtocolError::InvalidFormat(msg) => Self::Protocol(format!("Invalid format: {msg}")),
            ProtocolError::InvalidRequest(msg) => Self::Protocol(format!("Invalid request: {msg}")),
            ProtocolError::InvalidResponse(msg) => Self::Protocol(format!("Invalid response: {msg}")),
            ProtocolError::MissingField { field } => Self::Protocol(format!("Missing field: {field}")),
            ProtocolError::JsonRpc(msg) => Self::Protocol(format!("JSON-RPC error: {msg}")),
            ProtocolError::MethodNotFound(name) => Self::Protocol(format!("Method not found: {name}")),
            ProtocolError::InvalidParameters(msg) => Self::Protocol(format!("Invalid parameters: {msg}")),
            ProtocolError::Transport(msg) => Self::Protocol(format!("Transport error: {msg}")),
            ProtocolError::Server(msg) => Self::Protocol(format!("Server error: {msg}")),
        }
    }
}

impl From<rusqlite::Error> for McpCliError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Storage(format!("Database error: {}", err))
    }
}

impl From<ProtocolError> for ProtocolError {
    fn from(err: ProtocolError) -> Self {
        err
    }
}

/// Result type for MCP CLI
pub type Result<T> = std::result::Result<T, McpCliError>;