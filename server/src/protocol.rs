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

/// Character data (simplified for protocol)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    pub name: String,
    pub class: String,
    pub ancestry: String,
    pub attributes: AttributesData,
    pub hp: ResourceData,
    pub stress: i32,
    pub hope: ResourceData,
    pub evasion: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributesData {
    pub agility: i8,
    pub strength: i8,
    pub finesse: i8,
    pub instinct: i8,
    pub presence: i8,
    pub knowledge: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    pub current: i32,
    pub maximum: i32,
}

/// Dice roll result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollResult {
    pub hope: i32,
    pub fear: i32,
    pub modifier: i32,
    pub total: i32,
    pub controlling_die: String, // "Hope" or "Fear"
    pub is_critical: bool,
    pub is_success: bool,
}

/// Client → Server messages
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    #[serde(rename = "player_join")]
    PlayerJoin { name: String },
    
    #[serde(rename = "player_move")]
    PlayerMove { x: f32, y: f32 },
    
    #[serde(rename = "create_character")]
    CreateCharacter {
        name: String,
        class: String,
        ancestry: String,
        attributes: [i8; 6], // [agility, strength, finesse, instinct, presence, knowledge]
    },
    
    #[serde(rename = "roll_duality")]
    RollDuality {
        modifier: i32,
        with_advantage: bool,
    },
    
    #[serde(rename = "update_resource")]
    UpdateResource {
        resource: String, // "hp", "stress", or "hope"
        amount: i32,      // positive = gain, negative = lose
    },
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
    
    #[serde(rename = "character_created")]
    CharacterCreated {
        player_id: String,
        character: CharacterData,
    },
    
    #[serde(rename = "character_updated")]
    CharacterUpdated {
        player_id: String,
        character: CharacterData,
    },
    
    #[serde(rename = "roll_result")]
    RollResult {
        player_id: String,
        player_name: String,
        roll: RollResult,
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
    pub has_character: bool,
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
    fn test_create_character_deserialize() {
        let json = r#"{"type":"create_character","payload":{"name":"Theron","class":"Warrior","ancestry":"Human","attributes":[2,1,1,0,0,-1]}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        
        match msg {
            ClientMessage::CreateCharacter { name, class, ancestry, attributes } => {
                assert_eq!(name, "Theron");
                assert_eq!(class, "Warrior");
                assert_eq!(ancestry, "Human");
                assert_eq!(attributes, [2, 1, 1, 0, 0, -1]);
            },
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_roll_duality_deserialize() {
        let json = r#"{"type":"roll_duality","payload":{"modifier":2,"with_advantage":true}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        
        match msg {
            ClientMessage::RollDuality { modifier, with_advantage } => {
                assert_eq!(modifier, 2);
                assert!(with_advantage);
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
    
    #[test]
    fn test_character_data_serialize() {
        let char_data = CharacterData {
            name: "Theron".to_string(),
            class: "Warrior".to_string(),
            ancestry: "Human".to_string(),
            attributes: AttributesData {
                agility: 2,
                strength: 1,
                finesse: 1,
                instinct: 0,
                presence: 0,
                knowledge: -1,
            },
            hp: ResourceData { current: 6, maximum: 6 },
            stress: 0,
            hope: ResourceData { current: 5, maximum: 5 },
            evasion: 12,
        };
        
        let json = serde_json::to_string(&char_data).unwrap();
        assert!(json.contains("Theron"));
        assert!(json.contains("Warrior"));
    }
}
