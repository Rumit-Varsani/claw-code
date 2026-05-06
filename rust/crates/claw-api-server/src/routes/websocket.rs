use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use futures::StreamExt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::protocol::{ClientMsg, ServerMsg};
use crate::state::{AppState, ChatMessage, Session};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, session_id, state))
}

async fn send_msg(socket: &mut WebSocket, msg: &ServerMsg) -> bool {
    match serde_json::to_string(msg) {
        Ok(json) => socket.send(Message::Text(json)).await.is_ok(),
        Err(_) => false,
    }
}

fn now_millis() -> u64 {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards");
    dur.as_secs() * 1000 + u64::from(dur.subsec_millis())
}

async fn handle_socket(mut socket: WebSocket, session_id: String, state: AppState) {
    while let Some(Ok(frame)) = socket.recv().await {
        let text = match frame {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let client_msg: ClientMsg = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(_) => break,
        };

        match client_msg {
            ClientMsg::Cancel => break,
            ClientMsg::PermissionResponse { .. } => {
                // Not used in v1
            }
            ClientMsg::UserTurn { text } => {
                handle_user_turn(&mut socket, &session_id, &state, text).await;
            }
        }
    }
}

async fn handle_user_turn(
    socket: &mut WebSocket,
    session_id: &str,
    state: &AppState,
    text: String,
) {
    // 1. Store user message and get messages snapshot
    let messages = {
        let mut sessions = state.sessions.lock().await;
        let session = sessions
            .entry(session_id.to_string())
            .or_insert_with(|| Session {
                id: session_id.to_string(),
                messages: Vec::new(),
                created_at_ms: now_millis(),
            });
        session.messages.push(ChatMessage {
            role: "user".to_string(),
            content: text,
        });
        session.messages.clone()
    };

    // 2. Build request body
    let api_messages: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content,
            })
        })
        .collect();

    let body = serde_json::json!({
        "model": state.model,
        "max_tokens": 8192,
        "stream": true,
        "messages": api_messages,
    });

    // 3. POST to Anthropic
    let client = reqwest::Client::new();
    let resp = match client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &state.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(body.to_string())
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            let _ = send_msg(
                socket,
                &ServerMsg::Error {
                    message: format!("request failed: {e}"),
                },
            )
            .await;
            return;
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        let _ = send_msg(
            socket,
            &ServerMsg::Error {
                message: format!("Anthropic API {status}: {body_text}"),
            },
        )
        .await;
        return;
    }

    // 4. Stream SSE response
    let full_text = stream_sse_response(socket, resp).await;

    // 5. Store assistant response
    {
        let mut sessions = state.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: full_text,
            });
        }
    }
}

async fn stream_sse_response(socket: &mut WebSocket, resp: reqwest::Response) -> String {
    let mut full_text = String::new();
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let Ok(chunk) = chunk_result else { break };

        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete lines
        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim().to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if !line.starts_with("data: ") {
                continue;
            }

            let data = &line[6..];
            if data == "[DONE]" {
                continue;
            }

            let event: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let event_type = event["type"].as_str().unwrap_or("");

            if event_type == "content_block_delta" {
                if let Some("text_delta") = event["delta"]["type"].as_str() {
                    if let Some(t) = event["delta"]["text"].as_str() {
                        full_text.push_str(t);
                        if !send_msg(
                            socket,
                            &ServerMsg::TextDelta {
                                text: t.to_string(),
                            },
                        )
                        .await
                        {
                            return full_text;
                        }
                    }
                }
            } else if event_type == "message_stop" {
                let _ = send_msg(
                    socket,
                    &ServerMsg::TurnDone {
                        input_tokens: 0,
                        output_tokens: 0,
                    },
                )
                .await;
                return full_text;
            }
        }
    }

    full_text
}
