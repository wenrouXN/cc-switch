//! Provider routes — thin HTTP shell over `ProviderService` / Database.
//!
//! Mirrors `commands/provider.rs` for the web UI write path (CRUD + import + sort).

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;

use crate::app_config::AppType;
use crate::provider::Provider;
use crate::services::{ProviderService, ProviderSortUpdate};
use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/", get(list_providers))
        .route("/", post(add_provider))
        .route("/current", get(get_current))
        .route("/sort", post(update_sort))
        .route("/import-default", post(import_default))
        .route("/import-opencode-live", post(import_opencode_live))
        .route("/import-openclaw-live", post(import_openclaw_live))
        .route("/import-hermes-live", post(import_hermes_live))
        .route(
            "/import-claude-desktop-from-claude",
            post(import_claude_desktop_from_claude),
        )
        .route(
            "/ensure-claude-desktop-official",
            post(ensure_claude_desktop_official),
        )
        .route("/ensure-codex-official", post(ensure_codex_official))
        .route(
            "/ensure-grokbuild-official",
            post(ensure_grokbuild_official),
        )
        .route("/claude-desktop-status", get(claude_desktop_status))
        .route(
            "/claude-desktop-default-routes",
            get(claude_desktop_default_routes),
        )
        .route("/opencode-live-ids", get(opencode_live_ids))
        .route("/openclaw-live-ids", get(openclaw_live_ids))
        .route("/hermes-live-ids", get(hermes_live_ids))
        .route("/:id", get(get_one))
        .route("/:id", put(update_provider))
        .route("/:id", delete(delete_provider))
        .route("/:id/switch", post(switch))
        .route("/:id/remove-from-live", post(remove_from_live))
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": data }))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({ "success": false, "error": msg.to_string() }))
}

#[derive(Deserialize)]
struct AppQuery {
    app: String,
}

#[derive(Deserialize)]
struct ProviderBody {
    provider: Provider,
    app: String,
    #[serde(default, rename = "addToLive")]
    add_to_live: Option<bool>,
    #[serde(default, rename = "originalId")]
    original_id: Option<String>,
}

#[derive(Deserialize)]
struct SortBody {
    updates: Vec<ProviderSortUpdate>,
    app: String,
}

#[derive(Deserialize)]
struct AppBody {
    app: String,
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
        Ok(mut providers) => {
            for provider in providers.values_mut() {
                crate::web::redaction::redact_sensitive_values(&mut provider.settings_config);
            }
            ok(providers)
        }
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
    Path(id): Path<String>,
    Query(q): Query<AppQuery>,
) -> Json<serde_json::Value> {
    match state.db.get_provider_by_id(&id, &q.app) {
        Ok(Some(mut provider)) => {
            crate::web::redaction::redact_sensitive_values(&mut provider.settings_config);
            ok(provider)
        }
        Ok(None) => err(format!("Provider not found: {}", id)),
        Err(e) => err(e.to_string()),
    }
}

