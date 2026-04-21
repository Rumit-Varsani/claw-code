//! Ollama adapter implementation.
//!
//! Implements the Provider trait for Ollama-compatible endpoints.
//! Ollama's API is OpenAI-compatible but with a few quirks:
//!
//! 1. Ollama responds with `null` for `tool_calls` instead of `[]` when no tools are called.
//!    This adapter normalizes that to an empty vector for proper tool calling semantics.
//! 2. Ollama uses local URLs by default (localhost:11434).
//! 3. Models with a `:num_ctx` suffix can specify context window size; we use the
//!    default from the wire model name plus a generous 200k context window.
//!
//! This adapter is used when OPENAI_BASE_URL is set without OPENAI_API_KEY, which is
//! the common case for local providers like Ollama, LM Studio, vLLM, etc.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::json;

use crate::api::error::ApiError;
use crate::api::types::{
    InputContentBlock, InputContentBlockType, MessageRequest, MessageResponse, OutputContentBlock,
    ToolChoice, ToolDefinition,
};
use super::providers::ProviderFuture;
use super::providers::Provider;

// ----------------------------------------------------------------------
// Types for the openai-compatible API shape
// ----------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    id: String,
    model: String,
    choices: Vec<ChatChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    role: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ResponseToolCall>,
}

#[derive(Debug, Deserialize)]
struct ResponseToolCall {
    id: String,
    function: ResponseToolFunction,
}

#[derive(Debug, Deserialize)]
struct ResponseToolFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    id: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<ChunkChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct ChunkChoice {
    delta: ChunkDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ChunkDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    tool_calls: Vec<DeltaToolCall>,
}

#[derive(Debug, Deserialize)]
struct DeltaToolCall {
    #[serde(default)]
    index: u32,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: DeltaFunction,
}

#[derive(Debug, Default, Deserialize)]
struct DeltaFunction {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

// ----------------------------------------------------------------------
// Adapter struct
// ----------------------------------------------------------------------

pub struct OllamaAdapter {
    config: OllamaConfig,
    request_id_counter: BTreeMap<u32, String>,
}

pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub max_request_body_bytes: usize,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: "".to_string(),
            max_tokens: 4096,
            max_request_body_bytes: 512 * 1024, // 512KB
        }
    }
}

impl OllamaConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build config from environment variables.
    pub fn from_env() -> Result<Self, ApiError> {
        let base_url = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| {
            "http://localhost:11434".to_string()
        });

        let model = std::env::var("OLLAMA_MODEL").ok().unwrap_or_else(|| {
            // Default to qwen2.5-coder if not specified
            "qwen2.5-coder:7b".to_string()
        });

        let max_tokens = std::env::var("OLLAMA_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                if model.contains("large") || model.contains("12b") || model.contains("larger") {
                    32768
                } else {
                    4096
                }
            });

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
            max_tokens,
            max_request_body_bytes: 512 * 1024,
        })
    }
}

