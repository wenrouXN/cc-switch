//! WebSocket handler — bidirectional JSON-RPC.
//!
//! Frontend uses `invoke("command_name", params)` which the transport layer
//! converts to JSON-RPC over WS. This handler dispatches to the same
//! `ProxyService` / `Database` methods as the REST routes.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::app_config::AppType;
use crate::services::ProviderService;
use crate::store::AppState;
use crate::web::WsState;
use std::str::FromStr;

type Shared = (Arc<AppState>, Arc<WsState>);

#[derive(Deserialize)]
pub struct WsAuthQuery {
    token: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsAuthQuery>,
    State((state, ws_state)): State<Shared>,
) -> impl IntoResponse {
    if !crate::web::middleware::auth::authentication_disabled()
        && !query
            .token
            .as_deref()
            .is_some_and(crate::web::middleware::auth::is_valid_token)
    {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    }
    ws.on_upgrade(move |socket| handle_ws(socket, state, ws_state))
        .into_response()
}

async fn handle_ws(mut socket: WebSocket, state: Arc<AppState>, ws_state: Arc<WsState>) {
    // Subscribe to broadcast events
    let mut rx = ws_state.tx.subscribe();

    loop {
        tokio::select! {
            // Client message
            msg = socket.recv() => {
                let Some(Ok(Message::Text(text))) = msg else { break };
                let response = dispatch(&state, &text).await;
                let _ = socket.send(Message::Text(response.to_string().into())).await;
            }
            // Broadcast events to client
            Ok(event) = rx.recv() => {
                let _ = socket.send(Message::Text(event.to_string().into())).await;
            }
        }
    }
}

/// Dispatch a JSON-RPC request. Returns the response.
async fn dispatch(state: &Arc<AppState>, text: &str) -> serde_json::Value {
    let Ok(req) = serde_json::from_str::<serde_json::Value>(text) else {
        return json!({"jsonrpc": "2.0", "error": {"code": -32700, "message": "Parse error"}, "id": null});
    };

    let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(json!({}));
    let id = req.get("id").cloned().unwrap_or(json!(null));

    let result = handle_command(state, method, &params).await;

    match result {
        Ok(value) => json!({"jsonrpc": "2.0", "result": value, "id": id}),
        Err(e) => json!({"jsonrpc": "2.0", "error": {"code": -32000, "message": e}, "id": id}),
    }
}

