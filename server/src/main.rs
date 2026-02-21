// Daggerheart VTT Server
// Phase 1: Foundation & Connection

mod game;
mod protocol;
mod routes;
mod websocket;

use axum::{
    routing::{any, get},
    Router,
};
use std::net::UdpSocket;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::services::ServeDir;
use tracing_subscriber;

use crate::game::GameState;
use crate::websocket::AppState;

/// Get the local network IP address
fn get_local_ip() -> String {
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    return addr.ip().to_string();
                }
            }
        }
        Err(_) => {}
    }
    "localhost".to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("🎲 Daggerheart VTT Server - Phase 1");
    tracing::info!("====================================");

    // Get local IP
    let local_ip = get_local_ip();

    // Create game state
    let game_state = Arc::new(RwLock::new(GameState::new()));
    
    // Create broadcast channel for WebSocket messages
    let (broadcaster, _) = broadcast::channel::<String>(100);
    
    let app_state = AppState {
        game: game_state,
        broadcaster,
    };

    // Build application routes
    let app = Router::new()
        .route("/", get(routes::index))
        .route("/mobile", get(routes::mobile))
        .route("/api/qr-code", get(routes::qr_code))
        .route("/api/game-state", get(routes::game_state))
        .route("/ws", any(websocket::websocket_handler))
        // Serve static files from client directory
        .nest_service("/static", ServeDir::new("../client"))
        .with_state(app_state);

    // Determine server address
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    tracing::info!("✅ Server listening on http://{}", addr);
    tracing::info!("");
    tracing::info!("📡 Network Access:");
    tracing::info!("   Local IP:    http://{}:3000", local_ip);
    tracing::info!("   Localhost:   http://localhost:3000");
    tracing::info!("");
    tracing::info!("🖥️  TV View:     http://{}:3000", local_ip);
    tracing::info!("📱 Mobile View: http://{}:3000/mobile", local_ip);
    tracing::info!("🔌 WebSocket:   ws://{}:3000/ws", local_ip);
    tracing::info!("");
    tracing::info!("💡 Scan the QR code on TV to join from your phone!");
    tracing::info!("Press Ctrl+C to stop the server");

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}
