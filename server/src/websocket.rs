//! WebSocket connection handler

use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::game::SharedGameState;
use crate::protocol::{ClientMessage, PlayerInfo, ServerMessage};

/// Broadcast channel for server messages
pub type Broadcaster = broadcast::Sender<String>;

/// Application state passed to handlers
#[derive(Clone)]
pub struct AppState {
    pub game: SharedGameState,
    pub broadcaster: Broadcaster,
}

/// Handle WebSocket upgrade request
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to broadcasts
    let mut rx = state.broadcaster.subscribe();
    
    // Player ID for this connection
    let mut player_id: Option<Uuid> = None;
    
    // Spawn task to forward broadcasts to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
    
    // Main receive loop
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Parse client message
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        match client_msg {
                            ClientMessage::PlayerJoin { name } => {
                                // Add player to game state
                                let player = {
                                    let mut game = state_clone.game.write().await;
                                    game.add_player(name.clone())
                                };
                                
                                player_id = Some(player.id);
                                
                                tracing::info!("Player joined: {} ({}) at {:?}", 
                                    player.name, player.id, player.position);
                                
                                // Broadcast player joined with position and color
                                let msg = ServerMessage::PlayerJoined {
                                    player_id: player.id.to_string(),
                                    name: player.name.clone(),
                                    position: player.position,
                                    color: player.color.clone(),
                                };
                                let _ = state_clone.broadcaster.send(msg.to_json());
                                
                                // Send current players list to all clients (including new one)
                                // This ensures new clients see existing players
                                let players = {
                                    let game = state_clone.game.read().await;
                                    game.get_players()
                                        .into_iter()
                                        .map(|p| PlayerInfo {
                                            player_id: p.id.to_string(),
                                            name: p.name,
                                            connected: p.connected,
                                            position: p.position,
                                            color: p.color,
                                        })
                                        .collect()
                                };
                                
                                let list_msg = ServerMessage::PlayersList { players };
                                let _ = state_clone.broadcaster.send(list_msg.to_json());
                            }
                            
                            ClientMessage::PlayerMove { x, y } => {
                                if let Some(pid) = player_id {
                                    // Update position in game state
                                    let position = crate::protocol::Position::new(x, y);
                                    let updated = {
                                        let mut game = state_clone.game.write().await;
                                        game.update_position(&pid, position)
                                    };
                                    
                                    if updated {
                                        tracing::debug!("Player {} moved to ({}, {})", pid, x, y);
                                        
                                        // Broadcast movement
                                        let msg = ServerMessage::PlayerMoved {
                                            player_id: pid.to_string(),
                                            position,
                                        };
                                        let _ = state_clone.broadcaster.send(msg.to_json());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Invalid message: {}", e);
                    }
                }
            }
        }
        
        player_id
    });
    
    // Wait for tasks to complete
    tokio::select! {
        pid = &mut recv_task => {
            send_task.abort();
            
            // Player disconnected - remove from game
            if let Ok(Some(pid)) = pid {
                let removed = {
                    let mut game = state.game.write().await;
                    game.remove_player(&pid)
                };
                
                if let Some(player) = removed {
                    tracing::info!("Player left: {} ({})", player.name, player.id);
                    
                    let msg = ServerMessage::PlayerLeft {
                        player_id: player.id.to_string(),
                        name: player.name,
                    };
                    let _ = state.broadcaster.send(msg.to_json());
                }
            }
        }
        _ = &mut send_task => {
            recv_task.abort();
        }
    }
}
