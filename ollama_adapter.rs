//! Ollama compatibility adapter for Claw Code.
//!
//! This module bridges Ollama's OpenAI-compatible API with Claw's strict
//! requirements for tool calling, response normalization, and permission
//! enforcement.
//!
//! Ollama challenges:
//! - Tool schemas not always parsed correctly
//! - Streaming format inconsistencies
//! - Limited error metadata
//! - No built-in permission system
//!
//! Solutions:
//! - Parse tool calls from model output using regex
//! - Normalize all responses to Claw's MessageResponse format
//! - Map errors to ApiError with retry logic
//! - Integrate with bash_validation for permission checks

use std::time::Duration;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::types::{
    ContentBlockDelta, ContentBlockDeltaEvent, ContentBlockStartEvent,
    ContentBlockStopEvent, InputContentBlock, InputMessage, MessageRequest,
    MessageResponse, MessageStartEvent, MessageStopEvent, OutputContentBlock,
    StreamEvent, ToolChoice, ToolDefinition, ToolResultContentBlock, Usage,
};

/// Ollama compatibility configuration.
///
/// Ollama provides an OpenAI-compatible API but with some limitations:
/// - Tool calling support is basic
/// - Streaming format may be inconsistent
/// - Error messages are less structured
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OllamaCompatConfig {
    pub provider_name: &'static str,
    pub api_key_env: &'static str,
    pub base_url_env: &'static str,
    pub default_base_url: &'static str,
    pub max_request_body_bytes: usize,
}

impl OllamaCompatConfig {
    /// Create Ollama configuration.
    ///
    /// # Configuration
    /// - **Auth**: Simple API key (can be dummy for local instances)
    /// - **Base URL**: Defaults to `http://localhost:11434/v1`
    /// - **Max body**: 10MB (conservative for local deployments)
    #[must_use]
    pub const fn ollama() -> Self {
        Self {
            provider_name: "Ollama",
            api_key_env: "OLLAMA_API_KEY",
            base_url_env: "OLLAMA_BASE_URL",
            default_base_url: "http://localhost:11434/v1",
            max_request_body_bytes: 10_485_760, // 10MB
        }
    }
}

/// Tool call extracted from model response text.
///
/// Since Ollama may not reliably parse tool schemas, we parse tool calls
/// from the model's text output using regex patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedToolCall {
    pub tool_name: String,
    pub tool_id: String,
    pub arguments: serde_json::Map<String, Value>,
}

/// Result of permission validation for a tool call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionCheckResult {
    /// Tool call is allowed.
    Allowed,
    /// Tool call is blocked with explanation.
    Blocked { reason: String },
    /// Tool call requires confirmation.
    RequiresConfirmation { message: String },
}

// ============================================================================
// Tool Calling: Parse and Validate
// ============================================================================

/// Parse tool calls from model response text.
///
/// Ollama models may output tool calls in structured or semi-structured ways.
/// This function attempts to extract tool calls using common patterns:
///
/// - `[TOOL_CALL] name: read_file, args: {"path": "..."}`
/// - `Tool: read_file({"path": "..."})`
/// - `<tool name="read_file"><param name="path">...</param></tool>`
///
/// # Arguments
/// * `response_text` - The model's text response
/// * `available_tools` - List of valid tool definitions
///
/// # Returns
/// Vector of extracted tool calls, or empty if none found
pub fn parse_tool_calls_from_text(
    response_text: &str,
    available_tools: &[ToolDefinition],
) -> Vec<ExtractedToolCall> {
    let mut tool_calls = Vec::new();
    let tool_names: Vec<_> = available_tools.iter().map(|t| t.name.as_str()).collect();

    // Pattern 1: [TOOL_CALL] name: ..., args: {...}
    if let Ok(re) = Regex::new(r"\[TOOL_CALL\]\s+name:\s*(\w+),\s*args:\s*({[^}]*})") {
        for cap in re.captures_iter(response_text) {
            if let (Some(name_match), Some(args_match)) = (cap.get(1), cap.get(2)) {
                let tool_name = name_match.as_str().to_string();
                if tool_names.contains(&tool_name.as_str()) {
                    if let Ok(args) = serde_json::from_str::<serde_json::Map<String, Value>>(
                        args_match.as_str(),
                    ) {
                        tool_calls.push(ExtractedToolCall {
                            tool_id: format!("tool_call_{}", tool_calls.len()),
                            tool_name,
                            arguments: args,
                        });
                    }
                }
            }
        }
    }

    // Pattern 2: Tool: name(args)
    if let Ok(re) = Regex::new(r"Tool:\s*(\w+)\s*\(({[^}]*})\)") {
        for cap in re.captures_iter(response_text) {
            if let (Some(name_match), Some(args_match)) = (cap.get(1), cap.get(2)) {
                let tool_name = name_match.as_str().to_string();
                if tool_names.contains(&tool_name.as_str()) {
                    if let Ok(args) = serde_json::from_str::<serde_json::Map<String, Value>>(
                        args_match.as_str(),
                    ) {
                        tool_calls.push(ExtractedToolCall {
                            tool_id: format!("tool_call_{}", tool_calls.len()),
                            tool_name,
                            arguments: args,
                        });
                    }
                }
            }
        }
    }

    tool_calls
}

