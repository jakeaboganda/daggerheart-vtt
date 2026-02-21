// Daggerheart VTT Server
// Phase 1: Foundation & Connection

use axum::{
    Router,
    routing::get,
};
use tower_http::services::ServeDir;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("🎲 Daggerheart VTT Server starting...");

    // Build application
    let app = Router::new()
        .route("/", get(|| async { "Daggerheart VTT - Phase 1" }))
        // Serve static files from client directory
        .nest_service("/static", ServeDir::new("../client"));

    // Determine server address
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    tracing::info!("✅ Server listening on http://{}", addr);
    tracing::info!("📱 Open http://localhost:3000 in your browser");

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}
