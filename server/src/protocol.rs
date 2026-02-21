//! WebSocket message protocol

use serde::{Deserialize, Serialize};

/// Position on the map
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    /// Random position within map bounds
    pub fn random(width: f32, height: f32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen_range(50.0..width - 50.0),
            y: rng.gen_range(50.0..height - 50.0),
        }
    }
}

/// Client → Server messages
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    #[serde(rename = "player_join")]
    PlayerJoin { name: String },
    
    #[serde(rename = "player_move")]
    PlayerMove { x: f32, y: f32 },
}

/// Server → Client messages
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    #[serde(rename = "player_joined")]
    PlayerJoined {
        player_id: String,
        name: String,
        position: Position,
        color: String,
    },
    
    #[serde(rename = "player_left")]
    PlayerLeft {
        player_id: String,
        name: String,
    },
    
    #[serde(rename = "players_list")]
    PlayersList {
        players: Vec<PlayerInfo>,
    },
    
    #[serde(rename = "player_moved")]
    PlayerMoved {
        player_id: String,
        position: Position,
    },
    
    #[serde(rename = "error")]
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub player_id: String,
    pub name: String,
    pub connected: bool,
    pub position: Position,
    pub color: String,
}

impl ServerMessage {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_deserialize() {
        let json = r#"{"type":"player_join","payload":{"name":"Alice"}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        
        match msg {
            ClientMessage::PlayerJoin { name } => assert_eq!(name, "Alice"),
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_player_move_deserialize() {
        let json = r#"{"type":"player_move","payload":{"x":100.0,"y":200.0}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        
        match msg {
            ClientMessage::PlayerMove { x, y } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
            },
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_serialize() {
        let msg = ServerMessage::PlayerJoined {
            player_id: "123".to_string(),
            name: "Bob".to_string(),
            position: Position::new(100.0, 200.0),
            color: "#3b82f6".to_string(),
        };
        
        let json = msg.to_json();
        assert!(json.contains("player_joined"));
        assert!(json.contains("Bob"));
    }
}