async fn add_provider(
    State((state, _)): State<Shared>,
    Json(body): Json<ProviderBody>,
) -> Json<serde_json::Value> {
    let app_type = match AppType::from_str(&body.app) {
        Ok(a) => a,
        Err(e) => return err(e.to_string()),
    };
    match ProviderService::add(
        &state,
        app_type,
        body.provider,
        body.add_to_live.unwrap_or(true),
    ) {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn update_provider(
    State((state, _)): State<Shared>,
    Path(id): Path<String>,
    Json(body): Json<ProviderBody>,
) -> Json<serde_json::Value> {
    let app_type = match AppType::from_str(&body.app) {
        Ok(a) => a,
        Err(e) => return err(e.to_string()),
    };
    let original = body
        .original_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(id.as_str());
    let mut provider = body.provider;
    match state.db.get_provider_by_id(original, &body.app) {
        Ok(Some(existing)) => crate::web::redaction::preserve_masked_sensitive_values(
            &mut provider.settings_config,
            &existing.settings_config,
        ),
        Ok(None) => return err(format!("Provider not found: {}", original)),
        Err(e) => return err(e.to_string()),
    }
    match ProviderService::update(&state, app_type, Some(original), provider) {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn delete_provider(
    State((state, _)): State<Shared>,
    Path(id): Path<String>,
    Query(q): Query<AppQuery>,
) -> Json<serde_json::Value> {
    let app_type = match AppType::from_str(&q.app) {
        Ok(a) => a,
        Err(e) => return err(e.to_string()),
    };
    match ProviderService::delete(&state, app_type, &id) {
        Ok(()) => ok(true),
        Err(e) => err(e.to_string()),
    }
}

async fn switch(
    State((state, _)): State<Shared>,
    Path(id): Path<String>,
    Query(q): Query<AppQuery>,
) -> Json<serde_json::Value> {
    let app_type = match AppType::from_str(&q.app) {
        Ok(a) => a,
        Err(e) => return err(e.to_string()),
    };
    match ProviderService::switch(&state, app_type, &id) {
        Ok(mut result) => {
            // Align proxy in-memory target with live/DB switch. Surface failures as
            // warnings instead of silently leaving the proxy on a stale provider.
            if let Err(e) = state.proxy_service.switch_proxy_target(&q.app, &id).await {
                result
                    .warnings
                    .push(format!("proxy target update failed: {e}"));
            }
            ok(result)
        }
        Err(e) => err(e.to_string()),
    }
}

async fn remove_from_live(
    State((state, _)): State<Shared>,
    Path(id): Path<String>,
    Json(body): Json<AppBody>,
) -> Json<serde_json::Value> {
    let app_type = match AppType::from_str(&body.app) {
        Ok(a) => a,
        Err(e) => return err(e.to_string()),
    };
    match ProviderService::remove_from_live_config(&state, app_type, &id) {
        Ok(()) => ok(true),
        Err(e) => err(e.to_string()),
    }
}

async fn update_sort(
    State((state, _)): State<Shared>,
    Json(body): Json<SortBody>,
) -> Json<serde_json::Value> {
    let app_type = match AppType::from_str(&body.app) {
        Ok(a) => a,
        Err(e) => return err(e.to_string()),
    };
    match ProviderService::update_sort_order(&state, app_type, body.updates) {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn import_default(
    State((state, _)): State<Shared>,
    Json(body): Json<AppBody>,
) -> Json<serde_json::Value> {
    let app_type = match AppType::from_str(&body.app) {
        Ok(a) => a,
        Err(e) => return err(e.to_string()),
    };
    match ProviderService::import_default_config(&state, app_type) {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn import_opencode_live(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match crate::services::provider::import_opencode_providers_from_live(&state) {
        Ok(n) => ok(n),
        Err(e) => err(e.to_string()),
    }
}

async fn import_openclaw_live(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match crate::services::provider::import_openclaw_providers_from_live(&state) {
        Ok(n) => ok(n),
        Err(e) => err(e.to_string()),
    }
}

async fn import_hermes_live(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match crate::services::provider::import_hermes_providers_from_live(&state) {
        Ok(n) => ok(n),
        Err(e) => err(e.to_string()),
    }
}

async fn import_claude_desktop_from_claude(
    State((state, _)): State<Shared>,
) -> Json<serde_json::Value> {
    // Thin reimplementation of commands/provider.rs::import_claude_desktop_providers_from_claude
    // without Tauri AppHandle dependency.
    use crate::provider::ClaudeDesktopMode;

    let claude_providers = match state.db.get_all_providers(AppType::Claude.as_str()) {
        Ok(p) => p,
        Err(e) => return err(e.to_string()),
    };
    let existing_ids = match state.db.get_provider_ids(AppType::ClaudeDesktop.as_str()) {
        Ok(ids) => ids,
        Err(e) => return err(e.to_string()),
    };

    let mut imported = 0usize;
    for provider in claude_providers.values() {
        if existing_ids.contains(&provider.id) {
            continue;
        }
        let mut desktop_provider = provider.clone();
        desktop_provider.in_failover_queue = false;
        let meta = desktop_provider.meta.get_or_insert_with(Default::default);

        if crate::claude_desktop_config::is_compatible_direct_provider(provider) {
            meta.claude_desktop_mode = Some(ClaudeDesktopMode::Direct);
        } else {
            // Proxy-mode import requires desktop route suggestion; skip if unavailable.
            // Keep parity with desktop import: only compatible providers are imported.
            continue;
        }

        if let Err(e) = state
            .db
            .save_provider(AppType::ClaudeDesktop.as_str(), &desktop_provider)
        {
            return err(e.to_string());
        }
        imported += 1;
    }

    if let Err(e) = state.db.ensure_official_seed_by_id(
        crate::database::CLAUDE_DESKTOP_OFFICIAL_PROVIDER_ID,
        AppType::ClaudeDesktop,
    ) {
        log::warn!("Failed to ensure claude-desktop-official seed during import: {e}");
    }

    ok(imported)
}

async fn ensure_claude_desktop_official(
    State((state, _)): State<Shared>,
) -> Json<serde_json::Value> {
    match state.db.ensure_official_seed_by_id(
        crate::database::CLAUDE_DESKTOP_OFFICIAL_PROVIDER_ID,
        AppType::ClaudeDesktop,
    ) {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn ensure_codex_official(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match state
        .db
        .ensure_official_seed_by_id(crate::database::CODEX_OFFICIAL_PROVIDER_ID, AppType::Codex)
    {
        Ok(value) => ok(value),
        Err(error) => err(error),
    }
}

async fn ensure_grokbuild_official(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match state.db.ensure_official_seed_by_id(
        crate::database::GROKBUILD_OFFICIAL_PROVIDER_ID,
        AppType::GrokBuild,
    ) {
        Ok(value) => ok(value),
        Err(error) => err(error),
    }
}

async fn claude_desktop_status(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    let proxy_running = state.proxy_service.is_running().await;
    match crate::claude_desktop_config::get_status(state.db.as_ref(), proxy_running) {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn claude_desktop_default_routes() -> Json<serde_json::Value> {
    ok(crate::claude_desktop_config::default_proxy_routes())
}

async fn opencode_live_ids() -> Json<serde_json::Value> {
    match crate::opencode_config::get_providers() {
        Ok(providers) => ok(providers.keys().cloned().collect::<Vec<_>>()),
        Err(e) => err(e.to_string()),
    }
}

async fn openclaw_live_ids() -> Json<serde_json::Value> {
    match crate::openclaw_config::get_providers() {
        Ok(providers) => ok(providers.keys().cloned().collect::<Vec<_>>()),
        Err(e) => err(e.to_string()),
    }
}

async fn hermes_live_ids() -> Json<serde_json::Value> {
    match crate::hermes_config::get_providers() {
        Ok(providers) => ok(providers.keys().cloned().collect::<Vec<_>>()),
        Err(e) => err(e.to_string()),
    }
}

// ---- Universal Provider routes ----
// Thin shell over ProviderService (same as commands/provider.rs). Never db-only
// delete/upsert — delete must clear universal-{app}-{id} children.

pub fn universal_routes() -> Router<Shared> {
    Router::new()
        .route("/", get(list_universal))
        .route("/:id", get(get_universal))
        .route("/", post(upsert_universal))
        .route("/:id", delete(delete_universal))
        .route("/:id/sync", post(sync_universal))
}

fn redact_universal_provider(
    provider: crate::provider::UniversalProvider,
) -> crate::provider::UniversalProvider {
    let Ok(mut value) = serde_json::to_value(&provider) else {
        return provider;
    };
    crate::web::redaction::redact_sensitive_values(&mut value);
    serde_json::from_value(value).unwrap_or(provider)
}

fn is_mask_or_empty_secret(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.is_empty() || trimmed.contains("***")
}

async fn list_universal(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match ProviderService::list_universal(&state) {
        Ok(providers) => {
            let redacted = providers
                .into_iter()
                .map(|(id, provider)| (id, redact_universal_provider(provider)))
                .collect::<std::collections::HashMap<_, _>>();
            ok(redacted)
        }
        Err(e) => err(e.to_string()),
    }
}

async fn get_universal(
    State((state, _)): State<Shared>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match ProviderService::get_universal(&state, &id) {
        Ok(Some(provider)) => ok(redact_universal_provider(provider)),
        Ok(None) => err(format!("Universal provider not found: {id}")),
        Err(e) => err(e.to_string()),
    }
}

async fn upsert_universal(
    State((state, _)): State<Shared>,
    Json(mut provider): Json<crate::provider::UniversalProvider>,
) -> Json<serde_json::Value> {
    // Preserve existing apiKey when the client sends empty / masked value
    // (GET responses are redacted with ***).
    if is_mask_or_empty_secret(&provider.api_key) {
        match ProviderService::get_universal(&state, &provider.id) {
            Ok(Some(existing)) => provider.api_key = existing.api_key,
            Ok(None) => {}
            Err(e) => return err(e.to_string()),
        }
    }
    match ProviderService::upsert_universal(&state, provider) {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn delete_universal(
    State((state, _)): State<Shared>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match ProviderService::delete_universal(&state, &id) {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn sync_universal(
    State((state, _)): State<Shared>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match ProviderService::sync_universal_to_apps(&state, &id) {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}
