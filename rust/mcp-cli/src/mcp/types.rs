//! MCP Type definitions

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// Message role types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: MessageRole,

    /// Content of the message
    pub content: String,

    /// Timestamp
    pub timestamp: i64,

    /// Optional command that generated this message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

impl Message {
    /// Create a user message
    pub fn user(content: impl Into<String>, timestamp: i64) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            timestamp,
            command: None,
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>, timestamp: i64) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            timestamp,
            command: None,
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>, timestamp: i64) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            timestamp,
            command: None,
        }
    }
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Identifier for the tool
    pub name: String,

    /// Description of what the tool does
    pub description: String,

    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,

    /// Optional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl Tool {
    /// Create a new tool
    pub fn new(name: impl Into<String>, description: impl Into<String>, input_schema: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
            metadata: HashMap::new(),
        }
    }
}

/// Tool call parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// ID of the tool call
    pub id: String,

    /// Name of the tool
    pub name: String,

    /// Parameters for the tool
    pub arguments: serde_json::Value,
}

/// Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// ID of the tool call
    pub tool_call_id: String,

    /// Result content
    pub content: String,
}

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// URI for the resource
    pub uri: String,

    /// Name of the resource
    pub name: String,

    /// Description of the resource
    pub description: String,

    /// MIME type of the resource content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// JSON-RPC Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,

    /// Request ID
    pub id: serde_json::Value,

    /// Method to call
    pub method: String,

    /// Parameters (array or object)
    pub params: serde_json::Value,
}

/// JSON-RPC Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,

    /// Request ID
    pub id: serde_json::Value,

    /// Result (for successful calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Error (for failed calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorResponse>,
}

/// JSON-RPC Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Optional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Capabilities related to tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapabilities>,

    /// Capabilities related to resources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceCapabilities>,

    /// Capabilities related to prompts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptCapabilities>,

    /// General features
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub features: HashMap<String, serde_json::Value>,
}

/// Tool capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    /// List of tools (when provided in initialize response)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Server's list_changed capability
    #[serde(default)]
    pub list_changed: bool,
}

/// Resource capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    /// List of resources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<Resource>>,

    /// Server's list_changed capability
    #[serde(default)]
    pub list_changed: bool,
}

/// Prompt capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilities {
    /// List of prompts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<Vec<serde_json::Value>>,

    /// Server's list_changed capability
    #[serde(default)]
    pub list_changed: bool,
}

/// MCP Initialize Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    /// Protocol version
    pub protocol_version: String,

    /// Client capabilities
    pub client_capabilities: ClientCapabilities,

    /// Instance ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_info: Option<VersionInfo>,
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Root capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootCapabilities>,

    /// Workspace capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceCapabilities>,

    /// Feature capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, serde_json::Value>>,
}

/// Root capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCapabilities {
    /// List of roots
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Workspace capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceCapabilities {
    /// Workspace configuration capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<ConfigurationCapability>,
}

/// Configuration capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationCapability {
    /// Support for workspace configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<bool>,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Name of the client
    pub name: String,

    /// Version string
    pub version: String,
}

/// Initialize Response result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResponse {
    /// Protocol version
    pub protocol_version: String,

    /// Server capabilities
    pub capabilities: ServerCapabilities,

    /// Server info
    pub server_info: VersionInfo,
}

/// Message types for the protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Client to server
    ClientNotification,

    /// Server to client
    ServerNotification,

    /// Client to server (request)
    ClientRequest,

    /// Server to client (response)
    ServerResponse,
}

/// Create a JSON-RPC notification
pub fn notification(method: impl Into<String>, params: serde_json::Value) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": method.into(),
        "params": params
    })
}

/// Create a JSON-RPC request
pub fn request(method: impl Into<String>, params: serde_json::Value, id: impl Into<serde_json::Value>) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": method.into(),
        "params": params,
        "id": id.into()
    })
}

/// Create a successful JSON-RPC response
pub fn success_response(
    result: impl Into<serde_json::Value>,
    id: impl Into<serde_json::Value>,
) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "result": result.into(),
        "id": id.into()
    })
}

/// Create an error JSON-RPC response
pub fn error_response(
    code: i32,
    message: impl Into<String>,
    id: impl Into<serde_json::Value>,
    data: Option<serde_json::Value> = None,
) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "error": {
            "code": code,
            "message": message.into(),
            "data": data
        },
        "id": id.into()
    })
}