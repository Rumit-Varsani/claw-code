// Protocol types — mirrors rust/crates/claw-api-server/src/protocol.rs exactly.
// The Rust side uses `#[serde(tag = "type", rename_all = "snake_case")]`.

// ─── Server → Client messages ─────────────────────────────────────────────

export interface TextDelta {
  type: "text_delta";
  text: string;
}

export interface ToolCallStarted {
  type: "tool_call_started";
  id: string;
  name: string;
  input: string;
}

export interface ToolCallDone {
  type: "tool_call_done";
  id: string;
  output: string;
  is_error: boolean;
}

export interface PermissionRequest {
  type: "permission_request";
  request_id: string;
  tool_name: string;
  input: string;
}

export interface TurnDone {
  type: "turn_done";
  input_tokens: number;
  output_tokens: number;
}

export interface ServerError {
  type: "error";
  message: string;
}

export type ServerMsg =
  | TextDelta
  | ToolCallStarted
  | ToolCallDone
  | PermissionRequest
  | TurnDone
  | ServerError;

// ─── Client → Server messages ─────────────────────────────────────────────

export interface UserTurn {
  type: "user_turn";
  text: string;
}

export interface PermissionResponse {
  type: "permission_response";
  request_id: string;
  decision: string; // "allow" | "deny"
}

export interface Cancel {
  type: "cancel";
}

export type ClientMsg = UserTurn | PermissionResponse | Cancel;

// ─── REST API shapes ──────────────────────────────────────────────────────

export interface SessionSummary {
  session_id: string;
  created_at_ms: number;
  message_count: number;
}

export interface SessionDetail {
  session_id: string;
  messages: ChatMessage[];
}

export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

// ─── UI-level message (extends ChatMessage with streaming state) ──────────

export interface UIMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  streaming: boolean;
  timestamp: number;
}
