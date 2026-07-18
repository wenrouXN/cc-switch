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
        .route("/apply-claude-plugin", post(apply_claude_plugin))
        .route(
            "/claude-onboarding-skip",
            post(apply_claude_onboarding_skip),
        )
        .route(
            "/claude-onboarding-skip",
            delete(clear_claude_onboarding_skip),
        )
        .route("/webdav/settings", post(webdav_save_settings))
        .route("/webdav/test", post(webdav_test))
        .route("/webdav/upload", post(webdav_upload))
        .route("/webdav/download", post(webdav_download))
        .route("/webdav/remote-info", get(webdav_remote_info))
        .route("/s3/settings", post(s3_save_settings))
        .route("/s3/test", post(s3_test))
        .route("/s3/upload", post(s3_upload))
        .route("/s3/download", post(s3_download))
        .route("/s3/remote-info", get(s3_remote_info))
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
        Ok(s) => match crate::commands::save_settings_shared(state.as_ref(), s).await {
            Ok(saved) => ok(saved),
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

async fn create_backup(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
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

async fn get_log_config(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
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

async fn get_optimizer_config(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
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

async fn get_rectifier_config(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
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

async fn sync_providers_live(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
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
    let requested = req.path.as_deref().unwrap_or("<default>");
    err(format!(
        "Headless WebUI does not support changing the app config directory override (requested: {requested}). Use the desktop app instead."
    ))
}

#[derive(Deserialize)]
struct ApplyClaudePluginReq {
    official: bool,
}

async fn apply_claude_plugin(Json(req): Json<ApplyClaudePluginReq>) -> Json<serde_json::Value> {
    let result = if req.official {
        crate::claude_plugin::clear_claude_config()
    } else {
        crate::claude_plugin::write_claude_config()
    };
    match result {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn apply_claude_onboarding_skip() -> Json<serde_json::Value> {
    match crate::claude_mcp::set_has_completed_onboarding() {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn clear_claude_onboarding_skip() -> Json<serde_json::Value> {
    match crate::claude_mcp::clear_has_completed_onboarding() {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

#[derive(Deserialize)]
struct WebDavSaveReq {
    settings: crate::settings::WebDavSyncSettings,
    #[serde(default, rename = "passwordTouched")]
    password_touched: Option<bool>,
}

async fn webdav_save_settings(Json(req): Json<WebDavSaveReq>) -> Json<serde_json::Value> {
    let password_touched = req.password_touched.unwrap_or(false);
    let existing = crate::settings::get_webdav_sync_settings();
    let mut sync_settings = req.settings;
    if let Some(ref existing_settings) = existing {
        if !password_touched && sync_settings.password.is_empty() {
            sync_settings.password = existing_settings.password.clone();
        }
        sync_settings.status = existing_settings.status.clone();
    }
    sync_settings.normalize();
    if let Err(e) = sync_settings.validate() {
        return err(e.to_string());
    }
    match crate::settings::set_webdav_sync_settings(Some(sync_settings)) {
        Ok(()) => ok(json!({ "success": true })),
        Err(e) => err(e.to_string()),
    }
}

#[derive(Deserialize)]
struct WebDavTestReq {
    settings: crate::settings::WebDavSyncSettings,
    #[serde(default, rename = "preserveEmptyPassword")]
    preserve_empty_password: Option<bool>,
}

async fn webdav_test(Json(req): Json<WebDavTestReq>) -> Json<serde_json::Value> {
    let preserve = req.preserve_empty_password.unwrap_or(true);
    let existing = crate::settings::get_webdav_sync_settings();
    let mut settings = req.settings;
    if preserve {
        if let Some(ref existing_settings) = existing {
            if settings.password.is_empty() {
                settings.password = existing_settings.password.clone();
            }
        }
    }
    match crate::services::webdav_sync::check_connection(&settings).await {
        Ok(()) => ok(json!({ "success": true, "message": "WebDAV connection ok" })),
        Err(e) => err(e.to_string()),
    }
}

async fn webdav_upload(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    let Some(mut settings) = crate::settings::get_webdav_sync_settings() else {
        return err("WebDAV sync is not configured.");
    };
    if !settings.enabled {
        return err("WebDAV sync is disabled.");
    }
    match crate::services::webdav_sync::upload(state.db.as_ref(), &mut settings).await {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn webdav_download(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    let Some(mut settings) = crate::settings::get_webdav_sync_settings() else {
        return err("WebDAV sync is not configured.");
    };
    if !settings.enabled {
        return err("WebDAV sync is disabled.");
    }
    match crate::services::webdav_sync::download(state.db.as_ref(), &mut settings).await {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn webdav_remote_info() -> Json<serde_json::Value> {
    let Some(settings) = crate::settings::get_webdav_sync_settings() else {
        return err("WebDAV sync is not configured.");
    };
    if !settings.enabled {
        return err("WebDAV sync is disabled.");
    }
    match crate::services::webdav_sync::fetch_remote_info(&settings).await {
        Ok(Some(info)) => ok(info),
        Ok(None) => ok(json!({ "empty": true })),
        Err(e) => err(e.to_string()),
    }
}

#[derive(Deserialize)]
struct S3SaveReq {
    settings: crate::settings::S3SyncSettings,
    #[serde(default, rename = "passwordTouched")]
    password_touched: Option<bool>,
}

async fn s3_save_settings(Json(req): Json<S3SaveReq>) -> Json<serde_json::Value> {
    let password_touched = req.password_touched.unwrap_or(false);
    let existing = crate::settings::get_s3_sync_settings();
    let mut sync_settings = req.settings;
    if let Some(ref existing_settings) = existing {
        if !password_touched && sync_settings.secret_access_key.is_empty() {
            sync_settings.secret_access_key = existing_settings.secret_access_key.clone();
        }
        sync_settings.status = existing_settings.status.clone();
    }
    sync_settings.normalize();
    if let Err(e) = sync_settings.validate() {
        return err(e.to_string());
    }
    match crate::settings::set_s3_sync_settings(Some(sync_settings)) {
        Ok(()) => ok(json!({ "success": true })),
        Err(e) => err(e.to_string()),
    }
}

#[derive(Deserialize)]
struct S3TestReq {
    settings: crate::settings::S3SyncSettings,
    #[serde(default, rename = "preserveEmptyPassword")]
    preserve_empty_password: Option<bool>,
}

async fn s3_test(Json(req): Json<S3TestReq>) -> Json<serde_json::Value> {
    let preserve = req.preserve_empty_password.unwrap_or(true);
    let existing = crate::settings::get_s3_sync_settings();
    let mut settings = req.settings;
    if preserve {
        if let Some(ref existing_settings) = existing {
            if settings.secret_access_key.is_empty() {
                settings.secret_access_key = existing_settings.secret_access_key.clone();
            }
        }
    }
    match crate::services::s3_sync::check_connection(&settings).await {
        Ok(()) => ok(json!({ "success": true, "message": "S3 connection ok" })),
        Err(e) => err(e.to_string()),
    }
}

async fn s3_upload(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    let Some(mut settings) = crate::settings::get_s3_sync_settings() else {
        return err("S3 sync is not configured.");
    };
    if !settings.enabled {
        return err("S3 sync is disabled.");
    }
    match crate::services::s3_sync::upload(state.db.as_ref(), &mut settings).await {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn s3_download(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    let Some(mut settings) = crate::settings::get_s3_sync_settings() else {
        return err("S3 sync is not configured.");
    };
    if !settings.enabled {
        return err("S3 sync is disabled.");
    }
    match crate::services::s3_sync::download(state.db.as_ref(), &mut settings).await {
        Ok(v) => ok(v),
        Err(e) => err(e.to_string()),
    }
}

async fn s3_remote_info() -> Json<serde_json::Value> {
    let Some(settings) = crate::settings::get_s3_sync_settings() else {
        return err("S3 sync is not configured.");
    };
    if !settings.enabled {
        return err("S3 sync is disabled.");
    }
    match crate::services::s3_sync::fetch_remote_info(&settings).await {
        Ok(Some(info)) => ok(info),
        Ok(None) => ok(json!({ "empty": true })),
        Err(e) => err(e.to_string()),
    }
}
