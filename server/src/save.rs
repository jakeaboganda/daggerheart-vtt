//! Save/Load system for game state

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::game::{GameState, Player};
use crate::protocol::Position;

const SAVES_DIR: &str = "saves";

/// Saved game session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_saved: DateTime<Utc>,
    pub players: Vec<SavedPlayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPlayer {
    pub id: String,
    pub name: String,
    pub position: Position,
    pub color: String,
    pub character: Option<SavedCharacter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedCharacter {
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
}

impl SavedSession {
    /// Create a new saved session from game state
    pub fn from_game_state(game: &GameState, name: String) -> Self {
        let players = game
            .get_players()
            .iter()
            .map(SavedPlayer::from_player)
            .collect();

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            created_at: Utc::now(),
            last_saved: Utc::now(),
            players,
        }
    }

    /// Save to file
    pub fn save_to_file(&self) -> Result<PathBuf, String> {
        // Ensure saves directory exists
        fs::create_dir_all(SAVES_DIR)
            .map_err(|e| format!("Failed to create saves directory: {}", e))?;

        // Generate filename
        let timestamp = self.last_saved.format("%Y%m%d_%H%M%S");
        let safe_name = self
            .name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();
        let filename = format!("{}_{}.json", safe_name, timestamp);
        let path = Path::new(SAVES_DIR).join(&filename);

        // Write JSON
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(path)
    }

    /// Load from file
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let json = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        let mut session: SavedSession =
            serde_json::from_str(&json).map_err(|e| format!("Failed to deserialize: {}", e))?;

        // Update last_saved timestamp
        session.last_saved = Utc::now();

        Ok(session)
    }

    /// List all saved sessions
    pub fn list_saves() -> Result<Vec<(PathBuf, String, DateTime<Utc>)>, String> {
        if !Path::new(SAVES_DIR).exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(SAVES_DIR)
            .map_err(|e| format!("Failed to read saves directory: {}", e))?;

        let mut saves = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(session) = SavedSession::load_from_file(&path) {
                    saves.push((path, session.name, session.last_saved));
                }
            }
        }

        // Sort by last_saved (newest first)
        saves.sort_by(|a, b| b.2.cmp(&a.2));

        Ok(saves)
    }

    /// Apply this saved session to game state
    pub fn apply_to_game(&self, game: &mut GameState) -> Result<(), String> {
        use crate::game::Character;
        use daggerheart_engine::{
            character::{Ancestry, Attributes, Class},
            combat::{HitPoints, Hope, Stress},
        };

        // Clear existing players
        game.players.clear();

        // Restore players
        for saved_player in &self.players {
            let player_id =
                Uuid::parse_str(&saved_player.id).map_err(|e| format!("Invalid UUID: {}", e))?;

            let character = if let Some(saved_char) = &saved_player.character {
                // Parse class
                let class = match saved_char.class.as_str() {
                    "Bard" => Class::Bard,
                    "Druid" => Class::Druid,
                    "Guardian" => Class::Guardian,
                    "Ranger" => Class::Ranger,
                    "Rogue" => Class::Rogue,
                    "Seraph" => Class::Seraph,
                    "Sorcerer" => Class::Sorcerer,
                    "Warrior" => Class::Warrior,
                    "Wizard" => Class::Wizard,
                    _ => return Err(format!("Unknown class: {}", saved_char.class)),
                };

                // Parse ancestry
                let ancestry = match saved_char.ancestry.as_str() {
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
                    _ => return Err(format!("Unknown ancestry: {}", saved_char.ancestry)),
                };

                // Parse attributes
                let attributes = Attributes::from_array(saved_char.attributes)
                    .map_err(|e| format!("Invalid attributes: {}", e))?;

                // Create character
                let mut character =
                    Character::new(saved_char.name.clone(), class, ancestry, attributes);

                // Restore resources
                character.hp = HitPoints::new(saved_char.hp_max);
                if saved_char.hp_current < saved_char.hp_max {
                    let damage = saved_char.hp_max - saved_char.hp_current;
                    character.hp.take_damage(damage);
                }

                character.stress = Stress::new();
                character.stress.gain(saved_char.stress);

                character.hope = Hope::new(saved_char.hope_max);
                if saved_char.hope_current < saved_char.hope_max {
                    let spent = saved_char.hope_max - saved_char.hope_current;
                    let _ = character.hope.spend(spent);
                }

                character.evasion = saved_char.evasion;

                Some(character)
            } else {
                None
            };

            // Create player
            let player = Player {
                id: player_id,
                name: saved_player.name.clone(),
                connected: false, // Will reconnect
                position: saved_player.position,
                color: saved_player.color.clone(),
                character,
            };

            game.players.insert(player_id, player);
        }

        Ok(())
    }
}

impl SavedPlayer {
    fn from_player(player: &Player) -> Self {
        let character = player.character.as_ref().map(|c| SavedCharacter {
            name: c.name.clone(),
            class: format!("{:?}", c.class),
            ancestry: format!("{:?}", c.ancestry),
            attributes: [
                c.attributes.agility,
                c.attributes.strength,
                c.attributes.finesse,
                c.attributes.instinct,
                c.attributes.presence,
                c.attributes.knowledge,
            ],
            hp_current: c.hp.current,
            hp_max: c.hp.maximum,
            stress: c.stress.current,
            hope_current: c.hope.current,
            hope_max: c.hope.maximum,
            evasion: c.evasion,
        });

        Self {
            id: player.id.to_string(),
            name: player.name.clone(),
            position: player.position,
            color: player.color.clone(),
            character,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daggerheart_engine::character::{Ancestry, Attributes, Class};

    #[test]
    fn test_save_and_load() {
        let mut game = GameState::new();
        let player = game.add_player("Alice".to_string());

        // Create character
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        game.create_character(
            &player.id,
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs,
        )
        .unwrap();

        // Save
        let session = SavedSession::from_game_state(&game, "Test Session".to_string());
        let path = session.save_to_file().unwrap();

        // Load
        let loaded = SavedSession::load_from_file(&path).unwrap();

        assert_eq!(loaded.name, "Test Session");
        assert_eq!(loaded.players.len(), 1);
        assert_eq!(loaded.players[0].name, "Alice");
        assert!(loaded.players[0].character.is_some());

        // Cleanup
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_apply_to_game() {
        let mut game = GameState::new();
        let player = game.add_player("Alice".to_string());

        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        game.create_character(
            &player.id,
            "Theron".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs,
        )
        .unwrap();

        // Save
        let session = SavedSession::from_game_state(&game, "Test".to_string());

        // Apply to new game
        let mut new_game = GameState::new();
        session.apply_to_game(&mut new_game).unwrap();

        assert_eq!(new_game.get_players().len(), 1);
        let restored_player = &new_game.get_players()[0];
        assert_eq!(restored_player.name, "Alice");
        assert!(restored_player.character.is_some());
    }
}
