mod error;
mod protocol;
mod routes;
mod state;

use axum::routing::{get, post};
use axum::{Json, Router};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use state::AppState;

fn load_env_file(key: &str) -> Option<String> {
    let contents = std::fs::read_to_string(".env").ok()?;
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                let v = v.trim();
                // Strip surrounding quotes
                let v = v.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(v);
                let v = v.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')).unwrap_or(v);
                return Some(v.to_string());
            }
        }
    }
    None
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

#[tokio::main]
async fn main() {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .or_else(|| load_env_file("ANTHROPIC_API_KEY"))
        .expect("ANTHROPIC_API_KEY not found in env or .env file");

    println!("Loaded API key ({} chars)", api_key.len());

    let state = AppState::new(api_key);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/sessions", post(routes::sessions::create_session))
        .route("/api/sessions", get(routes::sessions::list_sessions))
        .route("/api/sessions/:id", get(routes::sessions::get_session))
        .route("/ws/:session_id", get(routes::websocket::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    axum::serve(listener, app).await.expect("server error");
}
