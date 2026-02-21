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
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::services::ServeDir;
use tracing_subscriber;

use crate::game::GameState;
use crate::websocket::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("🎲 Daggerheart VTT Server - Phase 1");
    tracing::info!("====================================");

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
    tracing::info!("🖥️  TV View:     http://localhost:3000");
    tracing::info!("📱 Mobile View: http://localhost:3000/mobile");
    tracing::info!("🔌 WebSocket:   ws://localhost:3000/ws");
    tracing::info!("");
    tracing::info!("Press Ctrl+C to stop the server");

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}
