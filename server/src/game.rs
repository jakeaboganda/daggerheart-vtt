//! Game state management - Phase 5A: Refactored architecture
//!
//! NEW ARCHITECTURE:
//! - Connection: Ephemeral WebSocket session (disappears on disconnect/refresh)
//! - Character: Persistent game entity (survives restarts, can be controlled by any connection)
//! - Control mapping: Connection → Character relationship

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use daggerheart_engine::{
    character::{Ancestry, Attributes, Class},
    combat::{HitPoints, Hope, Stress},
    core::dice::duality::DualityRoll,
};

use crate::protocol::{AttributesData, CharacterData, Position, ResourceData, RollResult};

/// Map dimensions
pub const MAP_WIDTH: f32 = 800.0;
pub const MAP_HEIGHT: f32 = 600.0;

/// Character color palette
const CHARACTER_COLORS: &[&str] = &[
    "#3b82f6", // Blue
    "#10b981", // Green
    "#f59e0b", // Orange
    "#ef4444", // Red
    "#8b5cf6", // Purple
    "#ec4899", // Pink
    "#14b8a6", // Teal
    "#f97316", // Dark Orange
];

/// A character in the game (persistent entity)
#[derive(Debug, Clone, Serialize)]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub class: Class,
    pub ancestry: Ancestry,
    pub attributes: Attributes,
    #[serde(skip)]
    pub hp: HitPoints,
    #[serde(skip)]
    pub stress: Stress,
    #[serde(skip)]
    pub hope: Hope,
    pub evasion: i32,
    pub position: Position,
    pub color: String,
    pub is_npc: bool,
    
    // Serializable resource values (for save/load)
    pub hp_current: u8,
    pub hp_max: u8,
    pub stress_current: u8,
    pub hope_current: u8,
    pub hope_max: u8,
}

impl Character {
    /// Create new player character
    pub fn new(
        name: String,
        class: Class,
        ancestry: Ancestry,
        attributes: Attributes,
        position: Position,
        color: String,
    ) -> Self {
        // Calculate HP
        let base_hp = class.starting_hp() as i32;
        let hp_modifier = ancestry.hp_modifier();
        let max_hp = (base_hp + hp_modifier as i32).max(1) as u8;

        // Calculate Evasion
        let base_evasion = class.starting_evasion() as i32;
        let evasion_modifier = ancestry.evasion_modifier();
        let evasion = base_evasion + evasion_modifier as i32;

        let hp = HitPoints::new(max_hp);
        let stress = Stress::new();
        let hope = Hope::new(5); // Standard starting Hope

        Self {
            id: Uuid::new_v4(),
            name,
            class,
            ancestry,
            attributes,
            hp,
            stress,
            hope,
            evasion,
            position,
            color,
            is_npc: false,
            hp_current: max_hp,
            hp_max: max_hp,
            stress_current: 0,
            hope_current: 5,
            hope_max: 5,
        }
    }

    /// Create NPC character
    pub fn new_npc(
        name: String,
        class: Class,
        ancestry: Ancestry,
        attributes: Attributes,
        position: Position,
        color: String,
        hp_max: u8,
    ) -> Self {
        let hp = HitPoints::new(hp_max);
        let stress = Stress::new();
        let hope = Hope::new(0); // NPCs typically don't have Hope

        // Calculate Evasion
        let base_evasion = class.starting_evasion() as i32;
        let evasion_modifier = ancestry.evasion_modifier();
        let evasion = base_evasion + evasion_modifier as i32;

        Self {
            id: Uuid::new_v4(),
            name,
            class,
            ancestry,
            attributes,
            hp,
            stress,
            hope,
            evasion,
            position,
            color,
            is_npc: true,
            hp_current: hp_max,
            hp_max,
            stress_current: 0,
            hope_current: 0,
            hope_max: 0,
        }
    }

    /// Sync serializable fields with runtime resources
    pub fn sync_resources(&mut self) {
        self.hp_current = self.hp.current;
        self.hp_max = self.hp.maximum;
        self.stress_current = self.stress.current;
        self.hope_current = self.hope.current;
        self.hope_max = self.hope.maximum;
    }

