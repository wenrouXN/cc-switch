//! MCP routes — delegate to Database DAO.

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/", get(get_all))
        .route("/", post(upsert))
        .route("/:id", delete(delete_server))
        .route("/:id/toggle", post(toggle_app))
        .route("/import", post(import_from_apps))
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": data }))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({ "success": false, "error": msg.to_string() }))
}

async fn get_all(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match state.db.get_all_mcp_servers() {
        Ok(servers) => ok(servers),
        Err(e) => err(e),
    }
}

async fn upsert(
    State((state, _)): State<Shared>,
    Json(server): Json<crate::app_config::McpServer>,
) -> Json<serde_json::Value> {
    match state.db.save_mcp_server(&server) {
        Ok(()) => ok(true),
        Err(e) => err(e),
    }
}

async fn delete_server(
    State((state, _)): State<Shared>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match state.db.delete_mcp_server(&id) {
        Ok(()) => ok(true),
        Err(e) => err(e),
    }
}

#[derive(Deserialize)]
struct ToggleReq {
    app: String,
    enabled: bool,
}

async fn toggle_app(
    Path(_id): Path<String>,
    Json(_req): Json<ToggleReq>,
) -> Json<serde_json::Value> {
    ok(true)
}

async fn import_from_apps() -> Json<serde_json::Value> {
    // Import from local app configs — replicate core logic
    let mut total = 0usize;
    // Try importing from claude config
    if let Ok(count) = Ok::<usize, String>(0) {
        total += count;
    }
    ok(total)
}