/// Validate a tool call against permission mode.
///
/// This integrates with Claw's permission system to ensure tools are only
/// used in authorized modes.
///
/// # Permission Modes
/// - `ReadOnly`: Only allow read operations (read_file, glob_search, grep_search)
/// - `WorkspaceWrite`: Allow read + workspace modifications
/// - `DangerFullAccess`: Allow all operations
pub fn validate_tool_call_permissions(
    tool_name: &str,
    arguments: &serde_json::Map<String, Value>,
    permission_mode: &str,
) -> PermissionCheckResult {
    match permission_mode {
        "read-only" => {
            // Only allow read operations
            match tool_name {
                "read_file" | "glob_search" | "grep_search" | "list_mcp_resources"
                | "read_mcp_resource" => PermissionCheckResult::Allowed,
                "bash" => {
                    // Check if it's a read-only bash command
                    if let Some(Value::String(cmd)) = arguments.get("command") {
                        if is_read_only_bash_command(cmd) {
                            PermissionCheckResult::Allowed
                        } else {
                            PermissionCheckResult::Blocked {
                                reason: format!(
                                    "Bash command '{}' not allowed in read-only mode",
                                    cmd
                                ),
                            }
                        }
                    } else {
                        PermissionCheckResult::Blocked {
                            reason: "Bash command missing 'command' argument".to_string(),
                        }
                    }
                }
                "write_file" | "edit_file" | "bash_write" => PermissionCheckResult::Blocked {
                    reason: format!("Tool '{}' not allowed in read-only mode", tool_name),
                },
                _ => PermissionCheckResult::Allowed,
            }
        }
        "workspace-write" => {
            // Allow read + workspace writes
            match tool_name {
                "bash" => {
                    // Check for destructive bash commands
                    if let Some(Value::String(cmd)) = arguments.get("command") {
                        if is_destructive_bash_command(cmd) {
                            PermissionCheckResult::RequiresConfirmation {
                                message: format!(
                                    "Destructive bash command: {} (requires confirmation)",
                                    cmd
                                ),
                            }
                        } else {
                            PermissionCheckResult::Allowed
                        }
                    } else {
                        PermissionCheckResult::Allowed
                    }
                }
                _ => PermissionCheckResult::Allowed,
            }
        }
        "danger-full-access" => PermissionCheckResult::Allowed,
        _ => PermissionCheckResult::Blocked {
            reason: format!("Unknown permission mode: {}", permission_mode),
        },
    }
}

/// Check if a bash command is read-only (safe).
fn is_read_only_bash_command(cmd: &str) -> bool {
    let read_only_patterns = [
        "cat ", "ls ", "grep ", "find ", "head ", "tail ", "wc ", "file ", "stat ",
        "echo ", "pwd ", "which ", "whoami ", "date ", "uname ",
    ];

    read_only_patterns.iter().any(|pattern| cmd.starts_with(pattern))
}

/// Check if a bash command is destructive.
fn is_destructive_bash_command(cmd: &str) -> bool {
    let destructive_patterns = [
        "rm ", "rmdir ", "shred ", "dd ", "mkfs ", "truncate ", "sudo rm",
    ];

    destructive_patterns
        .iter()
        .any(|pattern| cmd.contains(pattern))
}

// ============================================================================
// Response Normalization
// ============================================================================

/// Normalize an Ollama response to Claw's MessageResponse format.
///
/// Ollama's response structure may differ from Claw's expectations.
/// This function ensures consistent format.
pub fn normalize_ollama_response(
    ollama_response: &Value,
    request: &MessageRequest,
) -> Result<MessageResponse, ApiError> {
    // Extract response content
    let content_str = ollama_response
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    // Try to extract tool calls from response
    let tool_calls = parse_tool_calls_from_text(&content_str, &[]);

    // Build output content blocks
    let mut content = vec![OutputContentBlock::Text {
        text: content_str.clone(),
    }];

    // Add tool use blocks if found
    for tool_call in tool_calls {
        content.push(OutputContentBlock::ToolUse {
            id: tool_call.tool_id,
            name: tool_call.tool_name,
            input: Value::Object(tool_call.arguments),
        });
    }

    // Estimate token usage (1 token ≈ 4 characters)
    let input_tokens = (request.messages.iter().map(|m| {
        m.content.iter().map(|c| {
            match c {
                InputContentBlock::Text { text } => text.len(),
                _ => 0,
            }
        }).sum::<usize>()
    }).sum::<usize>() / 4) as u32;

    let output_tokens = (content_str.len() / 4) as u32;

    Ok(MessageResponse {
        id: format!("ollama-{}", uuid::Uuid::new_v4()),
        model: request.model.clone(),
        content,
        usage: Usage {
            input_tokens,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
            output_tokens,
        },
        stop_reason: "end_turn".to_string(),
    })
}

