//! Game state management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use daggerheart_engine::{
    character::{Ancestry, Attributes, Class},
    combat::{HitPoints, Hope, Stress},
    core::dice::duality::{DualityRoll},
};

use crate::protocol::{AttributesData, CharacterData, Position, ResourceData, RollResult};

/// Map dimensions
pub const MAP_WIDTH: f32 = 800.0;
pub const MAP_HEIGHT: f32 = 600.0;

/// Player color palette
const PLAYER_COLORS: &[&str] = &[
    "#3b82f6", // Blue
    "#10b981", // Green
    "#f59e0b", // Orange
    "#ef4444", // Red
    "#8b5cf6", // Purple
    "#ec4899", // Pink
    "#14b8a6", // Teal
    "#f97316", // Dark Orange
];

/// A character in the game
#[derive(Debug, Clone)]
pub struct Character {
    pub name: String,
    pub class: Class,
    pub ancestry: Ancestry,
    pub attributes: Attributes,
    pub hp: HitPoints,
    pub stress: Stress,
    pub hope: Hope,
    pub evasion: i32,
}

impl Character {
    /// Create new character
    pub fn new(
        name: String,
        class: Class,
        ancestry: Ancestry,
        attributes: Attributes,
    ) -> Self {
        // Calculate HP
        let base_hp = class.starting_hp() as i32;
        let hp_modifier = ancestry.hp_modifier();
        let max_hp = (base_hp + hp_modifier as i32).max(1) as u8;
        
        // Calculate Evasion
        let base_evasion = class.starting_evasion() as i32;
        let evasion_modifier = ancestry.evasion_modifier();
        let evasion = base_evasion + evasion_modifier as i32;
        
        Self {
            name,
            class,
            ancestry,
            attributes,
            hp: HitPoints::new(max_hp),
            stress: Stress::new(),
            hope: Hope::new(5), // Standard starting Hope
            evasion,
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

/// A connected player
#[derive(Debug, Clone, Serialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub connected: bool,
    pub position: Position,
    pub color: String,
    #[serde(skip)]
    pub character: Option<Character>,
}

/// The global game state
#[derive(Debug, Clone, Default)]
pub struct GameState {
    pub players: HashMap<Uuid, Player>,
    color_index: usize,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            color_index: 0,
        }
    }

    /// Add a new player with random position and assigned color
    pub fn add_player(&mut self, name: String) -> Player {
        let color = self.assign_color();
        let position = Position::random(MAP_WIDTH, MAP_HEIGHT);
        
        let player = Player {
            id: Uuid::new_v4(),
            name,
            connected: true,
            position,
            color,
            character: None,
        };
        
        self.players.insert(player.id, player.clone());
        player
    }

    /// Remove a player
    pub fn remove_player(&mut self, player_id: &Uuid) -> Option<Player> {
        self.players.remove(player_id)
    }
    
    /// Update player position
    pub fn update_position(&mut self, player_id: &Uuid, position: Position) -> bool {
        if let Some(player) = self.players.get_mut(player_id) {
            player.position = position;
            true
        } else {
            false
        }
    }
    
    /// Create character for a player
    pub fn create_character(
        &mut self,
        player_id: &Uuid,
        name: String,
        class: Class,
        ancestry: Ancestry,
        attributes: Attributes,
    ) -> Result<Character, String> {
        if let Some(player) = self.players.get_mut(player_id) {
            let character = Character::new(name, class, ancestry, attributes);
            player.character = Some(character.clone());
            Ok(character)
        } else {
            Err("Player not found".to_string())
        }
    }
    
    /// Get character for a player
    pub fn get_character(&self, player_id: &Uuid) -> Option<&Character> {
        self.players.get(player_id)?.character.as_ref()
    }
    
    /// Get mutable character for a player
    pub fn get_character_mut(&mut self, player_id: &Uuid) -> Option<&mut Character> {
        self.players.get_mut(player_id)?.character.as_mut()
    }
    
    /// Get display name for a player (character name if available, otherwise player name)
    pub fn get_display_name(&self, player_id: &Uuid) -> Option<String> {
        let player = self.players.get(player_id)?;
        Some(
            player.character
                .as_ref()
                .map(|c| c.name.clone())
                .unwrap_or_else(|| player.name.clone())
        )
    }
    
    /// Roll duality dice for a player
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

    /// Get all connected players
    pub fn get_players(&self) -> Vec<Player> {
        self.players.values().cloned().collect()
    }

    /// Get player count
    pub fn player_count(&self) -> usize {
        self.players.len()
    }
    
    /// Assign a color from the palette (cycles through)
    fn assign_color(&mut self) -> String {
        let color = PLAYER_COLORS[self.color_index % PLAYER_COLORS.len()].to_string();
        self.color_index += 1;
        color
    }
}

/// Shared game state wrapped for concurrent access
pub type SharedGameState = Arc<RwLock<GameState>>;

#[cfg(test)]
mod tests {
    use super::*;
    use daggerheart_engine::character::AttributeType;

    #[test]
    fn test_add_player() {
        let mut state = GameState::new();
        let player = state.add_player("Alice".to_string());
        
        assert_eq!(player.name, "Alice");
        assert_eq!(state.player_count(), 1);
        assert!(!player.color.is_empty());
        assert!(player.character.is_none());
    }

    #[test]
    fn test_remove_player() {
        let mut state = GameState::new();
        let player = state.add_player("Bob".to_string());
        
        let removed = state.remove_player(&player.id);
        assert!(removed.is_some());
        assert_eq!(state.player_count(), 0);
    }

    #[test]
    fn test_get_players() {
        let mut state = GameState::new();
        state.add_player("Alice".to_string());
        state.add_player("Bob".to_string());
        
        let players = state.get_players();
        assert_eq!(players.len(), 2);
    }
    
    #[test]
    fn test_update_position() {
        let mut state = GameState::new();
        let player = state.add_player("Alice".to_string());
        
        let new_pos = Position::new(100.0, 200.0);
        let updated = state.update_position(&player.id, new_pos);
        
        assert!(updated);
        let players = state.get_players();
        assert_eq!(players[0].position.x, 100.0);
        assert_eq!(players[0].position.y, 200.0);
    }
    
    #[test]
    fn test_color_assignment() {
        let mut state = GameState::new();
        let p1 = state.add_player("P1".to_string());
        let p2 = state.add_player("P2".to_string());
        let p3 = state.add_player("P3".to_string());
        
        // Should assign different colors
        assert_ne!(p1.color, p2.color);
        assert_ne!(p2.color, p3.color);
    }
    
    #[test]
    fn test_create_character() {
        let mut state = GameState::new();
        let player = state.add_player("Alice".to_string());
        
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let result = state.create_character(
            &player.id,
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs,
        );
        
        assert!(result.is_ok());
        let character = state.get_character(&player.id).unwrap();
        assert_eq!(character.name, "Theron");
        assert_eq!(character.class, Class::Warrior);
    }
    
    #[test]
    fn test_roll_duality() {
        let state = GameState::new();
        let result = state.roll_duality(2, false);
        
        // Should have valid values
        assert!(result.hope >= 1 && result.hope <= 12);
        assert!(result.fear >= 1 && result.fear <= 12);
        assert_eq!(result.modifier, 2);
        assert!(result.controlling_die == "Hope" || result.controlling_die == "Fear");
    }
}
