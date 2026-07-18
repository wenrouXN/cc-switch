//! Prompts routes — delegate to Database DAO.

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/", get(get_prompts))
        .route("/", post(upsert_prompt))
        .route("/:id", post(delete_prompt))
        .route("/import", post(import_prompt))
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": data }))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({ "success": false, "error": msg.to_string() }))
}

#[derive(serde::Deserialize)]
struct AppQuery {
    #[serde(default = "default_app")]
    app: String,
}
fn default_app() -> String {
    "claude".into()
}

async fn get_prompts(
    State((state, _)): State<Shared>,
    axum::extract::Query(q): axum::extract::Query<AppQuery>,
) -> Json<serde_json::Value> {
    match state.db.get_prompts(&q.app) {
        Ok(prompts) => ok(prompts),
        Err(e) => err(e),
    }
}

async fn upsert_prompt(
    State((state, _)): State<Shared>,
    Json(mut body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let app = body
        .get("app")
        .and_then(|v| v.as_str())
        .unwrap_or("claude")
        .to_string();
    let prompt_val = body
        .as_object_mut()
        .and_then(|m| m.remove("prompt"))
        .unwrap_or(body.clone());
    match serde_json::from_value::<crate::prompt::Prompt>(prompt_val) {
        Ok(p) => match state.db.save_prompt(&app, &p) {
            Ok(()) => ok(true),
            Err(e) => err(e),
        },
        Err(e) => err(format!("Invalid prompt: {}", e)),
    }
}

async fn delete_prompt(
    State((state, _)): State<Shared>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match state.db.delete_prompt("claude", &id) {
        Ok(()) => ok(true),
        Err(e) => err(e),
    }
}

async fn import_prompt() -> Json<serde_json::Value> {
    ok(0)
}