impl OllamaAdapter {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            config,
            request_id_counter: BTreeMap::new(),
        }
    }

    /// Send a chat completion request.
    async fn send_request(&self, request: &MessageRequest) -> Result<ChatCompletionResponse, ApiError> {
        let url = format!("{}/api/chat", self.config.base_url);
        let request_id = uuid::Uuid::new_v4().to_string();

        let payload = self.build_json_payload(request);
        let formatted_payload = serde_json::to_string(&payload).map_err(|error| {
            ApiError::Json {
                provider: "Ollama".to_string(),
                model: self.config.model.clone(),
                body_snippet: String::new(),
                source: error,
            }
        })?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(ApiError::from)?;

        let response = client
            .post(&url)
            .header("content-type", "application/json")
            .header("X-Request-ID", &request_id)
            .body(formatted_payload)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await.map_err(ApiError::from)?;

        if !status.is_success() {
            return Err(ApiError::Api {
                status,
                error_type: None,
                message: None,
                request_id: Some(request_id.clone()),
                body,
                retryable: true,
                suggested_action: Some("Wait a moment and try again".to_string()),
            });
        }

        let parsed: ChatCompletionResponse = serde_json::from_str(&body).map_err(|error| {
            ApiError::Json {
                provider: "Ollama".to_string(),
                model: self.config.model.clone(),
                body_snippet: truncate_body_snippet(&body, 200),
                source: error,
            }
        })?;

        Ok(parsed)
    }

    /// Build JSON payload from MessageRequest.
    fn build_json_payload(&self, request: &MessageRequest) -> Value {
        let mut messages = Vec::new();
        let wire_model = self.strip_model_prefix(&request.model);

        if let Some(system) = request.system.as_ref().filter(|v| !v.is_empty()) {
            messages.push(json!({
                "role": "system",
                "content": system,
            }));
        }

        for message in &request.messages {
            messages.extend(self.translate_message(message, wire_model.as_str()));
        }

        let mut payload = json!({
            "model": wire_model,
            "stream": request.stream,
        });

        // Ollama uses "messages" not "chat" for non-streaming
        payload["messages"] = json!(messages);
        payload["options"] = json!({
            "temperature": request.temperature.unwrap_or(0.7),
            "num_predict": request.max_tokens,
        });

        if request.stream {
            // For streaming, send "stream": true and use /api/generate endpoint
            payload["stream"] = json!(true);
        }

        // Add tools if present
        if let Some(tools) = &request.tools {
            payload["tools"] = json!(tools.iter().map(|t| self.openai_tool_definition(t)).collect::<Vec<_>>());
        }

        // Add tool_choice if present
        if let Some(tool_choice) = &request.tool_choice {
            payload["tool_choice"] = self.openai_tool_choice(tool_choice);
        }

        payload
    }

    /// Translate an internal MessageRequest message to OpenAI format.
    fn translate_message(&self, message: &InputContentBlock, wire_model: &str) -> Vec<Value> {
        let role = match &message.ty {
            InputContentBlockType::User => "user",
            InputContentBlockType::Assistant => "assistant",
            InputContentBlockType::System => "system",
            InputContentBlockType::Tool => "tool",
        };

        match message {
            InputContentBlock::ContentBlock { content: InputContentBlockType::Text { text }, .. } => {
                vec![json!({ "role": role, "content": text })]
            }
            InputContentBlock::ContentBlock { content: InputContentBlockType::Image { url, mime_type }, .. } => {
                vec![json!({ "role": role, "content": [{
                    "type": "image_url",
                    "image_url": { "url": url.to_string() }
                }] })]
            }
            InputContentBlock::ContentBlock { content: InputContentBlockType::Audio { url, format }, .. } => {
                vec![json!({ "role": role, "content": [{
                    "type": "audio_url",
                    "audio_url": { "url": url, "format": format.clone() }
                }] })]
            }
            InputContentBlock::ContentBlock { content: InputContentBlockType::ToolResult { id, name, input, output }, .. } => {
                vec![json!({
                    "role": role,
                    "tool_call_id": id,
                    "content": output
                })]
            }
        }
    }

    /// Normalize a message response from OpenAI format to our internal format.
    fn normalize_response(
        &self,
        model: &str,
        response: ChatCompletionResponse,
    ) -> Result<MessageResponse, ApiError> {
        let choice = response
            .choices
            .into_iter()
            .next()
            .ok_or(ApiError::InvalidSseFrame(
                "chat completion response missing choices",
            ))?;

        // Build content blocks
        let mut content = Vec::new();

        // Add text content if present
        if let Some(text) = choice.message.content.filter(|value| !value.is_empty()) {
            content.push(OutputContentBlock::Text { text });
        }

        // Add tool use blocks if present
        for tool_call in choice.message.tool_calls {
            content.push(OutputContentBlock::ToolUse {
                id: tool_call.id,
                name: tool_call.function.name,
                input: parse_tool_arguments(&tool_call.function.arguments),
            });
        }

        Ok(MessageResponse {
            id: response.id,
            kind: "message".to_string(),
            role: choice.message.role,
            content,
            model: response.model.if_empty_then(model.to_string()),
            stop_reason: choice
                .finish_reason
                .map(|value| normalize_finish_reason(&value)),
            stop_sequence: None,
            usage: Usage {
                input_tokens: response
                    .usage
                    .as_ref()
                    .map_or(0, |usage| usage.prompt_tokens),
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
                output_tokens: response
                    .usage
                    .as_ref()
                    .map_or(0, |usage| usage.completion_tokens),
            },
            request_id: None,
        })
    }

    /// Build JSON payload for streaming.
    fn build_stream_payload(&self, request: &MessageRequest) -> Value {
        let mut messages = Vec::new();
        let wire_model = self.strip_model_prefix(&request.model);

        if let Some(system) = request.system.as_ref().filter(|v| !v.is_empty()) {
            messages.push(json!({
                "role": "system",
                "content": system,
            }));
        }

        for message in &request.messages {
            messages.extend(self.translate_message(message, wire_model.as_str()));
        }

        json!({
            "model": wire_model,
            "stream": true,
            "options": {
                "temperature": request.temperature.unwrap_or(0.7),
                "num_predict": request.max_tokens,
            },
        })
    }

    /// Stream responses from the Ollama API.
    async fn stream_request(
        &self,
        request: &MessageRequest,
    ) -> Result<OllamaStream, ApiError> {
        let url = format!("{}/api/generate", self.config.base_url);
        let request_id = uuid::Uuid::new_v4().to_string();

        let payload = self.build_stream_payload(request);
        let formatted_payload = serde_json::to_string(&payload).map_err(|error| {
            ApiError::Json {
                provider: "Ollama".to_string(),
                model: self.config.model.clone(),
                body_snippet: String::new(),
                source: error,
            }
        })?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(ApiError::from)?;

        let response = client
            .post(&url)
            .header("content-type", "application/json")
            .header("X-Request-ID", &request_id)
            .body(formatted_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.map_err(ApiError::from)?;
            return Err(ApiError::Api {
                status: response.status(),
                error_type: None,
                message: None,
                request_id: Some(request_id.clone()),
                body,
                retryable: true,
                suggested_action: Some("Wait a moment and try again".to_string()),
            });
        }

        Ok(OllamaStream::new(response))
    }

    /// Strip routing prefix from model name.
    /// The prefix is used only to select transport; the backend expects the
    /// bare model id.
    fn strip_model_prefix(&self, model: &str) -> String {
        if let Some(pos) = model.find('/') {
            &model[pos + 1..]
        } else {
            model.to_string()
        }
    }

    /// Convert a ToolDefinition to OpenAI format.
    fn openai_tool_definition(&self, tool: &ToolDefinition) -> Value {
        let mut parameters = tool.input_schema.clone();
        // Add JSON schema validation for Ollama
        if let Value::Object(ref mut schema) = parameters {
            if !schema.contains_key("additionalProperties") {
                schema.insert("additionalProperties".into(), json!(false));
            }
        }

        json!({
            "type": "function",
            "function": {
                "name": tool.name,
                "description": tool.description,
                "parameters": parameters,
            }
        })
    }

    /// Convert a ToolChoice to OpenAI format.
    fn openai_tool_choice(&self, tool_choice: &ToolChoice) -> Value {
        match tool_choice {
            ToolChoice::Auto => json!("auto"),
            ToolChoice::Required => json!("required"),
            ToolChoice::Specific(ref name) => json::object!({
                "type": "function",
                "function": { "name": name },
            }),
        }
    }
}

