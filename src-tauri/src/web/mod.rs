pub mod handlers;
pub mod middleware;
pub mod routes;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use crate::store::AppState;

// ── Embedded web assets (compiled into binary) ──────────────────────
#[derive(rust_embed::Embed)]
#[folder = "../dist"]
#[include = "*"]
struct EmbeddedAssets;

// ── Runtime web-dist path lookup (dev / external mode) ──────────────
fn get_web_dist_path() -> Option<PathBuf> {
    // 1. Sibling of executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("web-dist");
            if p.exists() {
                return Some(p);
            }
        }
    }
    // 2. Current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join("web-dist");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub struct WsState {
    pub tx: tokio::sync::broadcast::Sender<serde_json::Value>,
}

impl WsState {
    pub fn new(tx: tokio::sync::broadcast::Sender<serde_json::Value>) -> Self {
        Self { tx }
    }
}

pub fn create_router(app_state: Arc<AppState>, ws_state: Arc<WsState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false);

    let shared = (app_state, ws_state);

    let protected = Router::new()
        .nest("/proxy", routes::proxy::routes())
        .nest("/failover", routes::failover::routes())
        .nest("/providers", routes::providers::routes())
        .nest("/settings", routes::settings::routes())
        .nest("/mcp", routes::mcp::routes())
        .nest("/prompts", routes::prompts::routes())
        .nest("/skills", routes::skills::routes())
        .nest("/sessions", routes::sessions::routes())
        .nest("/hermes", routes::hermes::routes())
        .nest("/openclaw", routes::openclaw::routes())
        .nest("/usage", routes::usage::routes())
        .nest("/universal-providers", routes::providers::universal_routes())
        .layer(axum::middleware::from_fn(middleware::auth::auth_middleware))
        .with_state(shared.clone());

    let auth_routes = Router::new()
        .nest("/auth", routes::auth::routes())
        .with_state(shared.clone());

    let ws_routes = Router::new()
        .route("/ws", axum::routing::get(handlers::ws::ws_handler))
        .route("/api/ws", axum::routing::get(handlers::ws::ws_handler))
        .with_state(shared.clone());

    // ── Static asset serving: filesystem > embedded ─────────────
    let static_service: axum::routing::MethodRouter = if let Some(web_dist) = get_web_dist_path() {
        log::info!("Serving web assets from filesystem: {:?}", web_dist);
        let index = web_dist.join("index.html");
        axum::routing::any_service(ServeDir::new(&web_dist).fallback(ServeFile::new(&index)))
    } else {
        log::info!("Serving web assets from embedded binary (no external web-dist found)");
        axum::routing::any(serve_embedded)
    };

    Router::new()
        .nest("/api/v1", auth_routes.merge(protected))
        .route("/health", get(health_check))
        .merge(ws_routes)
        .fallback(static_service)
        .layer(cors)
}

// ── Embedded asset handler (SPA fallback to index.html) ─────────────
async fn serve_embedded(
    req: axum::extract::Request,
) -> impl IntoResponse {
    let path = req.uri().path().trim_start_matches('/');

    // Try exact file first
    if let Some(content) = EmbeddedAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", mime.as_ref())
            .body(axum::body::Body::from(content.data))
            .unwrap();
    }

    // SPA fallback: serve index.html for any non-API path
    if let Some(index) = EmbeddedAssets::get("index.html") {
        return axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(axum::body::Body::from(index.data))
            .unwrap();
    }

    (StatusCode::NOT_FOUND, "web assets not found").into_response()
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}
