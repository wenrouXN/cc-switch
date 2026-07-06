//! Failover routes — delegate to `Database` DAO methods.
//!
//! Mirrors `commands/failover.rs` exactly.

use axum::{extract::State, routing::post, Json, Router};
use std::str::FromStr;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/queue", axum::routing::get(get_queue))
        .route("/queue", post(add_to_queue))
        .route("/queue/remove", post(remove_from_queue))
        .route("/available", axum::routing::get(get_available))
        .route("/enabled", axum::routing::get(get_enabled))
        .route("/enabled", post(set_enabled))
        .route("/health/:provider_id", axum::routing::get(get_provider_health))
        .route("/health", post(reset_circuit_breaker))
        .route("/circuit-breaker/config", axum::routing::get(get_cb_config))
        .route("/circuit-breaker/config", post(update_cb_config))
}

#[derive(Deserialize)]
struct AppRequest {
    app: String,
}

#[derive(Deserialize)]
struct QueueItemRequest {
    #[serde(rename = "appType")]
    app: String,
    #[serde(rename = "providerId")]
    provider_id: String,
}

#[derive(Deserialize)]
struct SetEnabledRequest {
    #[serde(rename = "appType")]
    app: String,
    enabled: bool,
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({"success": true, "data": data}))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({"success": false, "error": msg.to_string()}))
}

async fn get_queue(
    State((state, _)): State<Shared>,
    axum::extract::Query(req): axum::extract::Query<AppRequest>,
) -> Json<serde_json::Value> {
    match state.db.get_failover_queue(&req.app) {
        Ok(queue) => ok(queue),
        Err(e) => err(e.to_string()),
    }
}

async fn get_available(
    State((state, _)): State<Shared>,
    axum::extract::Query(req): axum::extract::Query<AppRequest>,
) -> Json<serde_json::Value> {
    match state.db.get_available_providers_for_failover(&req.app) {
        Ok(providers) => ok(providers),
        Err(e) => err(e.to_string()),
    }
}

async fn add_to_queue(
    State((state, _)): State<Shared>,
    Json(req): Json<QueueItemRequest>,
) -> Json<serde_json::Value> {
    match state
        .db
        .add_to_failover_queue(&req.app, &req.provider_id)
    {
        Ok(()) => ok(true),
        Err(e) => err(e.to_string()),
    }
}

async fn remove_from_queue(
    State((state, _)): State<Shared>,
    Json(req): Json<QueueItemRequest>,
) -> Json<serde_json::Value> {
    match state
        .db
        .remove_from_failover_queue(&req.app, &req.provider_id)
    {
        Ok(()) => ok(true),
        Err(e) => err(e.to_string()),
    }
}

async fn get_enabled(
    State((state, _)): State<Shared>,
    axum::extract::Query(req): axum::extract::Query<AppRequest>,
) -> Json<serde_json::Value> {
    match state.db.get_proxy_config_for_app(&req.app).await {
        Ok(config) => ok(config.auto_failover_enabled),
        Err(e) => err(e.to_string()),
    }
}

/// Set auto-failover enabled.
/// Mirrors `commands/failover::set_auto_failover_enabled`:
/// - Requires takeover enabled first
/// - Auto-adds current provider to queue if empty
/// - Switches to P1 before enabling
async fn set_enabled(
    State((state, _)): State<Shared>,
    Json(req): Json<SetEnabledRequest>,
) -> Json<serde_json::Value> {
    let mut config = match state.db.get_proxy_config_for_app(&req.app).await {
        Ok(c) => c,
        Err(e) => return err(e.to_string()),
    };

    if req.enabled && !config.enabled {
        return err("需要先启用该应用的代理接管，再开启故障转移");
    }

    if req.enabled {
        // Ensure queue is non-empty
        let mut queue = match state.db.get_failover_queue(&req.app) {
            Ok(q) => q,
            Err(e) => return err(e.to_string()),
        };

        if queue.is_empty() {
            // Auto-add current provider as P1
            let app_enum = match crate::app_config::AppType::from_str(&req.app) {
                Ok(a) => a,
                Err(_) => return err(format!("无效的应用类型: {}", req.app)),
            };
            let current_id =
                match crate::settings::get_effective_current_provider(&state.db, &app_enum) {
                    Ok(Some(id)) => id,
                    Ok(None) => {
                        return err("故障转移队列为空，且未设置当前供应商，无法开启故障转移")
                    }
                    Err(e) => return err(e.to_string()),
                };

            if let Err(e) = state.db.add_to_failover_queue(&req.app, &current_id) {
                return err(e.to_string());
            }
            queue = match state.db.get_failover_queue(&req.app) {
                Ok(q) => q,
                Err(e) => return err(e.to_string()),
            };
        }

        // Switch to P1
        let p1 = match queue.first() {
            Some(item) => item.provider_id.clone(),
            None => return err("故障转移队列为空，无法开启故障转移"),
        };

        if let Err(e) = state.proxy_service.switch_proxy_target(&req.app, &p1).await {
            return err(e);
        }
    }

    config.auto_failover_enabled = req.enabled;

    match state.db.update_proxy_config_for_app(config).await {
        Ok(()) => ok(true),
        Err(e) => err(e.to_string()),
    }
}

// ---- Provider health / circuit breaker routes ----

async fn get_provider_health(
    State((state, _)): State<Shared>,
    axum::extract::Path(provider_id): axum::extract::Path<String>,
    axum::extract::Query(q): axum::extract::Query<AppRequest>,
) -> Json<serde_json::Value> {
    match state.db.get_provider_health(&provider_id, &q.app).await {
        Ok(health) => ok(health),
        Err(e) => err(e.to_string()),
    }
}

#[derive(Deserialize)]
struct HealthRequest {
    #[serde(rename = "appType")]
    app_type: String,
    #[serde(rename = "providerId")]
    provider_id: String,
}

async fn reset_circuit_breaker(
    State((state, _)): State<Shared>,
    Json(req): Json<HealthRequest>,
) -> Json<serde_json::Value> {
    match state.db.reset_provider_health(&req.provider_id, &req.app_type).await {
        Ok(()) => ok(true),
        Err(e) => err(e.to_string()),
    }
}

async fn get_cb_config(
    State((state, _)): State<Shared>,
) -> Json<serde_json::Value> {
    match state.db.get_circuit_breaker_config().await {
        Ok(config) => ok(config),
        Err(e) => err(e.to_string()),
    }
}

async fn update_cb_config(
    State((state, _)): State<Shared>,
    Json(config): Json<crate::proxy::circuit_breaker::CircuitBreakerConfig>,
) -> Json<serde_json::Value> {
    match state.db.update_circuit_breaker_config(&config).await {
        Ok(()) => ok(true),
        Err(e) => err(e.to_string()),
    }
}
