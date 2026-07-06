//! Settings routes — delegate to settings/config/app_store modules.

use axum::{
    extract::{Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/", get(get_settings))
        .route("/", put(save_settings))
        .route("/backups", get(list_backups))
        .route("/backups", post(create_backup))
        .route("/backups/:name", put(rename_backup))
        .route("/backups/:name", delete(delete_backup))
        .route("/backups/:name/restore", post(restore_backup))
        .route("/log-config", get(get_log_config))
        .route("/log-config", put(set_log_config))
        .route("/optimizer-config", get(get_optimizer_config))
        .route("/optimizer-config", put(set_optimizer_config))
        .route("/rectifier-config", get(get_rectifier_config))
        .route("/rectifier-config", put(set_rectifier_config))
        .route("/sync-providers-live", post(sync_providers_live))
        .route("/config-dir", get(get_config_dir))
        .route("/claude-code-path", get(get_claude_code_path))
        .route("/app-config-path", get(get_app_config_path))
        .route("/app-config-dir-override", get(get_override))
        .route("/app-config-dir-override", post(set_override))
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": data }))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({ "success": false, "error": msg.to_string() }))
}

async fn get_settings() -> Json<serde_json::Value> {
    ok(crate::settings::get_settings_for_frontend())
}

async fn save_settings(
    State((state, _)): State<Shared>,
    Json(settings): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    match serde_json::from_value::<crate::settings::AppSettings>(settings) {
        Ok(s) => match crate::settings::update_settings(s) {
            Ok(()) => ok(true),
            Err(e) => err(e),
        },
        Err(e) => err(format!("Invalid settings: {}", e)),
    }
}

async fn list_backups() -> Json<serde_json::Value> {
    match crate::database::Database::list_backups() {
        Ok(backups) => ok(backups),
        Err(e) => err(e),
    }
}

async fn create_backup(
    State((state, _)): State<Shared>,
) -> Json<serde_json::Value> {
    match state.db.backup_database_file() {
        Ok(Some(path)) => ok(path.to_string_lossy().to_string()),
        Ok(None) => ok(""),
        Err(e) => err(e),
    }
}

#[derive(Deserialize)]
struct RenameBackupReq {
    #[serde(rename = "newName")]
    new_name: String,
}

async fn rename_backup(
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<RenameBackupReq>,
) -> Json<serde_json::Value> {
    match crate::database::Database::rename_backup(&name, &req.new_name) {
        Ok(new_name) => ok(new_name),
        Err(e) => err(e),
    }
}

async fn delete_backup(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match crate::database::Database::delete_backup(&name) {
        Ok(()) => ok(true),
        Err(e) => err(e),
    }
}

async fn restore_backup(
    State((state, _)): State<Shared>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match state.db.restore_from_backup(&name) {
        Ok(msg) => ok(msg),
        Err(e) => err(e),
    }
}

async fn get_log_config(
    State((state, _)): State<Shared>,
) -> Json<serde_json::Value> {
    match state.db.get_log_config() {
        Ok(c) => ok(c),
        Err(e) => err(e),
    }
}

async fn set_log_config(
    State((state, _)): State<Shared>,
    Json(config): Json<crate::proxy::types::LogConfig>,
) -> Json<serde_json::Value> {
    match state.db.set_log_config(&config) {
        Ok(()) => ok(true),
        Err(e) => err(e),
    }
}

async fn get_optimizer_config(
    State((state, _)): State<Shared>,
) -> Json<serde_json::Value> {
    match state.db.get_optimizer_config() {
        Ok(c) => ok(c),
        Err(e) => err(e),
    }
}

async fn set_optimizer_config(
    State((state, _)): State<Shared>,
    Json(config): Json<crate::proxy::types::OptimizerConfig>,
) -> Json<serde_json::Value> {
    match state.db.set_optimizer_config(&config) {
        Ok(()) => ok(true),
        Err(e) => err(e),
    }
}

async fn get_rectifier_config(
    State((state, _)): State<Shared>,
) -> Json<serde_json::Value> {
    match state.db.get_rectifier_config() {
        Ok(c) => ok(c),
        Err(e) => err(e),
    }
}

async fn set_rectifier_config(
    State((state, _)): State<Shared>,
    Json(config): Json<crate::proxy::types::RectifierConfig>,
) -> Json<serde_json::Value> {
    match state.db.set_rectifier_config(&config) {
        Ok(()) => ok(true),
        Err(e) => err(e),
    }
}

async fn sync_providers_live(
    State((state, _)): State<Shared>,
) -> Json<serde_json::Value> {
    match crate::services::ProviderService::sync_current_to_live(&state) {
        Ok(()) => ok(true),
        Err(e) => err(e),
    }
}

async fn get_config_dir() -> Json<serde_json::Value> {
    let path = crate::config::get_app_config_dir();
    ok(json!({ "path": path.to_string_lossy() }))
}

async fn get_claude_code_path() -> Json<serde_json::Value> {
    let path = crate::config::get_claude_config_dir();
    ok(json!({ "path": path.to_string_lossy() }))
}

async fn get_app_config_path() -> Json<serde_json::Value> {
    let path = crate::config::get_app_config_path();
    ok(json!({ "path": path.to_string_lossy() }))
}

async fn get_override() -> Json<serde_json::Value> {
    let path = crate::app_store::get_app_config_dir_override();
    ok(json!({ "path": path.map(|p| p.to_string_lossy().to_string()) }))
}

#[derive(Deserialize)]
struct SetOverrideReq {
    path: Option<String>,
}

async fn set_override(Json(req): Json<SetOverrideReq>) -> Json<serde_json::Value> {
    // Desktop-only: set_app_config_dir_to_store needs AppHandle
    ok(true)
}
