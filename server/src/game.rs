//! Game state management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A connected player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub connected: bool,
}

/// The global game state
#[derive(Debug, Clone, Default)]
pub struct GameState {
    pub players: HashMap<Uuid, Player>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    /// Add a new player
    pub fn add_player(&mut self, name: String) -> Player {
        let player = Player {
            id: Uuid::new_v4(),
            name,
            connected: true,
        };
        self.players.insert(player.id, player.clone());
        player
    }

    /// Remove a player
    pub fn remove_player(&mut self, player_id: &Uuid) -> Option<Player> {
        self.players.remove(player_id)
    }

    /// Get all connected players
    pub fn get_players(&self) -> Vec<Player> {
        self.players.values().cloned().collect()
    }

    /// Get player count
    pub fn player_count(&self) -> usize {
        self.players.len()
    }
}

/// Shared game state wrapped for concurrent access
pub type SharedGameState = Arc<RwLock<GameState>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_player() {
        let mut state = GameState::new();
        let player = state.add_player("Alice".to_string());
        
        assert_eq!(player.name, "Alice");
        assert_eq!(state.player_count(), 1);
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
}
