//! HTTP routes

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use qrcode::QrCode;
use serde_json::json;
use std::io::Cursor;
use std::net::UdpSocket;

use crate::websocket::AppState;

/// Get the local network IP address
fn get_local_ip() -> String {
    // Try to get local IP by connecting to a public DNS (doesn't actually send data)
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }

    // Fallback to localhost
    "localhost".to_string()
}

/// Root route - serve index.html
pub async fn index() -> Html<String> {
    let html = std::fs::read_to_string("../client/index.html")
        .unwrap_or_else(|_| "<h1>Error loading index.html</h1>".to_string());
    Html(html)
}

/// Mobile route - serve mobile.html
pub async fn mobile() -> Html<String> {
    let html = std::fs::read_to_string("../client/mobile.html")
        .unwrap_or_else(|_| "<h1>Error loading mobile.html</h1>".to_string());
    Html(html)
}

/// Generate QR code for connection URL
pub async fn qr_code() -> impl IntoResponse {
    // Get server address - use local IP instead of localhost
    let ip = get_local_ip();
    let url = format!("http://{}:3000/mobile", ip);

    tracing::info!("Generating QR code for: {}", url);

    // Generate QR code
    let code = QrCode::new(&url).unwrap();
    let image = code.render::<image::Luma<u8>>().build();

    // Convert to PNG bytes
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .unwrap();

    // Encode as base64 data URL
    let base64 = general_purpose::STANDARD.encode(&png_bytes);
    let data_url = format!("data:image/png;base64,{}", base64);

    Json(json!({
        "url": url,
        "qr_code": data_url
    }))
}

/// Get current game state
pub async fn game_state(State(state): State<AppState>) -> impl IntoResponse {
    let game = state.game.read().await;
    let players = game.get_players();

    Json(json!({
        "player_count": players.len(),
        "players": players
    }))
}
