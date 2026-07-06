//! Auth routes — no password required.

use axum::{routing::post, Json, Router};
use serde_json::json;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/login", post(login))
        .route("/change-password", post(change_password))
}

async fn login() -> Json<serde_json::Value> {
    Json(json!({"success": true, "data": {"token": "no-auth", "must_change": false}}))
}

async fn change_password() -> Json<serde_json::Value> {
    Json(json!({"success": true}))
}
