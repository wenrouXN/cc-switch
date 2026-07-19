//! Hermes routes.

use axum::{extract::State, routing::get, Json, Router};
use serde_json::json;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/model-config", get(get_model_config))
        .route("/memory-limits", get(get_memory_limits))
        .route("/launch-dashboard", axum::routing::post(launch_dashboard))
        .route("/open-web-ui", axum::routing::post(open_web_ui))
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": data }))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({ "success": false, "error": msg.to_string() }))
}

async fn get_model_config() -> Json<serde_json::Value> {
    match crate::commands::hermes::get_hermes_model_config() {
        Ok(config) => ok(config),
        Err(e) => err(e),
    }
}

async fn get_memory_limits() -> Json<serde_json::Value> {
    match crate::commands::hermes::get_hermes_memory_limits() {
        Ok(limits) => ok(limits),
        Err(e) => err(e),
    }
}

async fn launch_dashboard() -> Json<serde_json::Value> {
    // Desktop-only (opens native window / local process).
    err("Hermes dashboard launch is not available in web/headless mode")
}

async fn open_web_ui() -> Json<serde_json::Value> {
    err("Hermes web UI open is not available in web/headless mode")
}
