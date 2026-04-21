// INTEGRATION GUIDE: Adding Ollama to crates/api/src/providers/mod.rs
//
// This shows the changes needed to integrate the ollama_adapter module.

// ============================================================================
// STEP 1: Add module declaration (after existing modules)
// ============================================================================

pub mod ollama_adapter;  // Add this line after other provider modules

// ============================================================================
// STEP 2: Add Ollama to ProviderKind enum
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Anthropic,
    Xai,
    OpenAi,
    Ollama,  // ← ADD THIS
}

// ============================================================================
// STEP 3: Add Ollama configuration to MODEL_REGISTRY
// ============================================================================

// In the MODEL_REGISTRY const, add entries for Ollama models:

const MODEL_REGISTRY: &[(&str, ProviderMetadata)] = &[
    // ... existing entries ...
    
    // Ollama models (local or remote)
    (
        "ollama/llama2",
        ProviderMetadata {
            provider: ProviderKind::Ollama,
            auth_env: "OLLAMA_API_KEY",
            base_url_env: "OLLAMA_BASE_URL",
            default_base_url: "http://localhost:11434/v1",
        },
    ),
    (
        "ollama/llama3",
        ProviderMetadata {
            provider: ProviderKind::Ollama,
            auth_env: "OLLAMA_API_KEY",
            base_url_env: "OLLAMA_BASE_URL",
            default_base_url: "http://localhost:11434/v1",
        },
    ),
    (
        "ollama/glm-4.7-flash",
        ProviderMetadata {
            provider: ProviderKind::Ollama,
            auth_env: "OLLAMA_API_KEY",
            base_url_env: "OLLAMA_BASE_URL",
            default_base_url: "http://localhost:11434/v1",
        },
    ),
    (
        "ollama/qwen2.5-coder",
        ProviderMetadata {
            provider: ProviderKind::Ollama,
            auth_env: "OLLAMA_API_KEY",
            base_url_env: "OLLAMA_BASE_URL",
            default_base_url: "http://localhost:11434/v1",
        },
    ),
];

// ============================================================================
// STEP 4: Add Ollama to resolve_provider function
// ============================================================================

// In resolve_provider(), add a match arm for Ollama:

pub async fn resolve_provider(
    kind: ProviderKind,
    auth: &AuthSource,
) -> Result<Box<dyn Provider>, ApiError> {
    match kind {
        ProviderKind::Anthropic => {
            // ... existing code ...
        }
        ProviderKind::Xai => {
            // ... existing code ...
        }
        ProviderKind::OpenAi => {
            // ... existing code ...
        }
        ProviderKind::Ollama => {
            // Create Ollama client with compatibility layer
            let config = ollama_adapter::OllamaCompatConfig::ollama();
            let base_url = std::env::var(config.base_url_env)
                .unwrap_or_else(|_| config.default_base_url.to_string());
            let api_key = auth.get_key_for_env(config.api_key_env)?;
            
            Ok(Box::new(openai_compat::OpenAiCompatProvider::new(
                config.provider_name,
                &base_url,
                &api_key,
                config.max_request_body_bytes,
            )))
        }
    }
}

// ============================================================================
// STEP 5: Add Ollama error handling to error mapping
// ============================================================================

// In preflight_message_request(), add a check for Ollama:

pub fn preflight_message_request(
    request: &MessageRequest,
    model_metadata: &ModelTokenLimit,
) -> Result<(), ApiError> {
    // ... existing validation ...
    
    // For Ollama, be more lenient with context window checks
    // (Ollama may have different token counting)
    if request.model.starts_with("ollama/") {
        // Use 90% of context window as safety margin for Ollama
        let safety_margin = (model_metadata.context_window_tokens as f32 * 0.9) as u32;
        if estimated_tokens > safety_margin {
            return Err(ApiError::ContextWindowExceeded {
                model: request.model.clone(),
                estimated_input_tokens: input_tokens,
                requested_output_tokens: request.max_tokens,
                estimated_total_tokens: estimated_tokens,
                context_window_tokens: model_metadata.context_window_tokens,
            });
        }
    } else {
        // Use stricter checks for other providers
        if estimated_tokens > model_metadata.context_window_tokens {
            return Err(ApiError::ContextWindowExceeded {
                model: request.model.clone(),
                estimated_input_tokens: input_tokens,
                requested_output_tokens: request.max_tokens,
                estimated_total_tokens: estimated_tokens,
                context_window_tokens: model_metadata.context_window_tokens,
            });
        }
    }
    
    Ok(())
}

// ============================================================================
// STEP 6: Update detect_provider_kind to handle ollama/
// ============================================================================

pub fn detect_provider_kind(model: &str) -> ProviderKind {
    if model.starts_with("ollama/") {
        return ProviderKind::Ollama;
    }
    if model.starts_with("openai/") {
        return ProviderKind::OpenAi;
    }
    // ... rest of detection logic ...
}

// ============================================================================
// STEP 7: Add Ollama-specific configuration in runtime/src/lib.rs
// ============================================================================

// Add these environment variables to the config loader:

pub const OLLAMA_BASE_URL_ENV: &str = "OLLAMA_BASE_URL";
pub const OLLAMA_API_KEY_ENV: &str = "OLLAMA_API_KEY";
pub const DEFAULT_OLLAMA_BASE_URL: &str = "http://localhost:11434/v1";

// ============================================================================
// STEP 8: Update .claw.json example in documentation
// ============================================================================

// Add to README or USAGE.md:

{
  "model": "openai/glm-4.7-flash",
  "providers": {
    "ollama": {
      "base_url": "http://192.165.134.28:32780/v1",
      "api_key": "dummy"
    }
  }
}

// Or use environment variables:
// export OLLAMA_BASE_URL="http://192.165.134.28:32780/v1"
// export OLLAMA_API_KEY="dummy"
// claw --model ollama/glm-4.7-flash

// ============================================================================
// COMPLETE USAGE EXAMPLES
// ============================================================================

// Example 1: Local Ollama (default)
// export OLLAMA_API_KEY="dummy"
// claw --model ollama/llama2 prompt "hello"

// Example 2: Remote Ollama (vast.ai)
// export OLLAMA_BASE_URL="http://192.165.134.28:32780/v1"
// export OLLAMA_API_KEY="dummy"
// claw --model ollama/glm-4.7-flash

// Example 3: Interactive mode
// export OLLAMA_BASE_URL="http://192.165.134.28:32780/v1"
// export OLLAMA_API_KEY="dummy"
// claw --model ollama/glm-4.7-flash