    /// Restore runtime resources from serializable fields
    pub fn restore_resources(&mut self) {
        self.hp = HitPoints::new(self.hp_max);
        if self.hp_current < self.hp_max {
            let damage = self.hp_max - self.hp_current;
            self.hp.take_damage(damage);
        }

        self.stress = Stress::new();
        self.stress.gain(self.stress_current);

        self.hope = Hope::new(self.hope_max);
        if self.hope_current < self.hope_max {
            let spent = self.hope_max - self.hope_current;
            let _ = self.hope.spend(spent);
        }
    }

    /// Convert to protocol CharacterData
    pub fn to_data(&self) -> CharacterData {
        CharacterData {
            name: self.name.clone(),
            class: self.class.to_string(),
            ancestry: self.ancestry.to_string(),
            attributes: AttributesData {
                agility: self.attributes.agility,
                strength: self.attributes.strength,
                finesse: self.attributes.finesse,
                instinct: self.attributes.instinct,
                presence: self.attributes.presence,
                knowledge: self.attributes.knowledge,
            },
            hp: ResourceData {
                current: self.hp.current as i32,
                maximum: self.hp.maximum as i32,
            },
            stress: self.stress.current as i32,
            hope: ResourceData {
                current: self.hope.current as i32,
                maximum: self.hope.maximum as i32,
            },
            evasion: self.evasion,
        }
    }
}

/// A WebSocket connection (ephemeral)
#[derive(Debug, Clone)]
pub struct Connection {
    pub id: Uuid,
    pub connected_at: std::time::SystemTime,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            connected_at: std::time::SystemTime::now(),
        }
    }
}

/// The global game state
#[derive(Debug, Clone, Default)]
pub struct GameState {
    /// All characters in the game (persistent)
    pub characters: HashMap<Uuid, Character>,
    
    /// Active WebSocket connections (ephemeral)
    pub connections: HashMap<Uuid, Connection>,
    
    /// Which connection controls which character
    pub control_mapping: HashMap<Uuid, Uuid>, // connection_id -> character_id
    
