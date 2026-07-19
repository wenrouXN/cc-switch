//! Headless web server entry point.
//!
//! Started via `--headless` flag. Skips Tauri/GTK entirely.

use std::net::SocketAddr;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::{create_router, WsState};

pub fn run_headless() -> ! {
    crate::panic_hook::setup_panic_hook();

    // Headless skips Tauri `.setup()`, which is where the desktop path installs the
    // rustls CryptoProvider. Without this, the first HTTPS outbound (Claude
    // /v1/messages forward via tokio-rustls) panics and the client sees an empty reply.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let port: u16 = std::env::var("CC_SWITCH_WEB_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let bind_all = std::env::var("CC_SWITCH_WEB_BIND_ALL")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let bind_addr: [u8; 4] = if bind_all {
        [0, 0, 0, 0]
    } else {
        [127, 0, 0, 1]
    };
    let addr = SocketAddr::from((bind_addr, port));

    // Build the full desktop AppState (Database + ProxyService)
    // Database::init() uses the default ~/.cc-switch/ path
    let db = Arc::new(crate::database::Database::init().expect("Failed to initialize database"));
    let app_state = Arc::new(AppState::new(db));

    let pw_path = crate::config::get_app_config_dir().join("web_password");
    let password = if pw_path.exists() {
        let stored = std::fs::read_to_string(&pw_path)
            .expect("Failed to read web_password")
            .trim()
            .to_string();
        if stored.is_empty() {
            crate::web::middleware::auth::default_password().to_string()
        } else {
            stored
        }
    } else {
        crate::web::middleware::auth::default_password().to_string()
    };
    let no_auth = std::env::var("CC_SWITCH_WEB_NO_AUTH")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if bind_all && !no_auth && password == crate::web::middleware::auth::default_password() {
        panic!(
            "Refusing CC_SWITCH_WEB_BIND_ALL=true with the default web password. \
             Set a non-default password in {} before exposing the server.",
            pw_path.display()
        );
    }
    if no_auth {
        eprintln!(
            "[cc-switch] WARNING: CC_SWITCH_WEB_NO_AUTH=1 disables HTTP authentication; use only for local development."
        );
    }
    crate::web::middleware::auth::init(pw_path, password, no_auth)
        .expect("Failed to initialize web authentication");

    let (ws_state, _rx) = {
        let (tx, rx) = tokio::sync::broadcast::channel(100);
        (Arc::new(WsState::new(tx)), rx)
    };
    let router = create_router(app_state, ws_state);

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind");
        println!("Listening on {}", addr);
        axum::serve(listener, router).await.expect("Server failed");
    });

    std::process::exit(0);
}