async fn handle_command(
    state: &Arc<AppState>,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    match method {
        // ===== Proxy =====
        "start_proxy_server" => state.proxy_service.start().await.map(|v| json!(v)),
        "stop_proxy_server" => {
            let takeover = state.proxy_service.get_takeover_status().await?;
            if takeover.claude
                || takeover.codex
                || takeover.gemini
                || takeover.opencode
                || takeover.openclaw
            {
                return Err("仍有应用处于代理接管状态，请先关闭接管再停止".into());
            }
            state.proxy_service.stop().await.map(|()| json!(null))
        }
        "stop_proxy_with_restore" => state
            .proxy_service
            .stop_with_restore()
            .await
            .map(|()| json!(null)),
        "get_proxy_status" => state.proxy_service.get_status().await.map(|v| json!(v)),
        "is_proxy_running" => Ok(json!(state.proxy_service.is_running().await)),
        "get_proxy_takeover_status" => state
            .proxy_service
            .get_takeover_status()
            .await
            .map(|v| json!(v)),
        "set_proxy_takeover_for_app" => {
            let app_type = params
                .get("appType")
                .or_else(|| params.get("app_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let enabled = params
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if app_type.is_empty() {
                return Err("appType required".into());
            }
            state
                .proxy_service
                .set_takeover_for_app(app_type, enabled)
                .await
                .map(|()| json!(null))
        }
        "is_live_takeover_active" => state
            .proxy_service
            .is_takeover_active()
            .await
            .map(|v| json!(v)),
        "get_proxy_config" => state.proxy_service.get_config().await.map(|v| json!(v)),
        "update_proxy_config" => {
            let config: crate::proxy::ProxyConfig =
                serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
            state
                .proxy_service
                .update_config(&config)
                .await
                .map(|()| json!(null))
        }
        "get_global_proxy_config" => state
            .db
            .get_global_proxy_config()
            .await
            .map(|v| json!(v))
            .map_err(|e| e.to_string()),
        "update_global_proxy_config" => {
            let config: crate::proxy::types::GlobalProxyConfig =
                serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
            state
                .db
                .update_global_proxy_config(config)
                .await
                .map(|()| json!(null))
                .map_err(|e| e.to_string())
        }
        "get_proxy_config_for_app" => {
            let app = params
                .get("appType")
                .or_else(|| params.get("app_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            state
                .db
                .get_proxy_config_for_app(app)
                .await
                .map(|v| json!(v))
                .map_err(|e| e.to_string())
        }
        "update_proxy_config_for_app" => {
            let config: crate::proxy::types::AppProxyConfig =
                serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
            let app_type = config.app_type.clone();
            let circuit = crate::proxy::CircuitBreakerConfig::from(&config);
            state
                .db
                .update_proxy_config_for_app(config)
                .await
                .map_err(|e| e.to_string())?;
            let _ = state
                .proxy_service
                .update_circuit_breaker_config_for_app(&app_type, circuit)
                .await;
            Ok(json!(null))
        }

        // ===== Provider =====
        "get_providers" => {
            let app = params
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude");
            let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
            ProviderService::list(state, app_type)
                .map(|v| json!(v))
                .map_err(|e| e.to_string())
        }
        "get_current_provider" => {
            let app = params
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude");
            let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
            ProviderService::current(state, app_type)
                .map(|v| json!(v))
                .map_err(|e| e.to_string())
        }
        "switch_provider" => {
            let app = params
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude");
            let id = params
                .get("id")
                .or_else(|| params.get("providerId"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
            ProviderService::switch(state, app_type, id)
                .map(|_| json!(null))
                .map_err(|e| e.to_string())
        }

        // ===== Failover =====
        "get_failover_queue" => {
            let app = params
                .get("appType")
                .or_else(|| params.get("app_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            state
                .db
                .get_failover_queue(app)
                .map(|v| json!(v))
                .map_err(|e| e.to_string())
        }
        "get_available_providers_for_failover" => {
            let app = params
                .get("appType")
                .or_else(|| params.get("app_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            state
                .db
                .get_available_providers_for_failover(app)
                .map(|v| json!(v))
                .map_err(|e| e.to_string())
        }
        "add_to_failover_queue" => {
            let app = params
                .get("appType")
                .or_else(|| params.get("app_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let id = params
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            state
                .db
                .add_to_failover_queue(app, id)
                .map(|()| json!(null))
                .map_err(|e| e.to_string())
        }
        "remove_from_failover_queue" => {
            let app = params
                .get("appType")
                .or_else(|| params.get("app_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let id = params
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            state
                .db
                .remove_from_failover_queue(app, id)
                .map(|()| json!(null))
                .map_err(|e| e.to_string())
        }
        "get_auto_failover_enabled" => {
            let app = params
                .get("appType")
                .or_else(|| params.get("app_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            state
                .db
                .get_proxy_config_for_app(app)
                .await
                .map(|c| json!(c.auto_failover_enabled))
                .map_err(|e| e.to_string())
        }
        "set_auto_failover_enabled" => {
            // Delegate to failover route logic (reuse the REST handler logic)
            let app = params
                .get("appType")
                .or_else(|| params.get("app_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let enabled = params
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let mut config = state
                .db
                .get_proxy_config_for_app(app)
                .await
                .map_err(|e| e.to_string())?;
            if enabled && !config.enabled {
                return Err("需要先启用该应用的代理接管，再开启故障转移".into());
            }
            config.auto_failover_enabled = enabled;
            state
                .db
                .update_proxy_config_for_app(config)
                .await
                .map(|()| json!(null))
                .map_err(|e| e.to_string())
        }

        // ===== Circuit breaker =====
        "get_circuit_breaker_config" => state
            .db
            .get_circuit_breaker_config()
            .await
            .map(|v| json!(v))
            .map_err(|e| e.to_string()),
        "update_circuit_breaker_config" => {
            let config: crate::proxy::CircuitBreakerConfig =
                serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
            state
                .db
                .update_circuit_breaker_config(&config)
                .await
                .map_err(|e| e.to_string())?;
            let _ = state
                .proxy_service
                .update_circuit_breaker_configs(config)
                .await;
            Ok(json!(null))
        }
        "get_provider_health" => {
            let pid = params
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let app = params
                .get("appType")
                .or_else(|| params.get("app_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            state
                .db
                .get_provider_health(pid, app)
                .await
                .map(|v| json!(v))
                .map_err(|e| e.to_string())
        }
        "reset_circuit_breaker" => {
            let pid = params
                .get("providerId")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let app = params
                .get("appType")
                .or_else(|| params.get("app_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            state
                .db
                .update_provider_health(pid, app, true, None)
                .await
                .map_err(|e| e.to_string())?;
            state
                .proxy_service
                .reset_provider_circuit_breaker(pid, app)
                .await?;
            Ok(json!(null))
        }

        // ===== Catch-all =====
        _ => {
            log::warn!("Unknown WS command: {}", method);
            Ok(json!(null))
        }
    }
}
