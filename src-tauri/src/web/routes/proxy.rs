//! Proxy routes — all logic delegated to `ProxyService`.

use axum::{
    extract::{Query, State},
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

use crate::services::ProviderService;
use crate::store::AppState;
use crate::web::WsState;
use std::str::FromStr;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/status", get(get_status))
        .route("/start", post(start))
        .route("/stop", post(stop))
        .route("/stop-with-restore", post(stop_with_restore))
        .route("/switch", post(switch_provider))
        .route("/takeover", get(get_takeover_status))
        .route("/takeover", post(set_takeover))
        .route("/config", get(get_config))
        .route("/config", post(update_config))
        .route("/config/global", get(get_global_config))
        .route("/config/global", post(update_global_config))
        .route("/config/app", get(get_app_config))
        .route("/config/app", post(update_app_config))
        .route("/default-cost-multiplier", get(get_default_cost_multiplier))
        .route("/default-cost-multiplier", put(set_default_cost_multiplier))
        .route("/pricing-model-source", get(get_pricing_model_source))
        .route("/pricing-model-source", put(set_pricing_model_source))
}

#[derive(Deserialize)]
struct SwitchRequest {
    #[serde(rename = "appType")]
    app_type: String,
    #[serde(rename = "providerId")]
    provider_id: String,
}

#[derive(Deserialize)]
struct TakeoverRequest {
    app: String,
    enabled: bool,
}

#[derive(Deserialize)]
struct StringValue {
    value: String,
}

// ---- helpers ----

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({"success": true, "data": data}))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({"success": false, "error": msg.to_string()}))
}

// ---- handlers ----

async fn get_status(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match state.proxy_service.get_status().await {
        Ok(status) => ok(status),
        Err(e) => err(e),
    }
}

async fn start(State((state, ws)): State<Shared>) -> Json<serde_json::Value> {
    match state.proxy_service.start().await {
        Ok(info) => {
            broadcast(&ws, "proxy.started", json!({"port": info.port}));
            ok(info)
        }
        Err(e) => err(e),
    }
}

async fn stop(State((state, ws)): State<Shared>) -> Json<serde_json::Value> {
    match state.proxy_service.get_takeover_status().await {
        Ok(t) if t.claude || t.codex || t.gemini || t.opencode || t.openclaw => {
            return err("仍有应用处于代理接管状态，请先在设置中关闭对应应用接管后再停止本地路由。");
        }
        Err(e) => return err(e),
        _ => {}
    }
    match state.proxy_service.stop().await {
        Ok(()) => {
            broadcast(&ws, "proxy.stopped", json!({}));
            ok(true)
        }
        Err(e) => err(e),
    }
}

async fn stop_with_restore(State((state, ws)): State<Shared>) -> Json<serde_json::Value> {
    match state.proxy_service.stop_with_restore().await {
        Ok(()) => {
            broadcast(&ws, "proxy.stopped", json!({"restored": true}));
            ok(true)
        }
        Err(e) => err(e),
    }
}

async fn switch_provider(
    State((state, ws)): State<Shared>,
    Json(req): Json<SwitchRequest>,
) -> Json<serde_json::Value> {
    let app_type = match crate::app_config::AppType::from_str(&req.app_type) {
        Ok(a) => a,
        Err(_) => return err(format!("Invalid app type: {}", req.app_type)),
    };

    // Delegate to ProviderService::switch — handles takeover, normal mode, DB, live config
    match ProviderService::switch(&state, app_type, &req.provider_id) {
        Ok(_) => {
            // Also update proxy target if proxy is running
            let _ = state
                .proxy_service
                .switch_proxy_target(&req.app_type, &req.provider_id)
                .await;
            broadcast(
                &ws,
                "proxy.provider_switched",
                json!({"app": req.app_type, "id": req.provider_id}),
            );
            ok(true)
        }
        Err(e) => err(e.to_string()),
    }
}

async fn get_takeover_status(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match state.proxy_service.get_takeover_status().await {
        Ok(status) => ok(status),
        Err(e) => err(e),
    }
}

