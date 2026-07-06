pub mod handlers;
pub mod middleware;
pub mod routes;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use crate::store::AppState;

pub struct WsState {
    pub tx: tokio::sync::broadcast::Sender<serde_json::Value>,
}

impl WsState {
    pub fn new(tx: tokio::sync::broadcast::Sender<serde_json::Value>) -> Self {
        Self { tx }
    }
}

fn get_web_dist_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("web-dist");
            if p.exists() {
                return p;
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join("web-dist");
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("web-dist")
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

    let web_dist = get_web_dist_path();
    let index = web_dist.join("index.html");
    log::info!("Serving web assets from {:?}", web_dist);

    let static_service = ServeDir::new(&web_dist).fallback(ServeFile::new(&index));

    Router::new()
        .nest("/api/v1", auth_routes.merge(protected))
        .route("/health", get(health_check))
        .merge(ws_routes)
        .fallback_service(static_service)
        .layer(cors)
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}
