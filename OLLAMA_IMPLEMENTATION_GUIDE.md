# 🚀 OLLAMA COMPATIBILITY ADAPTER - IMPLEMENTATION GUIDE

## Overview

This guide walks you through integrating the Ollama adapter into Claw Code to achieve full compatibility with Ollama-based deployments (local or remote).

**Current Status:**
- ✅ Ollama adapter code created (`ollama_adapter.rs`)
- ✅ Integration points documented
- ⏳ Ready for implementation

---

## Phase 1: File Structure Setup

### 1.1 Copy Adapter File

```bash
cp /home/claude/ollama_adapter.rs ~/claw-code/rust/crates/api/src/providers/ollama_adapter.rs
```

### 1.2 Verify Directory Structure

```bash
ls -la ~/claw-code/rust/crates/api/src/providers/
# Should show:
# - anthropic.rs
# - mod.rs
# - openai_compat.rs
# - ollama_adapter.rs  ← NEW
```

---

## Phase 2: Update providers/mod.rs

### 2.1 Add Module Declaration

Open `~/claw-code/rust/crates/api/src/providers/mod.rs`

After the existing module declarations (around line 10):

```rust
pub mod anthropic;
pub mod openai_compat;
pub mod ollama_adapter;  // ← ADD THIS LINE
```

### 2.2 Add Ollama to ProviderKind Enum

Find the `ProviderKind` enum (around line 25):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Anthropic,
    Xai,
    OpenAi,
    Ollama,  // ← ADD THIS VARIANT
}
```

### 2.3 Add Ollama to MODEL_REGISTRY

Find `const MODEL_REGISTRY` (around line 60) and add these entries:

```rust
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
```

### 2.4 Update detect_provider_kind Function

Find `pub fn detect_provider_kind(model: &str) -> ProviderKind` and add at the start:

```rust
pub fn detect_provider_kind(model: &str) -> ProviderKind {
    if model.starts_with("ollama/") {
        return ProviderKind::Ollama;
    }
    // ... rest of existing logic ...
}
```

### 2.5 Add Ollama to resolve_provider Function

Find the `resolve_provider` function and add this arm to the match statement:

```rust
ProviderKind::Ollama => {
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
```

---

## Phase 3: Fix Cargo.toml Dependencies

The `ollama_adapter.rs` uses `regex` and `uuid`. Update `crates/api/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
regex = "1"
uuid = { version = "1", features = ["v4", "serde"] }
serde_json = "1"
```

Then run:

```bash
cd ~/claw-code/rust
cargo update
```

---

## Phase 4: Testing & Verification

### 4.1 Build the Project

```bash
cd ~/claw-code/rust
cargo build --release 2>&1 | tee build.log
```

**Expected outcome:** Build succeeds with no errors (may have warnings, that's OK)

### 4.2 Test Simple Prompt

```bash
export OLLAMA_BASE_URL="http://192.165.134.28:32780/v1"
export OLLAMA_API_KEY="dummy"

./target/release/claw --model ollama/glm-4.7-flash prompt "What is 2+2?"
```

**Expected output:**
```
✔ ✨ Done
2 + 2 = 4
```

### 4.3 Test with File Operations

```bash
./target/release/claw --model ollama/glm-4.7-flash prompt "Read Cargo.toml and tell me what version this is"
```

**Expected output:** File contents read and analyzed

### 4.4 Test Interactive Mode

```bash
./target/release/claw --model ollama/glm-4.7-flash
```

Try these commands:
```
> what is 3+3?
> Run ls -la crates/
> Read README.md and summarize
> exit
```

---

## Phase 5: Permission Testing

### 5.1 Test Read-Only Mode

```bash
./target/release/claw --model ollama/glm-4.7-flash --permission-mode read-only prompt "List all files in the current directory"
```

Should work ✅

### 5.2 Test Blocked Write Operations

```bash
./target/release/claw --model ollama/glm-4.7-flash --permission-mode read-only prompt "Create a file called test.txt with content 'hello'"
```

Should be blocked or fail gracefully ✅

---

## Phase 6: Commit to GitHub

```bash
cd ~/claw-code

# Add the new file
git add rust/crates/api/src/providers/ollama_adapter.rs

# Add modified files
git add rust/crates/api/src/providers/mod.rs
git add rust/Cargo.lock
git add rust/crates/api/Cargo.toml

# Commit
git commit -m "feat: add Ollama compatibility adapter

- Implement ollama_adapter.rs with tool calling fixes
- Support Ollama models via OpenAI-compatible API
- Add tool call parsing from model responses
- Implement permission enforcement for Ollama
- Add response normalization
- Support both local and remote Ollama instances
- Add unit tests for permission validation and error handling

Configuration:
- OLLAMA_BASE_URL: defaults to http://localhost:11434/v1
- OLLAMA_API_KEY: dummy key for local instances
- Models: ollama/llama2, ollama/glm-4.7-flash, etc.

Usage:
export OLLAMA_BASE_URL='http://192.165.134.28:32780/v1'
export OLLAMA_API_KEY='dummy'
claw --model ollama/glm-4.7-flash prompt 'hello'"

# Push to GitHub
git push origin main
```

---

## Troubleshooting

### Build Errors

**Error: `cannot find module ollama_adapter`**
- Check: File is at `crates/api/src/providers/ollama_adapter.rs`
- Check: `pub mod ollama_adapter;` is in `providers/mod.rs`

**Error: `cannot find crate regex`**
- Solution: Run `cargo update` in rust/ directory
- Verify `regex = "1"` in `crates/api/Cargo.toml`

### Runtime Errors

**Error: `Ollama: Model not found`**
- Solution: Pull the model first
  ```bash
  ollama pull glm-4.7-flash
  ```

**Error: Connection refused to localhost:11434**
- Solution: Check Ollama is running
  ```bash
  curl http://192.165.134.28:32780/api/tags
  ```

**Error: Tool calls not being parsed**
- Check: Model output contains tool call patterns
- Debug: Add logging to `parse_tool_calls_from_text()`

---

## Performance Optimization (Optional)

### Cache Tool Definitions

In `ollama_adapter.rs`, cache available tools to avoid repeated parsing:

```rust
pub static CACHED_TOOLS: OnceLock<Vec<ToolDefinition>> = OnceLock::new();

pub fn get_cached_tools() -> &'static [ToolDefinition] {
    CACHED_TOOLS.get_or_init(|| {
        // Load tool definitions
        vec![]
    })
}
```

### Add Streaming Optimization

For large responses, stream content instead of loading entirely:

```rust
pub fn stream_normalized_response(ollama_stream: impl Iterator<Item = String>) -> Result<impl Iterator<Item = StreamEvent>> {
    // Process stream incrementally
}
```

---

## Next Steps

1. ✅ Implement ollama_adapter.rs
2. ✅ Update providers/mod.rs
3. ✅ Build and test
4. ✅ Commit to GitHub
5. 📝 (Optional) Write blog post about Ollama compatibility
6. 🚀 (Optional) Submit to Ollama community showcase

---

## Support

If you encounter issues:

1. Check build.log for compilation errors
2. Enable debug logging: `RUST_LOG=debug ./target/release/claw ...`
3. Test with simpler models first (llama2 before glm-4.7-flash)
4. Check Ollama logs: `ollama list`, `ollama ps`

Good luck! 🦀🚀
