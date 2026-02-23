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

/// Dice roll result (legacy - kept for compatibility)
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

/// Roll target type for GM requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollTargetType {
    Specific, // One or more specific characters
    All,      // All player characters
    Npc,      // GM-controlled character
}

/// Type of roll being requested
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollType {
    Action,    // General action check (use attribute)
    Attack,    // Melee/ranged attack
    Spellcast, // Casting a spell
    Save,      // Reactive save
}

/// Success type of a roll
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuccessType {
    Failure,
    SuccessWithHope,
    SuccessWithFear,
    CriticalSuccess,
}

/// Which die is controlling the outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControllingDie {
    Hope,
    Fear,
    Tied, // Only when doubles
}

/// Detailed roll result for Phase 1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedRollResult {
    // The roll
    pub hope_die: u8,              // 1-12
    pub fear_die: u8,              // 1-12
    pub advantage_die: Option<u8>, // 1-6 if had advantage

    // Modifiers breakdown
    pub attribute_modifier: i8,
    pub proficiency_modifier: i8,
    pub situational_modifier: i8,
    pub hope_bonus: i8, // +2 if spent Hope via Experience
    pub total_modifier: i8,

    // Result
    pub total: u16,
    pub difficulty: u16,

    // Outcome
    pub success_type: SuccessType,
    pub controlling_die: ControllingDie,
    pub is_critical: bool,

    // Resource changes
    pub hope_change: i8, // +1, -1, or 0
    pub fear_change: i8, // +1 or 0
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

    /// GM requests a dice roll (Phase 1)
    #[serde(rename = "request_roll")]
    RequestRoll {
        target_type: RollTargetType,
        target_character_ids: Vec<String>,
        roll_type: RollType,
        attribute: Option<String>, // "agility", "strength", etc.
        difficulty: u16,
        context: String, // "Leap across the chasm"
        narrative_stakes: Option<String>,
        situational_modifier: i8,
        has_advantage: bool,
        is_combat: bool,
    },

    /// Player executes a requested roll (Phase 1)
    #[serde(rename = "execute_roll")]
    ExecuteRoll {
        request_id: String,
        spend_hope_for_bonus: bool,
        chosen_experience: Option<String>,
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
    CharacterRemoved { character_id: String, name: String },

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

    /// Dice roll result (legacy)
    #[serde(rename = "roll_result")]
    RollResult {
        character_id: String,
        character_name: String,
        roll: RollResult,
    },

    /// Roll requested by GM (Phase 1)
    #[serde(rename = "roll_requested")]
    RollRequested {
        request_id: String,
        roll_type: RollType,
        attribute: Option<String>,
        difficulty: u16,
        context: String,
        narrative_stakes: Option<String>,
        base_modifier: i8,
        situational_modifier: i8,
        total_modifier: i8,
        has_advantage: bool,
        your_attribute_value: i8,
        your_proficiency: i8,
        can_spend_hope: bool,
        experiences: Vec<String>,
    },

    /// Detailed roll result (Phase 1)
    #[serde(rename = "detailed_roll_result")]
    DetailedRollResult {
        request_id: String,
        character_id: String,
        character_name: String,
        roll_type: RollType,
        context: String,
        roll_details: DetailedRollResult,
        outcome_description: String,
        new_hope: u8,
        new_fear: u8,
    },

    /// Roll request status (GM-only, Phase 1)
    #[serde(rename = "roll_request_status")]
    RollRequestStatus {
        request_id: String,
        pending_characters: Vec<String>,
        completed_characters: Vec<String>,
    },
    
    /// Game event (for event log)
    #[serde(rename = "game_event")]
    GameEvent {
        timestamp: String,
        event_type: String,
        message: String,
        character_name: Option<String>,
        details: Option<String>,
    },
    
    /// Event log history
    #[serde(rename = "event_log")]
    EventLog {
        events: Vec<GameEventData>,
    },

    /// Error message
    #[serde(rename = "error")]
    Error { message: String },
}

/// Game event data for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEventData {
    pub timestamp: String,
    pub event_type: String,
    pub message: String,
    pub character_name: Option<String>,
    pub details: Option<String>,
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
            ClientMessage::RequestRoll {
                target_type: RollTargetType::Specific,
                target_character_ids: vec!["char-1".to_string()],
                roll_type: RollType::Action,
                attribute: Some("agility".to_string()),
                difficulty: 14,
                context: "Leap across chasm".to_string(),
                narrative_stakes: None,
                situational_modifier: 0,
                has_advantage: false,
                is_combat: false,
            },
            ClientMessage::ExecuteRoll {
                request_id: "req-1".to_string(),
                spend_hope_for_bonus: false,
                chosen_experience: None,
            },
        ];

        assert_eq!(messages.len(), 8);
    }

    #[test]
    fn test_all_server_messages() {
        // Test all server message variants can be constructed
        let messages = vec![
            ServerMessage::Connected {
                connection_id: "conn-1".to_string(),
            },
            ServerMessage::CharactersList { characters: vec![] },
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

    // Phase 1: GM-Initiated Dice Rolls Tests

    #[test]
    fn test_request_roll_deserialize() {
        let json = r#"{
            "type":"request_roll",
            "payload":{
                "target_type":"specific",
                "target_character_ids":["char-123"],
                "roll_type":"action",
                "attribute":"agility",
                "difficulty":14,
                "context":"Leap across chasm",
                "narrative_stakes":null,
                "situational_modifier":0,
                "has_advantage":false,
                "is_combat":false
            }
        }"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::RequestRoll {
                target_type,
                difficulty,
                context,
                ..
            } => {
                assert!(matches!(target_type, RollTargetType::Specific));
                assert_eq!(difficulty, 14);
                assert_eq!(context, "Leap across chasm");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_execute_roll_deserialize() {
        let json = r#"{
            "type":"execute_roll",
            "payload":{
                "request_id":"req-123",
                "spend_hope_for_bonus":true,
                "chosen_experience":"Former acrobat"
            }
        }"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::ExecuteRoll {
                request_id,
                spend_hope_for_bonus,
                chosen_experience,
            } => {
                assert_eq!(request_id, "req-123");
                assert!(spend_hope_for_bonus);
                assert_eq!(chosen_experience, Some("Former acrobat".to_string()));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_success_type_serialization() {
        let success = SuccessType::CriticalSuccess;
        let json = serde_json::to_string(&success).unwrap();
        assert_eq!(json, r#""critical_success""#);

        let loaded: SuccessType = serde_json::from_str(&json).unwrap();
        assert_eq!(success, loaded);
    }

    #[test]
    fn test_controlling_die_serialization() {
        let die = ControllingDie::Hope;
        let json = serde_json::to_string(&die).unwrap();
        assert_eq!(json, r#""hope""#);

        let loaded: ControllingDie = serde_json::from_str(&json).unwrap();
        assert_eq!(die, loaded);
    }

    #[test]
    fn test_roll_type_serialization() {
        let roll_type = RollType::Action;
        let json = serde_json::to_string(&roll_type).unwrap();
        assert_eq!(json, r#""action""#);

        let loaded: RollType = serde_json::from_str(&json).unwrap();
        assert!(matches!(loaded, RollType::Action));
    }
}
