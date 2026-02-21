//! Game state management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::protocol::Position;

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

/// A connected player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub connected: bool,
    pub position: Position,
    pub color: String,
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

    #[test]
    fn test_add_player() {
        let mut state = GameState::new();
        let player = state.add_player("Alice".to_string());
        
        assert_eq!(player.name, "Alice");
        assert_eq!(state.player_count(), 1);
        assert!(!player.color.is_empty());
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
}
