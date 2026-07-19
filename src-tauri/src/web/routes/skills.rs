//! Skills routes — delegate to Database DAO.

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/", get(get_skills))
        .route("/installed", get(get_installed))
        .route("/repos", get(get_repos))
        .route("/repos", post(add_repo))
        .route("/backups", get(get_backups))
        .route("/discover", post(discover))
        .route("/updates", post(check_updates))
        .route("/import", post(import_skills))
        .route("/unmanaged", post(scan_unmanaged))
        .route("/migrate-storage", post(migrate_storage))
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": data }))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({ "success": false, "error": msg.to_string() }))
}

async fn get_skills(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match state.db.get_all_installed_skills() {
        Ok(skills) => ok(skills),
        Err(e) => err(e),
    }
}

async fn get_installed(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match state.db.get_all_installed_skills() {
        Ok(skills) => ok(skills),
        Err(e) => err(e),
    }
}

async fn get_repos(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match state.db.get_skill_repos() {
        Ok(repos) => ok(repos),
        Err(e) => err(e),
    }
}

async fn add_repo(
    State((state, _)): State<Shared>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    match serde_json::from_value::<crate::services::skill::SkillRepo>(body) {
        Ok(repo) => match state.db.save_skill_repo(&repo) {
            Ok(()) => ok(true),
            Err(e) => err(e),
        },
        Err(e) => err(format!("Invalid repo: {}", e)),
    }
}

async fn get_backups() -> Json<serde_json::Value> {
    err("Skills backups listing is not available in web/headless mode")
}

async fn discover() -> Json<serde_json::Value> {
    err("Skills discovery is not available in web/headless mode")
}

async fn check_updates() -> Json<serde_json::Value> {
    err("Skills update check is not available in web/headless mode")
}

async fn import_skills() -> Json<serde_json::Value> {
    err("Skills import is not available in web/headless mode")
}

async fn scan_unmanaged() -> Json<serde_json::Value> {
    err("Skills unmanaged scan is not available in web/headless mode")
}

async fn migrate_storage() -> Json<serde_json::Value> {
    err("Skills storage migration is not available in web/headless mode")
}
