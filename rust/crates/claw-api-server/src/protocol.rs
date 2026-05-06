use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    TextDelta {
        text: String,
    },
    ToolCallStarted {
        id: String,
        name: String,
        input: String,
    },
    ToolCallDone {
        id: String,
        output: String,
        is_error: bool,
    },
    PermissionRequest {
        request_id: String,
        tool_name: String,
        input: String,
    },
    TurnDone {
        input_tokens: u32,
        output_tokens: u32,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    UserTurn {
        text: String,
    },
    PermissionResponse {
        request_id: String,
        decision: String,
    },
    Cancel,
}