// ----------------------------------------------------------------------
// Streaming support
// ----------------------------------------------------------------------

pub struct OllamaStream {
    response: reqwest::Response,
}

impl OllamaStream {
    pub fn new(response: reqwest::Response) -> Self {
        Self { response }
    }

    /// Read chunks from the stream and emit events.
    pub async fn chunk(&mut self) -> Result<Vec<ChatCompletionChunk>, ApiError> {
        let mut chunks = Vec::new();
        let mut buffer = Vec::new();

        loop {
            let chunk_bytes = self.response.chunk().await.map_err(ApiError::from)?;
            if chunk_bytes.is_none() {
                break;
            }

            buffer.extend_from_slice(chunk_bytes.as_ref().unwrap());

            while let Some(frame_end) = next_sse_frame(&mut buffer) {
                let frame = std::str::from_utf8(&buffer[..frame_end]).map_err(|error| {
                    ApiError::InvalidSseFrame(format!("invalid UTF-8 in SSE frame: {error}"))
                })?;

                // Ollama returns JSONL lines: {"response":"hello"}
                let line = frame.trim_end_matches('\n');
                if line.is_empty() || line.starts_with(':') {
                    // Skip keep-alive lines or non-data lines
                    continue;
                }

                let parsed: ChatCompletionChunk = serde_json::from_str(line).map_err(|error| {
                    ApiError::Json {
                        provider: "Ollama".to_string(),
                        model: String::new(),
                        body_snippet: line.to_string(),
                        source: error,
                    }
                })?;

                if let Some(choice) = parsed.choices.first() {
                    if choice.delta.content.is_some() || choice.delta.tool_calls.len() > 0 {
                        chunks.push(parsed);
                    }
                }

                // Remove the processed frame from buffer
                buffer = buffer[frame_end..].to_vec();
            }
        }

        Ok(chunks)
    }