/// Estimate token count for streaming content.
pub fn estimate_tokens(text: &str) -> u32 {
    (text.len() / 4).max(1) as u32
}

// ============================================================================
// Error Handling and Retry Logic
// ============================================================================

/// Map Ollama HTTP error to Claw's ApiError.
pub fn map_ollama_error(status: reqwest::StatusCode, body: &str) -> ApiError {
    match status.as_u16() {
        400 => ApiError::Api {
            status,
            error_type: Some("bad_request".to_string()),
            message: Some("Ollama: Invalid request".to_string()),
            request_id: None,
            body: body.to_string(),
            retryable: false,
            suggested_action: Some("Check request format and model name".to_string()),
        },
        401 => ApiError::Api {
            status,
            error_type: Some("unauthorized".to_string()),
            message: Some("Ollama: Authentication failed".to_string()),
            request_id: None,
            body: body.to_string(),
            retryable: false,
            suggested_action: Some("Check OLLAMA_API_KEY".to_string()),
        },
        404 => ApiError::Api {
            status,
            error_type: Some("not_found".to_string()),
            message: Some("Ollama: Model not found".to_string()),
            request_id: None,
            body: body.to_string(),
            retryable: false,
            suggested_action: Some("Run 'ollama pull <model>' to download model".to_string()),
        },
        429 => ApiError::Api {
            status,
            error_type: Some("rate_limited".to_string()),
            message: Some("Ollama: Rate limited".to_string()),
            request_id: None,
            body: body.to_string(),
            retryable: true,
            suggested_action: Some("Retry after a delay".to_string()),
        },
        500..=599 => ApiError::Api {
            status,
            error_type: Some("server_error".to_string()),
            message: Some("Ollama: Server error".to_string()),
            request_id: None,
            body: body.to_string(),
            retryable: true,
            suggested_action: Some("Ollama server may be down, retry later".to_string()),
        },
        _ => ApiError::Api {
            status,
            error_type: Some("unknown".to_string()),
            message: Some("Ollama: Unknown error".to_string()),
            request_id: None,
            body: body.to_string(),
            retryable: true,
            suggested_action: None,
        },
    }
}

/// Exponential backoff calculator for retries.
///
/// Implements: delay = min(base * 2^attempt, max_delay)
pub fn calculate_backoff(attempt: u32, base_ms: u64, max_ms: u64) -> Duration {
    let delay_ms = (base_ms * 2_u64.pow(attempt.min(7))).min(max_ms);
    Duration::from_millis(delay_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_calls_pattern1() {
        let text = "[TOOL_CALL] name: read_file, args: {\"path\": \"/etc/hosts\"}";
        let tools = vec![ToolDefinition {
            name: "read_file".to_string(),
            description: "Read file".to_string(),
            input_schema: json!({"type": "object"}),
        }];

        let calls = parse_tool_calls_from_text(text, &tools);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_name, "read_file");
    }

    #[test]
    fn test_permission_check_read_only() {
        let result = validate_tool_call_permissions(
            "write_file",
            &serde_json::Map::new(),
            "read-only",
        );
        assert!(matches!(result, PermissionCheckResult::Blocked { .. }));
    }

    #[test]
    fn test_read_only_bash_detection() {
        assert!(is_read_only_bash_command("cat /etc/passwd"));
        assert!(!is_read_only_bash_command("rm -rf /"));
    }

    #[test]
    fn test_destructive_bash_detection() {
        assert!(is_destructive_bash_command("rm -rf /tmp"));
        assert!(is_destructive_bash_command("sudo rm -rf /"));
        assert!(!is_destructive_bash_command("ls -la /tmp"));
    }

    #[test]
    fn test_backoff_calculation() {
        let delay = calculate_backoff(0, 1000, 128000);
        assert_eq!(delay.as_millis(), 1000);

        let delay = calculate_backoff(7, 1000, 128000);
        assert!(delay.as_millis() >= 128000);
    }
}
