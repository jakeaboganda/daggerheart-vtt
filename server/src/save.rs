//! Save/Load system - Phase 5A: Refactored for Character/Connection architecture

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use daggerheart_engine::character::{Ancestry, Attributes, Class};

use crate::game::{Character, GameState};
use crate::protocol::Position;

/// Saved character data (without runtime resources)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedCharacter {
    pub id: String,
    pub name: String,
    pub class: String,
    pub ancestry: String,
    pub attributes: [i8; 6],
    pub hp_current: u8,
    pub hp_max: u8,
    pub stress: u8,
    pub hope_current: u8,
    pub hope_max: u8,
    pub evasion: i32,
    pub position: Position,
    pub color: String,
    pub is_npc: bool,
}

/// A saved game session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_saved: DateTime<Utc>,
    pub characters: Vec<SavedCharacter>,
}

impl SavedCharacter {
    fn from_character(character: &Character) -> Self {
        Self {
            id: character.id.to_string(),
            name: character.name.clone(),
            class: format!("{:?}", character.class),
            ancestry: format!("{:?}", character.ancestry),
            attributes: [
                character.attributes.agility,
                character.attributes.strength,
                character.attributes.finesse,
                character.attributes.instinct,
                character.attributes.presence,
                character.attributes.knowledge,
            ],
            hp_current: character.hp.current,
            hp_max: character.hp.maximum,
            stress: character.stress.current,
            hope_current: character.hope.current,
            hope_max: character.hope.maximum,
            evasion: character.evasion,
            position: character.position,
            color: character.color.clone(),
            is_npc: character.is_npc,
        }
    }

    fn to_character(&self) -> Result<Character, String> {
        let id = Uuid::parse_str(&self.id).map_err(|e| format!("Invalid character ID: {}", e))?;

        let class = match self.class.as_str() {
            "Bard" => Class::Bard,
            "Druid" => Class::Druid,
            "Guardian" => Class::Guardian,
            "Ranger" => Class::Ranger,
            "Rogue" => Class::Rogue,
            "Seraph" => Class::Seraph,
            "Sorcerer" => Class::Sorcerer,
            "Warrior" => Class::Warrior,
            "Wizard" => Class::Wizard,
            _ => return Err(format!("Invalid class: {}", self.class)),
        };

        let ancestry = match self.ancestry.as_str() {
            "Clank" => Ancestry::Clank,
            "Daemon" => Ancestry::Daemon,
            "Drakona" => Ancestry::Drakona,
            "Dwarf" => Ancestry::Dwarf,
            "Faerie" => Ancestry::Faerie,
            "Faun" => Ancestry::Faun,
            "Fungril" => Ancestry::Fungril,
            "Galapa" => Ancestry::Galapa,
            "Giant" => Ancestry::Giant,
            "Goblin" => Ancestry::Goblin,
            "Halfling" => Ancestry::Halfling,
            "Human" => Ancestry::Human,
            "Inferis" => Ancestry::Inferis,
            "Katari" => Ancestry::Katari,
            "Orc" => Ancestry::Orc,
            "Ribbet" => Ancestry::Ribbet,
            "Simiah" => Ancestry::Simiah,
            _ => return Err(format!("Invalid ancestry: {}", self.ancestry)),
        };

        let attributes = Attributes::from_array(self.attributes)
            .map_err(|e| format!("Invalid attributes: {}", e))?;

        let mut character = if self.is_npc {
            Character::new_npc(
                self.name.clone(),
                class,
                ancestry,
                attributes,
                self.position,
                self.color.clone(),
                self.hp_max,
            )
        } else {
            Character::new(
                self.name.clone(),
                class,
                ancestry,
                attributes,
                self.position,
                self.color.clone(),
            )
        };

        // Override ID to preserve it
        character.id = id;

        // Restore resources to saved values
        character.hp_current = self.hp_current;
        character.hp_max = self.hp_max;
        character.stress_current = self.stress;
        character.hope_current = self.hope_current;
        character.hope_max = self.hope_max;
        character.evasion = self.evasion;
        character.position = self.position;

        character.restore_resources();

        Ok(character)
    }
}

