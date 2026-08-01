use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::commands::{CurrentProfileIds, ProfileDto, ProfilesResponse};
use crate::services::profile::{ProfileScope, ProfileService};
use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/", get(list_profiles))
        .route("/", post(create_profile))
        .route("/clear", post(clear_current_profile))
        .route("/:id", put(update_profile))
        .route("/:id", delete(delete_profile))
        .route("/:id/apply", post(apply_profile))
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": data }))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({ "success": false, "error": msg.to_string() }))
}

async fn list_profiles(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    let profiles = match ProfileService::list(&state) {
        Ok(profiles) => profiles.into_iter().map(ProfileDto::from).collect(),
        Err(error) => return err(error),
    };
    let current_ids = match current_profile_ids(&state) {
        Ok(ids) => ids,
        Err(error) => return err(error),
    };
    ok(ProfilesResponse {
        profiles,
        current_ids,
    })
}

fn current_profile_ids(state: &AppState) -> Result<CurrentProfileIds, String> {
    Ok(CurrentProfileIds {
        claude: state
            .db
            .get_current_profile_id(ProfileScope::Claude.as_str())
            .map_err(|error| error.to_string())?,
        claude_desktop: state
            .db
            .get_current_profile_id(ProfileScope::ClaudeDesktop.as_str())
            .map_err(|error| error.to_string())?,
        codex: state
            .db
            .get_current_profile_id(ProfileScope::Codex.as_str())
            .map_err(|error| error.to_string())?,
    })
}

#[derive(Deserialize)]
struct CreateProfileBody {
    name: String,
    scope: String,
}

async fn create_profile(
    State((state, _)): State<Shared>,
    Json(body): Json<CreateProfileBody>,
) -> Json<serde_json::Value> {
    let scope = match ProfileScope::parse(&body.scope) {
        Ok(scope) => scope,
        Err(error) => return err(error),
    };
    match ProfileService::create(&state, &body.name, scope) {
        Ok(profile) => ok(ProfileDto::from(profile)),
        Err(error) => err(error),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateProfileBody {
    name: Option<String>,
    resnapshot: Option<bool>,
    scope: Option<String>,
}

async fn update_profile(
    State((state, _)): State<Shared>,
    Path(id): Path<String>,
    Json(body): Json<UpdateProfileBody>,
) -> Json<serde_json::Value> {
    let scope = match body.scope.as_deref().map(ProfileScope::parse).transpose() {
        Ok(scope) => scope,
        Err(error) => return err(error),
    };
    match ProfileService::update(
        &state,
        &id,
        body.name,
        body.resnapshot.unwrap_or(false),
        scope,
    ) {
        Ok(profile) => ok(ProfileDto::from(profile)),
        Err(error) => err(error),
    }
}

async fn delete_profile(
    State((state, _)): State<Shared>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match ProfileService::delete(&state, &id) {
        Ok(()) => ok(true),
        Err(error) => err(error),
    }
}

#[derive(Deserialize)]
struct ScopeBody {
    scope: String,
}

async fn clear_current_profile(
    State((state, _)): State<Shared>,
    Json(body): Json<ScopeBody>,
) -> Json<serde_json::Value> {
    let scope = match ProfileScope::parse(&body.scope) {
        Ok(scope) => scope,
        Err(error) => return err(error),
    };
    match state.db.set_current_profile_id(scope.as_str(), None) {
        Ok(()) => ok(true),
        Err(error) => err(error),
    }
}

async fn apply_profile(
    State((state, ws_state)): State<Shared>,
    Path(id): Path<String>,
    Json(body): Json<ScopeBody>,
) -> Json<serde_json::Value> {
    let scope = match ProfileScope::parse(&body.scope) {
        Ok(scope) => scope,
        Err(error) => return err(error),
    };
    let (warnings, should_stop_proxy) = match ProfileService::apply(&state, &id, scope) {
        Ok(result) => result,
        Err(error) => return err(error),
    };

    if should_stop_proxy {
        if let Err(error) = state.proxy_service.stop().await {
            log::warn!("切换项目后停止代理服务失败: {error}");
        }
    }

    for app in scope.apps() {
        let provider_id = crate::settings::get_effective_current_provider(&state.db, app)
            .ok()
            .flatten()
            .unwrap_or_default();
        let _ = ws_state.tx.send(json!({
            "event": "provider.switched",
            "data": { "app": app.as_str(), "id": provider_id }
        }));
    }
    let _ = ws_state.tx.send(json!({
        "event": "profile.applied",
        "data": { "profileId": id, "scope": scope.as_str() }
    }));

    ok(warnings)
}
