//! Headless web login and password-management routes.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new().route("/login", post(login))
}

#[derive(Deserialize)]
struct LoginRequest {
    password: String,
}

async fn login(Json(request): Json<LoginRequest>) -> Response {
    match crate::web::middleware::auth::login(&request.password) {
        Ok(token) => Json(json!({
            "success": true,
            "data": {
                "token": token,
                "mustChange": request.password == crate::web::middleware::auth::default_password()
            }
        }))
        .into_response(),
        Err(error) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"success": false, "error": error})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

pub async fn change_password(Json(request): Json<ChangePasswordRequest>) -> Response {
    match crate::web::middleware::auth::change_password(
        &request.current_password,
        &request.new_password,
    ) {
        Ok(token) => Json(json!({"success": true, "data": {"token": token}})).into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"success": false, "error": error})),
        )
            .into_response(),
    }
}
