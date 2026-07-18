//! Sessions routes.

use axum::{extract::State, routing::get, Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/", get(list_sessions))
        .route("/messages", get(get_messages))
        .route("/delete", axum::routing::post(delete_session))
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": data }))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({ "success": false, "error": msg.to_string() }))
}

async fn list_sessions() -> Json<serde_json::Value> {
    match crate::commands::session_manager::list_sessions().await {
        Ok(sessions) => ok(sessions),
        Err(e) => err(e),
    }
}

async fn get_messages(
    axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let provider_id = q.get("providerId").cloned().unwrap_or_default();
    let source_path = q.get("sourcePath").cloned().unwrap_or_default();
    match crate::commands::session_manager::get_session_messages(provider_id, source_path).await {
        Ok(msgs) => ok(msgs),
        Err(e) => err(e),
    }
}

#[derive(Deserialize)]
struct DeleteReq {
    #[serde(rename = "providerId")]
    provider_id: String,
    #[serde(rename = "sessionId", default)]
    session_id: Option<String>,
    #[serde(rename = "sourcePath", default)]
    source_path: Option<String>,
}

async fn delete_session(Json(req): Json<DeleteReq>) -> Json<serde_json::Value> {
    match crate::commands::session_manager::delete_session(
        req.provider_id,
        req.session_id.unwrap_or_default(),
        req.source_path.unwrap_or_default(),
    )
    .await
    {
        Ok(_) => ok(true),
        Err(e) => err(e),
    }
}
