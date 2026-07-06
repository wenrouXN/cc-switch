//! Headless web server entry point.
//!
//! Started via `--headless` flag. Skips Tauri/GTK entirely.

use std::net::SocketAddr;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::{create_router, WsState};

pub fn run_headless() -> ! {
    crate::panic_hook::setup_panic_hook();

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
    let db = Arc::new(
        crate::database::Database::init().expect("Failed to initialize database"),
    );
    let app_state = Arc::new(AppState::new(db));

    let pw_path = crate::config::get_app_config_dir().join("web_password");
    let password = if pw_path.exists() {
        std::fs::read_to_string(&pw_path).unwrap_or_default().trim().to_string()
    } else {
        "admin".to_string()
    };
    println!();
    println!("============================================================");
    println!("[cc-switch] WEB_PASSWORD (default password for web login):");
    println!("Default: admin (must change on first login)");
    println!("============================================================");
    println!();

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
        axum::serve(listener, router)
            .await
            .expect("Server failed");
    });

    std::process::exit(0);
}
