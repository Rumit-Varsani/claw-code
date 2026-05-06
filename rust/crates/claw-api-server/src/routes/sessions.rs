use axum::extract::{Path, State};
use axum::Json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;
use crate::state::{AppState, Session};

pub async fn create_session(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let id = uuid::Uuid::new_v4().to_string();
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards");
    let created_at_ms = dur.as_secs() * 1000 + u64::from(dur.subsec_millis());

    let session = Session {
        id: id.clone(),
        messages: Vec::new(),
        created_at_ms,
    };

    state.sessions.lock().await.insert(id.clone(), session);

    Json(serde_json::json!({
        "session_id": id,
        "created_at_ms": created_at_ms,
    }))
}

pub async fn list_sessions(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let sessions = state.sessions.lock().await;
    let mut list: Vec<serde_json::Value> = sessions
        .values()
        .map(|s| {
            serde_json::json!({
                "session_id": s.id,
                "created_at_ms": s.created_at_ms,
                "message_count": s.messages.len(),
            })
        })
        .collect();

    list.sort_by(|a, b| {
        let a_ts = a["created_at_ms"].as_u64().unwrap_or(0);
        let b_ts = b["created_at_ms"].as_u64().unwrap_or(0);
        b_ts.cmp(&a_ts)
    });

    Json(serde_json::Value::Array(list))
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let sessions = state.sessions.lock().await;
    let session = sessions
        .get(&id)
        .ok_or_else(|| AppError::NotFound(format!("session {id} not found")))?;

    Ok(Json(serde_json::json!({
        "session_id": session.id,
        "messages": session.messages,
    })))
}