    /// Color assignment index
    pub(crate) color_index: usize,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            connections: HashMap::new(),
            control_mapping: HashMap::new(),
            color_index: 0,
        }
    }

    /// Add a new connection
    pub fn add_connection(&mut self) -> Connection {
        let conn = Connection::new();
        self.connections.insert(conn.id, conn.clone());
        conn
    }

    /// Remove a connection and its control mapping
    pub fn remove_connection(&mut self, conn_id: &Uuid) -> Option<Connection> {
        self.control_mapping.remove(conn_id);
        self.connections.remove(conn_id)
    }

    /// Create a new character
    pub fn create_character(
        &mut self,
        name: String,
        class: Class,
        ancestry: Ancestry,
        attributes: Attributes,
    ) -> Character {
        let color = self.assign_color();
        let position = Position::random(MAP_WIDTH, MAP_HEIGHT);
        
        let character = Character::new(name, class, ancestry, attributes, position, color);
        self.characters.insert(character.id, character.clone());
        character
    }

    /// Select a character for a connection to control
    pub fn select_character(&mut self, conn_id: &Uuid, char_id: &Uuid) -> Result<(), String> {
        if !self.connections.contains_key(conn_id) {
            return Err("Connection not found".to_string());
        }
        
        if !self.characters.contains_key(char_id) {
            return Err("Character not found".to_string());
        }
        
        // Check if character is already controlled by another connection
        if let Some((controlling_conn_id, _)) = self.control_mapping
            .iter()
            .find(|(_, &controlled_char_id)| controlled_char_id == *char_id)
        {
            if controlling_conn_id != conn_id {
                return Err("Character already controlled by another connection".to_string());
            }
        }
        
        self.control_mapping.insert(*conn_id, *char_id);
        Ok(())
    }

    /// Get the character controlled by a connection
    pub fn get_controlled_character(&self, conn_id: &Uuid) -> Option<&Character> {
        let char_id = self.control_mapping.get(conn_id)?;
        self.characters.get(char_id)
    }

    /// Get mutable reference to controlled character
    pub fn get_controlled_character_mut(&mut self, conn_id: &Uuid) -> Option<&mut Character> {
        let char_id = *self.control_mapping.get(conn_id)?;
        self.characters.get_mut(&char_id)
    }

    /// Get character by ID
    pub fn get_character(&self, char_id: &Uuid) -> Option<&Character> {
        self.characters.get(char_id)
    }

    /// Get mutable character by ID
    pub fn get_character_mut(&mut self, char_id: &Uuid) -> Option<&mut Character> {
        self.characters.get_mut(char_id)
    }

    /// Update character position
    pub fn update_character_position(&mut self, char_id: &Uuid, position: Position) -> bool {
        if let Some(character) = self.characters.get_mut(char_id) {
            character.position = position;
            character.sync_resources(); // Sync resources whenever we modify character
            true
        } else {
            false
        }
    }

    /// Roll duality dice for a character
    pub fn roll_duality(&self, modifier: i32, with_advantage: bool) -> RollResult {
        let roll = DualityRoll::roll();

        let result = if with_advantage {
            roll.with_advantage()
        } else {
            roll.with_modifier(modifier as i8)
        };

        // Standard difficulty is 12 in Daggerheart
        const STANDARD_DIFFICULTY: u16 = 12;

        RollResult {
            hope: result.roll.hope as i32,
            fear: result.roll.fear as i32,
            modifier,
            total: result.total as i32,
            controlling_die: match result.controlling {
                daggerheart_engine::core::dice::duality::ControllingDie::Hope => "Hope".to_string(),
                daggerheart_engine::core::dice::duality::ControllingDie::Fear => "Fear".to_string(),
                daggerheart_engine::core::dice::duality::ControllingDie::Tied => "Tied".to_string(),
            },
            is_critical: result.is_critical,
            is_success: result.is_success(STANDARD_DIFFICULTY),
        }
    }

    /// Get all characters
    pub fn get_characters(&self) -> Vec<&Character> {
        self.characters.values().collect()
    }

    /// Get all player characters (non-NPCs)
    pub fn get_player_characters(&self) -> Vec<&Character> {
        self.characters
            .values()
            .filter(|c| !c.is_npc)
            .collect()
    }

    /// Get all NPCs
    pub fn get_npcs(&self) -> Vec<&Character> {
        self.characters
            .values()
            .filter(|c| c.is_npc)
            .collect()
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Get character count
    pub fn character_count(&self) -> usize {
        self.characters.len()
    }

    /// Assign a color from the palette (cycles through)
    fn assign_color(&mut self) -> String {
        let color = CHARACTER_COLORS[self.color_index % CHARACTER_COLORS.len()].to_string();
        self.color_index += 1;
        color
    }

    /// Sync all character resources (call before saving)
    pub fn sync_all_resources(&mut self) {
        for character in self.characters.values_mut() {
            character.sync_resources();
        }
    }

    /// Restore all character resources (call after loading)
    pub fn restore_all_resources(&mut self) {
        for character in self.characters.values_mut() {
            character.restore_resources();
        }
    }
}

