//! Backend configuration for AI models

use crate::{mcp::Message, Result};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Backend {
    /// OpenAI-compatible API
    OpenAI {
        /// API endpoint URL
        base_url: String,

        /// API key
        api_key: Option<String>,

        /// Model name
        model: String,

        /// Temperature (0-2)
        temperature: f32,
    },

    /// Ollama local API
    Ollama {
        /// Ollama base URL (default: http://localhost:11434)
        base_url: Option<String>,

        /// Model name
        model: String,

        /// Timeout (seconds)
        timeout: u64,
    },
}

impl Default for Backend {
    fn default() -> Self {
        Self::Ollama {
            base_url: None,
            model: crate::DEFAULT_MODEL.to_string(),
            timeout: 60,
        }
    }
}

impl Backend {
    /// Get the model name
    pub fn model(&self) -> &str {
        match self {
            Backend::OpenAI { model, .. } => model,
            Backend::Ollama { model, .. } => model,
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        match self {
            Backend::OpenAI { base_url, .. } => base_url,
            Backend::Ollama {
                base_url: Some(url),
                ..
            } => url,
            Backend::Ollama { .. } => "http://localhost:11434",
        }
    }

    /// Check if API key is configured
    pub fn has_api_key(&self) -> bool {
        match self {
            Backend::OpenAI { api_key, .. } => api_key.is_some(),
            Backend::Ollama { .. } => true,
        }
    }

    /// Validate the backend configuration
    pub fn validate(&self) -> Result<()> {
        match self {
            Backend::OpenAI { base_url, api_key, model, .. } => {
                if base_url.is_empty() {
                    return Err(crate::McpCliError::validation("base_url is required"));
                }
                if model.is_empty() {
                    return Err(crate::McpCliError::validation("model is required"));
                }
                if let Some(key) = api_key {
                    if key.is_empty() {
                        return Err(crate::McpCliError::validation("api_key is required"));
                    }
                }
            }
            Backend::Ollama { model, .. } => {
                if model.is_empty() {
                    return Err(crate::McpCliError::validation("model is required"));
                }
            }
        }

        Ok(())
    }
}

/// Backend client for making API calls
pub struct BackendClient {
    backend: Backend,
    http_client: reqwest::Client,
}

impl BackendClient {
    /// Create a new backend client
    pub fn new(backend: Backend) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(backend.get_timeout()))
            .build()
            .unwrap();

        Self {
            backend,
            http_client,
        }
    }

    /// Send a chat completion request
    pub async fn chat_completion(
        &self,
        messages: Vec<Message>,
    ) -> Result<String> {
        self.backend.validate()?;

        match &self.backend {
            Backend::OpenAI { base_url, api_key, model, temperature } => {
                self.chat_completion_openai(
                    base_url,
                    api_key.as_ref().map(|s| s.as_str()),
                    model,
                    temperature,
                    messages,
                )
                .await
            }
            Backend::Ollama {
                base_url,
                model,
                timeout,
            } => {
                self.chat_completion_ollama(
                    base_url.as_deref().unwrap_or("http://localhost:11434"),
                    model,
                    *timeout,
                    messages,
                )
                .await
            }
        }
    }

    /// Chat completion for OpenAI-compatible API
    async fn chat_completion_openai(
        &self,
        base_url: &str,
        api_key: Option<&str>,
        model: &str,
        temperature: f32,
        messages: Vec<Message>,
    ) -> Result<String> {
        // Build the messages array
        let api_messages: Vec<serde_json::Value> = messages
            .into_iter()
            .map(|msg| {
                let role = match msg.role {
                    mcp::MessageRole::User => "user",
                    mcp::MessageRole::Assistant => "assistant",
                    mcp::MessageRole::System => "system",
                };
                serde_json::json!({
                    "role": role,
                    "content": msg.content
                })
            })
            .collect();

        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        let client = reqwest::Client::new();
        let mut request = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": model,
                "messages": api_messages,
                "temperature": temperature
            }));

        if let Some(key) = api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request.send().await.map_err(|e| {
            crate::McpCliError::validation(format!("Request failed: {}", e))
        })?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            crate::McpCliError::validation(format!("Failed to read response: {}", e))
        })?;

        if !status.is_success() {
            return Err(crate::McpCliError::validation(format!(
                "API error: {} - {}",
                status,
                body
            )));
        }

        // Parse response and extract the assistant message
        let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            crate::McpCliError::validation(format!("Failed to parse response: {}", e))
        })?;

        let choices = json
            .get("choices")
            .and_then(|c| c.as_array())
            .ok_or_else(|| {
                crate::McpCliError::validation("No choices in response".to_string())
            })?;

        // Return the assistant's content
        let text = choices
            .get(0)
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .or_else(|| {
                // Fallback to reasoning content if available
                choices
                    .get(0)
                    .and_then(|c| c.get("message"))
                    .and_then(|m| m.get("reasoning_content"))
                    .and_then(|c| c.as_str())
            })
            .ok_or_else(|| {
                crate::McpCliError::validation("No content in response".to_string())
            })?;

        Ok(text.to_string())
    }

    /// Chat completion for Ollama API
    async fn chat_completion_ollama(
        &self,
        base_url: &str,
        model: &str,
        timeout: u64,
        messages: Vec<Message>,
    ) -> Result<String> {
        let url = format!("{}/api/chat", base_url.trim_end_matches('/'));

        // Build the messages array
        let api_messages: Vec<serde_json::Value> = messages
            .into_iter()
            .map(|msg| {
                let role = match msg.role {
                    mcp::MessageRole::User => "user",
                    mcp::MessageRole::Assistant => "assistant",
                    mcp::MessageRole::System => "system",
                };
                serde_json::json!({
                    "role": role,
                    "content": msg.content
                })
            })
            .collect();

        let mut response = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": model,
                "messages": api_messages,
                "stream": false,
                "options": {
                    "num_predict": timeout
                }
            }))
            .send()
            .await
            .map_err(|e| {
                crate::McpCliError::validation(format!("Request failed: {}", e))
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            crate::McpCliError::validation(format!("Failed to read response: {}", e))
        })?;

        if !status.is_success() {
            return Err(crate::McpCliError::validation(format!(
                "API error: {} - {}",
                status,
                body
            )));
        }

        // Parse response and extract the assistant message
        let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            crate::McpCliError::validation(format!("Failed to parse response: {}", e))
        })?;

        let message = json
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                crate::McpCliError::validation("No content in response".to_string())
            })?;

        Ok(message.to_string())
    }

    /// Get timeout in seconds
    fn get_timeout(&self) -> u64 {
        match &self.backend {
            Backend::Ollama { timeout, .. } => *timeout,
            _ => 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_default() {
        let be = Backend::default();
        assert_eq!(be.model(), crate::DEFAULT_MODEL);
    }

    #[test]
    fn test_openai_validation() {
        let be = Backend::OpenAI {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: Some("sk-test".to_string()),
            model: "gpt-4".to_string(),
            temperature: 0.7,
        };
        assert!(be.validate().is_ok());
    }

    #[test]
    fn test_ollama_validation() {
        let be = Backend::Ollama {
            base_url: Some("http://localhost:11434".to_string()),
            model: "llama2".to_string(),
            timeout: 60,
        };
        assert!(be.validate().is_ok());
    }

    #[tokio::test]
    async fn test_backend_client_creation() {
        let be = Backend::Ollama {
            base_url: Some("http://localhost:11434".to_string()),
            model: "llama2".to_string(),
            timeout: 60,
        };
        let client = BackendClient::new(be);
        assert_eq!(client.backend.model(), "llama2");
    }
}
}