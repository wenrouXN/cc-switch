//! Headless web authentication and Bearer-session middleware.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use once_cell::sync::OnceCell;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::RwLock,
};
use subtle::ConstantTimeEq;
use uuid::Uuid;

const DEFAULT_PASSWORD: &str = "admin";

struct AuthState {
    password_path: PathBuf,
    password: RwLock<String>,
    sessions: RwLock<HashSet<String>>,
    no_auth: bool,
}

static AUTH_STATE: OnceCell<AuthState> = OnceCell::new();

pub fn default_password() -> &'static str {
    DEFAULT_PASSWORD
}

pub fn init(password_path: PathBuf, password: String, no_auth: bool) -> Result<(), String> {
    AUTH_STATE
        .set(AuthState {
            password_path,
            password: RwLock::new(password),
            sessions: RwLock::new(HashSet::new()),
            no_auth,
        })
        .map_err(|_| "Web authentication was initialized more than once".to_string())
}

fn state() -> Result<&'static AuthState, String> {
    AUTH_STATE
        .get()
        .ok_or_else(|| "Web authentication is not initialized".to_string())
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    let left_hash = Sha256::digest(left.as_bytes());
    let right_hash = Sha256::digest(right.as_bytes());
    left_hash.as_slice().ct_eq(right_hash.as_slice()).into()
}

fn issue_session(auth: &AuthState) -> Result<String, String> {
    let token = Uuid::new_v4().to_string();
    auth.sessions
        .write()
        .map_err(|_| "Authentication session lock is poisoned".to_string())?
        .insert(token.clone());
    Ok(token)
}

pub fn login(password: &str) -> Result<String, String> {
    let auth = state()?;
    if auth.no_auth {
        return Ok("no-auth".to_string());
    }
    let stored = auth
        .password
        .read()
        .map_err(|_| "Authentication password lock is poisoned".to_string())?;
    if !constant_time_eq(password, &stored) {
        return Err("Invalid web password".to_string());
    }
    drop(stored);
    issue_session(auth)
}

pub fn change_password(current_password: &str, new_password: &str) -> Result<String, String> {
    let auth = state()?;
    if auth.no_auth {
        return Err("Password changes are disabled when CC_SWITCH_WEB_NO_AUTH=1".to_string());
    }
    if new_password.trim().len() < 8 {
        return Err("New password must contain at least 8 characters".to_string());
    }

    let mut stored = auth
        .password
        .write()
        .map_err(|_| "Authentication password lock is poisoned".to_string())?;
    if !constant_time_eq(current_password, &stored) {
        return Err("Current web password is incorrect".to_string());
    }
    write_password_file(&auth.password_path, new_password.trim())?;
    *stored = new_password.trim().to_string();
    drop(stored);

    auth.sessions
        .write()
        .map_err(|_| "Authentication session lock is poisoned".to_string())?
        .clear();
    issue_session(auth)
}

fn write_password_file(path: &Path, password: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, format!("{password}\n")).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }
    std::fs::rename(&temp_path, path).map_err(|e| e.to_string())
}

pub fn authentication_disabled() -> bool {
    state().map(|auth| auth.no_auth).unwrap_or(false)
}

pub fn is_valid_token(token: &str) -> bool {
    let Ok(auth) = state() else {
        return false;
    };
    if auth.no_auth {
        return true;
    }
    auth.sessions
        .read()
        .map(|sessions| sessions.contains(token))
        .unwrap_or(false)
}

pub async fn auth_middleware(request: Request, next: Next) -> Response {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    if authentication_disabled() || token.is_some_and(is_valid_token) {
        return next.run(request).await;
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(json!({"success": false, "error": "Unauthorized"})),
    )
        .into_response()
}