/// Shared game state wrapped for concurrent access
pub type SharedGameState = Arc<RwLock<GameState>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_connection() {
        let mut state = GameState::new();
        let conn = state.add_connection();

        assert_eq!(state.connection_count(), 1);
        assert!(state.connections.contains_key(&conn.id));
    }

    #[test]
    fn test_remove_connection() {
        let mut state = GameState::new();
        let conn = state.add_connection();

        let removed = state.remove_connection(&conn.id);
        assert!(removed.is_some());
        assert_eq!(state.connection_count(), 0);
    }

    #[test]
    fn test_create_character() {
        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        
        let character = state.create_character(
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs,
        );

        assert_eq!(character.name, "Theron");
        assert_eq!(character.class, Class::Warrior);
        assert!(!character.is_npc);
        assert_eq!(state.character_count(), 1);
    }

    #[test]
    fn test_select_character() {
        let mut state = GameState::new();
        let conn = state.add_connection();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character = state.create_character(
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs,
        );

        let result = state.select_character(&conn.id, &character.id);
        assert!(result.is_ok());

        let controlled = state.get_controlled_character(&conn.id);
        assert!(controlled.is_some());
        assert_eq!(controlled.unwrap().name, "Theron");
    }

    #[test]
    fn test_select_character_already_controlled() {
        let mut state = GameState::new();
        let conn1 = state.add_connection();
        let conn2 = state.add_connection();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character = state.create_character(
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs,
        );

        // First connection controls character
        state.select_character(&conn1.id, &character.id).unwrap();

        // Second connection tries to control same character - should fail
        let result = state.select_character(&conn2.id, &character.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_character_position() {
        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character = state.create_character(
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs,
        );

        let new_pos = Position::new(100.0, 200.0);
        let updated = state.update_character_position(&character.id, new_pos);

        assert!(updated);
        let char = state.get_character(&character.id).unwrap();
        assert_eq!(char.position.x, 100.0);
        assert_eq!(char.position.y, 200.0);
    }

    #[test]
    fn test_connection_removal_clears_control() {
        let mut state = GameState::new();
        let conn = state.add_connection();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character = state.create_character(
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs,
        );

        state.select_character(&conn.id, &character.id).unwrap();
        assert!(state.control_mapping.contains_key(&conn.id));

        state.remove_connection(&conn.id);
        assert!(!state.control_mapping.contains_key(&conn.id));
        // Character should still exist
        assert!(state.characters.contains_key(&character.id));
    }

    #[test]
    fn test_get_player_characters_and_npcs() {
        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        
        // Create PC
        state.create_character(
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs.clone(),
        );

        // Create NPC
        let npc = Character::new_npc(
            "Goblin".to_string(),
            Class::Rogue,
            Ancestry::Goblin,
            attrs,
            Position::random(MAP_WIDTH, MAP_HEIGHT),
            "#ff0000".to_string(),
            10,
        );
        state.characters.insert(npc.id, npc);

        assert_eq!(state.get_player_characters().len(), 1);
        assert_eq!(state.get_npcs().len(), 1);
        assert_eq!(state.character_count(), 2);
    }

    #[test]
    fn test_color_assignment() {
        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        
        let c1 = state.create_character("C1".to_string(), Class::Warrior, Ancestry::Human, attrs.clone());
        let c2 = state.create_character("C2".to_string(), Class::Warrior, Ancestry::Human, attrs.clone());
        let c3 = state.create_character("C3".to_string(), Class::Warrior, Ancestry::Human, attrs);

        // Should assign different colors
        assert_ne!(c1.color, c2.color);
        assert_ne!(c2.color, c3.color);
    }

    #[test]
    fn test_roll_duality() {
        let state = GameState::new();
        let result = state.roll_duality(2, false);

        // Should have valid values
        assert!(result.hope >= 1 && result.hope <= 12);
        assert!(result.fear >= 1 && result.fear <= 12);
        assert_eq!(result.modifier, 2);
        assert!(result.controlling_die == "Hope" 
            || result.controlling_die == "Fear" 
            || result.controlling_die == "Tied");
    }

    #[test]
    fn test_resource_sync_and_restore() {
        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character = state.create_character(
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs,
        );

        // Modify resources
        let char_mut = state.get_character_mut(&character.id).unwrap();
        char_mut.hp.take_damage(3);
        char_mut.stress.gain(2);
        let _ = char_mut.hope.spend(1);

        // Sync to serializable fields
        let char_mut = state.get_character_mut(&character.id).unwrap();
        char_mut.sync_resources();

        let hp_current = char_mut.hp_current;
        let stress_current = char_mut.stress_current;
        let hope_current = char_mut.hope_current;

        // Restore from serializable fields
        let char_mut = state.get_character_mut(&character.id).unwrap();
        char_mut.restore_resources();

        assert_eq!(char_mut.hp.current, hp_current);
        assert_eq!(char_mut.stress.current, stress_current);
        assert_eq!(char_mut.hope.current, hope_current);
    }
}
