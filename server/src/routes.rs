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

use crate::save::SavedSession;
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
    let characters = game.get_characters();

    Json(json!({
        "character_count": characters.len(),
        "connection_count": game.connection_count(),
        "characters": characters
    }))
}

/// GM view - serve gm.html
pub async fn gm() -> Html<String> {
    let html = std::fs::read_to_string("../client/gm.html")
        .unwrap_or_else(|_| "<h1>Error loading gm.html</h1>".to_string());
    Html(html)
}

/// Save current game state
pub async fn save_game(State(state): State<AppState>) -> Json<serde_json::Value> {
    let game = state.game.read().await;
    let session = SavedSession::from_game_state(&game, "Manual Save".to_string());

    match session.save_to_file() {
        Ok(path) => Json(json!({
            "success": true,
            "path": path.display().to_string(),
            "session": session
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": e
        })),
    }
}

/// List all saved sessions
pub async fn list_saves() -> Json<serde_json::Value> {
    match SavedSession::list_saves() {
        Ok(saves) => {
            let saves_data: Vec<_> = saves
                .into_iter()
                .map(|(path, name, timestamp)| {
                    json!({
                        "path": path.display().to_string(),
                        "name": name,
                        "timestamp": timestamp.to_rfc3339()
                    })
                })
                .collect();

            Json(json!({
                "success": true,
                "saves": saves_data
            }))
        }
        Err(e) => Json(json!({
            "success": false,
            "error": e
        })),
    }
}

/// Load a saved session
pub async fn load_game(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let path_str = match payload.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => {
            return Json(json!({
                "success": false,
                "error": "Missing 'path' field"
            }))
        }
    };

    let path = std::path::Path::new(path_str);

    match SavedSession::load_from_file(path) {
        Ok(session) => {
            // Apply to game state
            let mut game = state.game.write().await;

            if let Err(e) = session.apply_to_game(&mut game) {
                return Json(json!({
                    "success": false,
                    "error": format!("Failed to apply session: {}", e)
                }));
            }

            // Notify all connected clients to refresh
            let msg = crate::protocol::ServerMessage::Error {
                message: "Session loaded. Please refresh your browser.".to_string(),
            };
            let _ = state.broadcaster.send(msg.to_json());

            Json(json!({
                "success": true,
                "session": session
            }))
        }
        Err(e) => Json(json!({
            "success": false,
            "error": e
        })),
    }
}