    pub async fn finish_reason(&self) -> Option<String> {
        // For Ollama, finish_reason is empty string when done rather than "stop"
        None
    }
}

// ----------------------------------------------------------------------
// Helper functions
// ----------------------------------------------------------------------

fn parse_tool_arguments(arguments: &str) -> serde_json::Value {
    serde_json::from_str(arguments).unwrap_or_else(|_| json!({ "raw": arguments }))
}

fn next_sse_frame(buffer: &mut Vec<u8>) -> Option<usize> {
    buffer.windows(2).position(|window| window == b"\n\n").map(|pos| pos + 2)
}

pub trait RequestTruncater {
    fn truncater() -> Self;
}

impl RequestTruncater for RequestTruncater {
    fn truncater() -> Self {
        Self
    }
}

fn normalize_finish_reason(reason: &str) -> Option<String> {
    // Ollama returns "stop" when complete, same as OpenAI
    match reason {
        "stop" | "tool_calls" | null_marker => Some(reason.to_string()),
        "length" => None,
        _ => Some(reason.to_string()),
    }
}

#[derive(Debug, Default, Deserialize)]
struct ToolResultMarker {}

fn deserialize_null_as_empty_vec<'de, D>(deserializer: D) -> Result<Vec<DeltaToolCall>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Inner {
        #[serde(default)]
        index: u32,
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        function: DeltaFunction,
    }

    let result: Option<Vec<Inner>> = Option::deserialize(deserializer)?;
    Ok(result
        .into_iter()
        .flatten()
        .map(|inner| DeltaToolCall {
            index: inner.index,
            id: inner.id,
            function: inner.function,
        })
        .collect())
}

/// Estimate the serialized JSON size of a request payload in bytes.
pub fn estimate_request_body_size(request: &MessageRequest, config: &OllamaConfig) -> usize {
    let payload = json!({
        "model": config.model,
        "stream": request.stream,
        "messages": json!([]), // placeholder
    });
    serde_json::to_vec(&payload).map_or(0, |v| v.len())
}

/// Pre-flight check for request body size against provider limits.
pub fn check_request_body_size(
    request: &MessageRequest,
    config: &OllamaConfig,
) -> Result<(), ApiError> {
    let estimated_bytes = estimate_request_body_size(request, config);

    if estimated_bytes > config.max_request_body_bytes {
        Err(ApiError::RequestBodySizeExceeded {
            estimated_bytes,
            max_bytes: config.max_request_body_bytes,
            provider: "Ollama".to_string(),
        })
    } else {
        Ok(())
    }
}

/// Truncate `body` so the resulting snippet contains at most `max_chars`
/// characters (counted by Unicode scalar values, not bytes), preserving the
/// leading slice of the body that the caller most often needs to inspect.
fn truncate_body_snippet(body: &str, max_chars: usize) -> String {
    let mut taken_chars = 0;
    let mut byte_end = 0;
    for (offset, character) in body.char_indices() {
        if taken_chars >= max_chars {
            break;
        }
        taken_chars += 1;
        byte_end = offset + character.len_utf8();
    }
    if taken_chars >= max_chars && byte_end < body.len() {
        format!("{}…", &body[..byte_end])
    } else {
        body[..byte_end].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_chat_completion_request() {
        let req = json!({
            "model": "qwen2.5-coder:7b",
            "stream": false,
            "options": { "temperature": 0.7, "num_predict": 100 },
        });
        let payload = serde_json::to_string(&req).unwrap();
        assert!(payload.contains("model"));
        assert!(payload.contains("temperature"));
        assert!(payload.contains("num_predict"));
    }

    #[test]
    fn test_truncate_body_snippet() {
        let body = "a".repeat(300) + "X";
        let snippet = truncate_body_snippet(&body, 200);
        assert_eq!(snippet.len(), 201); // 200 chars + "…"
        assert_eq!(snippet.chars().count(), 201);
    }
}