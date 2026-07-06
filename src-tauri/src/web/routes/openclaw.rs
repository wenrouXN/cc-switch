//! OpenClaw routes.

use axum::{routing::get, Json, Router};
use serde_json::json;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/health", get(health))
        .route("/model-catalog", get(model_catalog))
        .route("/default-model", get(default_model))
        .route("/agents-defaults", get(agents_defaults))
        .route("/env", get(get_env))
        .route("/tools", get(tools))
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": data }))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({ "success": false, "error": msg.to_string() }))
}

async fn health() -> Json<serde_json::Value> {
    match crate::commands::openclaw::scan_openclaw_config_health() {
        Ok(warnings) => ok(warnings),
        Err(e) => err(e),
    }
}

async fn model_catalog() -> Json<serde_json::Value> {
    match crate::commands::openclaw::get_openclaw_model_catalog() {
        Ok(catalog) => ok(catalog),
        Err(e) => err(e),
    }
}

async fn default_model() -> Json<serde_json::Value> {
    match crate::commands::openclaw::get_openclaw_default_model() {
        Ok(model) => ok(model),
        Err(e) => err(e),
    }
}

async fn agents_defaults() -> Json<serde_json::Value> {
    match crate::commands::openclaw::get_openclaw_agents_defaults() {
        Ok(defaults) => ok(defaults),
        Err(e) => err(e),
    }
}

async fn get_env() -> Json<serde_json::Value> {
    match crate::commands::openclaw::get_openclaw_env() {
        Ok(env) => ok(env),
        Err(e) => err(e),
    }
}

async fn tools() -> Json<serde_json::Value> {
    match crate::commands::openclaw::get_openclaw_tools() {
        Ok(t) => ok(t),
        Err(e) => err(e),
    }
}