async fn set_takeover(
    State((state, ws)): State<Shared>,
    Json(req): Json<TakeoverRequest>,
) -> Json<serde_json::Value> {
    match state
        .proxy_service
        .set_takeover_for_app(&req.app, req.enabled)
        .await
    {
        Ok(()) => {
            broadcast(
                &ws,
                "proxy.takeover_changed",
                json!({"app": req.app, "enabled": req.enabled}),
            );
            ok(true)
        }
        Err(e) => err(e),
    }
}

async fn get_config(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match state.proxy_service.get_config().await {
        Ok(c) => ok(c),
        Err(e) => err(e),
    }
}

async fn update_config(
    State((state, _)): State<Shared>,
    Json(config): Json<crate::proxy::ProxyConfig>,
) -> Json<serde_json::Value> {
    match state.proxy_service.update_config(&config).await {
        Ok(()) => ok(true),
        Err(e) => err(e),
    }
}

async fn get_global_config(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match state.db.get_global_proxy_config().await {
        Ok(c) => ok(c),
        Err(e) => err(e.to_string()),
    }
}

async fn update_global_config(
    State((state, _)): State<Shared>,
    Json(config): Json<crate::proxy::types::GlobalProxyConfig>,
) -> Json<serde_json::Value> {
    match state.db.update_global_proxy_config(config).await {
        Ok(()) => ok(true),
        Err(e) => err(e.to_string()),
    }
}

async fn get_app_config(
    State((state, _)): State<Shared>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let app = params.get("app").cloned().unwrap_or_default();
    match state.db.get_proxy_config_for_app(&app).await {
        Ok(c) => ok(c),
        Err(e) => err(e.to_string()),
    }
}

async fn update_app_config(
    State((state, _)): State<Shared>,
    Json(config): Json<crate::proxy::types::AppProxyConfig>,
) -> Json<serde_json::Value> {
    let app_type = config.app_type.clone();
    let circuit_config = crate::proxy::CircuitBreakerConfig::from(&config);

    match state.db.update_proxy_config_for_app(config).await {
        Ok(()) => {
            let _ = state
                .proxy_service
                .update_circuit_breaker_config_for_app(&app_type, circuit_config)
                .await;
            ok(true)
        }
        Err(e) => err(e.to_string()),
    }
}

async fn get_default_cost_multiplier(
    State((state, _)): State<Shared>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let app = params
        .get("app")
        .cloned()
        .unwrap_or_else(|| "claude".into());
    match state.db.get_default_cost_multiplier(&app).await {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn set_default_cost_multiplier(
    State((state, _)): State<Shared>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<StringValue>,
) -> Json<serde_json::Value> {
    if payload
        .value
        .trim()
        .parse::<f64>()
        .map(|v| v < 0.0)
        .unwrap_or(true)
    {
        return err("Invalid multiplier");
    }
    let app = params
        .get("app")
        .cloned()
        .unwrap_or_else(|| "claude".into());
    match state
        .db
        .set_default_cost_multiplier(&app, payload.value.trim())
        .await
    {
        Ok(()) => ok(true),
        Err(e) => err(e.to_string()),
    }
}

async fn get_pricing_model_source(
    State((state, _)): State<Shared>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let app = params
        .get("app")
        .cloned()
        .unwrap_or_else(|| "claude".into());
    match state.db.get_pricing_model_source(&app).await {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn set_pricing_model_source(
    State((state, _)): State<Shared>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<StringValue>,
) -> Json<serde_json::Value> {
    let value = payload.value.trim();
    if value != "request" && value != "response" {
        return err("Invalid pricing model source");
    }
    let app = params
        .get("app")
        .cloned()
        .unwrap_or_else(|| "claude".into());
    match state.db.set_pricing_model_source(&app, value).await {
        Ok(()) => ok(true),
        Err(e) => err(e.to_string()),
    }
}

fn broadcast(ws: &WsState, event: &str, data: serde_json::Value) {
    let _ = ws.tx.send(json!({"event": event, "data": data}));
}
