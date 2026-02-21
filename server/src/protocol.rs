//! WebSocket message protocol

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Client → Server messages
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    #[serde(rename = "player_join")]
    PlayerJoin { name: String },
}

/// Server → Client messages
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    #[serde(rename = "player_joined")]
    PlayerJoined {
        player_id: String,
        name: String,
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
        }
    }

    #[test]
    fn test_server_message_serialize() {
        let msg = ServerMessage::PlayerJoined {
            player_id: "123".to_string(),
            name: "Bob".to_string(),
        };
        
        let json = msg.to_json();
        assert!(json.contains("player_joined"));
        assert!(json.contains("Bob"));
    }
}
