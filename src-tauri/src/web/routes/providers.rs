//! Provider routes — delegate to `ProviderService` and `Database`.
//!
//! Mirrors `commands/provider.rs` for the subset the web UI needs.

use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::app_config::AppType;
use std::str::FromStr;
use crate::services::ProviderService;
use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/", get(list_providers))
        .route("/current", get(get_current))
        .route("/:id/switch", post(switch))
        .route("/:id", axum::routing::get(get_one))
}

#[derive(Deserialize)]
struct AppQuery {
    app: String,
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({"success": true, "data": data}))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({"success": false, "error": msg.to_string()}))
}

async fn list_providers(
    State((state, _)): State<Shared>,
    Query(q): Query<AppQuery>,
) -> Json<serde_json::Value> {
    let app_type = match AppType::from_str(&q.app) {
        Ok(a) => a,
        Err(e) => return err(e.to_string()),
    };
    match ProviderService::list(&state, app_type) {
        Ok(providers) => ok(providers),
        Err(e) => err(e.to_string()),
    }
}

async fn get_current(
    State((state, _)): State<Shared>,
    Query(q): Query<AppQuery>,
) -> Json<serde_json::Value> {
    let app_type = match AppType::from_str(&q.app) {
        Ok(a) => a,
        Err(e) => return err(e.to_string()),
    };
    match ProviderService::current(&state, app_type) {
        Ok(id) => ok(id),
        Err(e) => err(e.to_string()),
    }
}

async fn get_one(
    State((state, _)): State<Shared>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Query(q): Query<AppQuery>,
) -> Json<serde_json::Value> {
    match state.db.get_provider_by_id(&id, &q.app) {
        Ok(Some(provider)) => ok(provider),
        Ok(None) => err(format!("Provider not found: {}", id)),
        Err(e) => err(e.to_string()),
    }
}

async fn switch(
    State((state, _)): State<Shared>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Query(q): Query<AppQuery>,
) -> Json<serde_json::Value> {
    let app_type = match AppType::from_str(&q.app) {
        Ok(a) => a,
        Err(e) => return err(e.to_string()),
    };
    // Use ProviderService::switch which handles live config + proxy takeover + DB
    match ProviderService::switch(&state, app_type, &id) {
        Ok(_) => {
            // Also update proxy target if proxy is running
            let _ = state.proxy_service.switch_proxy_target(&q.app, &id).await;
            ok(true)
        }
        Err(e) => err(e.to_string()),
    }
}

// ---- Universal Provider routes ----

pub fn universal_routes() -> Router<Shared> {
    Router::new()
        .route("/", get(list_universal))
        .route("/:id", get(get_universal))
        .route("/", post(upsert_universal))
        .route("/:id", axum::routing::delete(delete_universal))
        .route("/:id/sync", post(sync_universal))
}

async fn list_universal(
    State((state, _)): State<Shared>,
) -> Json<serde_json::Value> {
    match state.db.get_all_universal_providers() {
        Ok(providers) => ok(providers),
        Err(e) => err(e),
    }
}

async fn get_universal(
    State((state, _)): State<Shared>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match state.db.get_universal_provider(&id) {
        Ok(provider) => ok(provider),
        Err(e) => err(e),
    }
}

async fn upsert_universal(
    State((state, _)): State<Shared>,
    Json(provider): Json<crate::provider::UniversalProvider>,
) -> Json<serde_json::Value> {
    match state.db.save_universal_provider(&provider) {
        Ok(()) => ok(true),
        Err(e) => err(e),
    }
}

async fn delete_universal(
    State((state, _)): State<Shared>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match state.db.delete_universal_provider(&id) {
        Ok(_) => ok(true),
        Err(e) => err(e),
    }
}

async fn sync_universal(
    State((state, _)): State<Shared>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match crate::services::ProviderService::sync_universal_to_apps(&state, &id) {
        Ok(_) => ok(true),
        Err(e) => err(e),
    }
}
