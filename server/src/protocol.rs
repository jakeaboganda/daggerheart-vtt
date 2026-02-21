//! WebSocket message protocol - Phase 5A: Refactored for Character/Connection architecture

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

/// Character info for listing (includes control status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterInfo {
    pub id: String,
    pub name: String,
    pub class: String,
    pub ancestry: String,
    pub position: Position,
    pub color: String,
    pub is_npc: bool,
    pub controlled_by_me: bool, // True if this connection controls this character
    pub controlled_by_other: bool, // True if another connection controls this character
}

/// Client → Server messages
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    /// Client connects (no name needed - connections are anonymous)
    #[serde(rename = "connect")]
    Connect,

    /// Client selects a character to control
    #[serde(rename = "select_character")]
    SelectCharacter { character_id: String },

    /// Client creates a new character
    #[serde(rename = "create_character")]
    CreateCharacter {
        name: String,
        class: String,
        ancestry: String,
        attributes: [i8; 6], // [agility, strength, finesse, instinct, presence, knowledge]
    },

    /// Move the controlled character
    #[serde(rename = "move_character")]
    MoveCharacter { x: f32, y: f32 },

    /// Roll duality dice for the controlled character
    #[serde(rename = "roll_duality")]
    RollDuality { modifier: i32, with_advantage: bool },

    /// Update resource for the controlled character
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
    /// Connection established, returns connection ID
    #[serde(rename = "connected")]
    Connected { connection_id: String },

    /// List of all characters in the game
    #[serde(rename = "characters_list")]
    CharactersList { characters: Vec<CharacterInfo> },

    /// Character was selected successfully
    #[serde(rename = "character_selected")]
    CharacterSelected {
        character_id: String,
        character: CharacterData,
    },

    /// A character spawned in the game
    #[serde(rename = "character_spawned")]
    CharacterSpawned {
        character_id: String,
        name: String,
        position: Position,
        color: String,
        is_npc: bool,
    },

    /// A character was removed from the game
    #[serde(rename = "character_removed")]
    CharacterRemoved {
        character_id: String,
        name: String,
    },

    /// A character moved
    #[serde(rename = "character_moved")]
    CharacterMoved {
        character_id: String,
        position: Position,
    },

    /// Character was created
    #[serde(rename = "character_created")]
    CharacterCreated {
        character_id: String,
        character: CharacterData,
    },

    /// Character was updated (resources, etc.)
    #[serde(rename = "character_updated")]
    CharacterUpdated {
        character_id: String,
        character: CharacterData,
    },

    /// Dice roll result
    #[serde(rename = "roll_result")]
    RollResult {
        character_id: String,
        character_name: String,
        roll: RollResult,
    },

    /// Error message
    #[serde(rename = "error")]
    Error { message: String },
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
    fn test_connect_deserialize() {
        let json = r#"{"type":"connect"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Connect => (),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_select_character_deserialize() {
        let json = r#"{"type":"select_character","payload":{"character_id":"abc-123"}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::SelectCharacter { character_id } => {
                assert_eq!(character_id, "abc-123");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_move_character_deserialize() {
        let json = r#"{"type":"move_character","payload":{"x":100.0,"y":200.0}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::MoveCharacter { x, y } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_create_character_deserialize() {
        let json = r#"{"type":"create_character","payload":{"name":"Theron","class":"Warrior","ancestry":"Human","attributes":[2,1,1,0,0,-1]}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::CreateCharacter {
                name,
                class,
                ancestry,
                attributes,
            } => {
                assert_eq!(name, "Theron");
                assert_eq!(class, "Warrior");
                assert_eq!(ancestry, "Human");
                assert_eq!(attributes, [2, 1, 1, 0, 0, -1]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roll_duality_deserialize() {
        let json = r#"{"type":"roll_duality","payload":{"modifier":2,"with_advantage":true}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::RollDuality {
                modifier,
                with_advantage,
            } => {
                assert_eq!(modifier, 2);
                assert!(with_advantage);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_serialize() {
        let msg = ServerMessage::CharacterSpawned {
            character_id: "char-123".to_string(),
            name: "Theron".to_string(),
            position: Position::new(100.0, 200.0),
            color: "#3b82f6".to_string(),
            is_npc: false,
        };

        let json = msg.to_json();
        assert!(json.contains("character_spawned"));
        assert!(json.contains("Theron"));
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
            hp: ResourceData {
                current: 6,
                maximum: 6,
            },
            stress: 0,
            hope: ResourceData {
                current: 5,
                maximum: 5,
            },
            evasion: 12,
        };

        let json = serde_json::to_string(&char_data).unwrap();
        assert!(json.contains("Theron"));
        assert!(json.contains("Warrior"));
    }

    #[test]
    fn test_position_random() {
        let pos = Position::random(800.0, 600.0);
        assert!(pos.x >= 50.0 && pos.x <= 750.0);
        assert!(pos.y >= 50.0 && pos.y <= 550.0);
    }

    #[test]
    fn test_character_info_serialize() {
        let info = CharacterInfo {
            id: "char-123".to_string(),
            name: "Theron".to_string(),
            class: "Warrior".to_string(),
            ancestry: "Human".to_string(),
            position: Position::new(100.0, 200.0),
            color: "#3b82f6".to_string(),
            is_npc: false,
            controlled_by_me: true,
            controlled_by_other: false,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("Theron"));
        assert!(json.contains("controlled_by_me"));
    }

    #[test]
    fn test_all_client_messages() {
        // Test all client message variants can be constructed
        let messages = vec![
            ClientMessage::Connect,
            ClientMessage::SelectCharacter {
                character_id: "char-1".to_string(),
            },
            ClientMessage::CreateCharacter {
                name: "Test".to_string(),
                class: "Warrior".to_string(),
                ancestry: "Human".to_string(),
                attributes: [2, 1, 1, 0, 0, -1],
            },
            ClientMessage::MoveCharacter { x: 100.0, y: 200.0 },
            ClientMessage::RollDuality {
                modifier: 0,
                with_advantage: false,
            },
            ClientMessage::UpdateResource {
                resource: "hp".to_string(),
                amount: -2,
            },
        ];

        assert_eq!(messages.len(), 6);
    }

    #[test]
    fn test_all_server_messages() {
        // Test all server message variants can be constructed
        let messages = vec![
            ServerMessage::Connected {
                connection_id: "conn-1".to_string(),
            },
            ServerMessage::CharactersList {
                characters: vec![],
            },
            ServerMessage::CharacterSelected {
                character_id: "char-1".to_string(),
                character: CharacterData {
                    name: "Test".to_string(),
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
                    hp: ResourceData {
                        current: 6,
                        maximum: 6,
                    },
                    stress: 0,
                    hope: ResourceData {
                        current: 5,
                        maximum: 5,
                    },
                    evasion: 12,
                },
            },
            ServerMessage::CharacterSpawned {
                character_id: "char-1".to_string(),
                name: "Test".to_string(),
                position: Position::new(100.0, 200.0),
                color: "#3b82f6".to_string(),
                is_npc: false,
            },
            ServerMessage::CharacterRemoved {
                character_id: "char-1".to_string(),
                name: "Test".to_string(),
            },
            ServerMessage::CharacterMoved {
                character_id: "char-1".to_string(),
                position: Position::new(100.0, 200.0),
            },
            ServerMessage::Error {
                message: "Test error".to_string(),
            },
        ];

        assert_eq!(messages.len(), 7);
    }
}