impl SavedSession {
    /// Create a new saved session from game state
    pub fn from_game_state(game: &GameState, name: String) -> Self {
        let characters = game
            .get_characters()
            .iter()
            .map(|c| SavedCharacter::from_character(c))
            .collect();

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            created_at: Utc::now(),
            last_saved: Utc::now(),
            characters,
        }
    }

    /// Save to JSON file
    pub fn save_to_file(&self) -> Result<PathBuf, String> {
        // Create saves directory if it doesn't exist
        let saves_dir = Path::new("saves");
        if !saves_dir.exists() {
            fs::create_dir_all(saves_dir)
                .map_err(|e| format!("Failed to create saves directory: {}", e))?;
        }

        // Generate filename with timestamp
        let timestamp = self.last_saved.format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.json", self.name.replace(' ', "_"), timestamp);
        let path = saves_dir.join(filename);

        // Serialize and save
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write save file: {}", e))?;

        Ok(path)
    }

    /// Load from JSON file
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let json =
            fs::read_to_string(path).map_err(|e| format!("Failed to read save file: {}", e))?;

        serde_json::from_str(&json).map_err(|e| format!("Failed to parse save file: {}", e))
    }

    /// List all saved sessions in the saves directory
    pub fn list_saves() -> Result<Vec<(PathBuf, String, DateTime<Utc>)>, String> {
        let saves_dir = Path::new("saves");
        if !saves_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(saves_dir)
            .map_err(|e| format!("Failed to read saves directory: {}", e))?;

        let mut saves = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(session) = Self::load_from_file(&path) {
                    saves.push((path, session.name, session.last_saved));
                }
            }
        }

        // Sort by timestamp (newest first)
        saves.sort_by(|a, b| b.2.cmp(&a.2));

        Ok(saves)
    }

    /// Apply this saved session to a game state
    /// This replaces all characters but does NOT touch connections
    pub fn apply_to_game(&self, game: &mut GameState) -> Result<(), String> {
        // Clear existing characters
        game.characters.clear();
        game.control_mapping.clear(); // Clear control mappings since characters are gone

        // Restore all characters
        for saved_char in &self.characters {
            let character = saved_char.to_character()?;
            game.characters.insert(character.id, character);
        }

        println!("✅ Loaded {} characters from save", self.characters.len());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::GameState;

    #[test]
    fn test_save_and_load() {
        let mut game = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();

        // Create a character
        let character =
            game.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

        // Save session
        let session = SavedSession::from_game_state(&game, "Test Session".to_string());
        assert_eq!(session.characters.len(), 1);
        assert_eq!(session.characters[0].name, "Theron");

        // Modify character resources
        let char_mut = game.get_character_mut(&character.id).unwrap();
        char_mut.hp.take_damage(2);
        char_mut.stress.gain(1);
        char_mut.sync_resources();

        // Save again
        let session2 = SavedSession::from_game_state(&game, "Modified Session".to_string());
        assert!(session2.characters[0].hp_current < session2.characters[0].hp_max);
        assert!(session2.characters[0].stress > 0);
    }

    #[test]
    fn test_apply_to_game() {
        let mut game = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();

        // Create characters
        game.create_character(
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs.clone(),
        );
        game.create_character("Elara".to_string(), Class::Wizard, Ancestry::Faerie, attrs);

        // Save
        let session = SavedSession::from_game_state(&game, "Test".to_string());

        // Create new game state and apply
        let mut new_game = GameState::new();
        session.apply_to_game(&mut new_game).unwrap();

        assert_eq!(new_game.character_count(), 2);
        assert_eq!(new_game.get_player_characters().len(), 2);
    }

    #[test]
    fn test_character_round_trip() {
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let mut character = Character::new(
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs,
            Position::new(100.0, 200.0),
            "#3b82f6".to_string(),
        );

        // Modify resources
        character.hp.take_damage(3);
        character.stress.gain(2);
        let _ = character.hope.spend(1);
        character.sync_resources();

        // Convert to saved character and back
        let saved = SavedCharacter::from_character(&character);
        let restored = saved.to_character().unwrap();

        assert_eq!(restored.name, character.name);
        assert_eq!(restored.hp.current, character.hp.current);
        assert_eq!(restored.stress.current, character.stress.current);
        assert_eq!(restored.hope.current, character.hope.current);
        assert_eq!(restored.position.x, character.position.x);
        assert_eq!(restored.position.y, character.position.y);
    }

    #[test]
    fn test_npc_round_trip() {
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let mut npc = Character::new_npc(
            "Goblin".to_string(),
            Class::Rogue,
            Ancestry::Goblin,
            attrs,
            Position::new(50.0, 50.0),
            "#ff0000".to_string(),
            8,
        );

        npc.hp.take_damage(2);
        npc.sync_resources();

        let saved = SavedCharacter::from_character(&npc);
        let restored = saved.to_character().unwrap();

        assert!(restored.is_npc);
        assert_eq!(restored.name, "Goblin");
        assert_eq!(restored.hp.current, 6); // 8 - 2
    }
}
