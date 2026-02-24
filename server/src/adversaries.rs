//! Adversary template system

use serde::{Deserialize, Serialize};

/// Adversary template for spawning enemies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdversaryTemplate {
    pub id: String,
    pub name: String,
    pub tier: String, // "common", "medium", "boss"
    pub hp: u8,
    pub evasion: u8,
    pub armor: u8,
    pub attack_modifier: i8,
    pub damage: String, // e.g., "1d6", "2d8+2"
    pub description: String,
}

impl AdversaryTemplate {
    /// Get all built-in templates
    pub fn get_all_templates() -> Vec<AdversaryTemplate> {
        vec![
            // Common enemies
            AdversaryTemplate {
                id: "goblin".to_string(),
                name: "Goblin".to_string(),
                tier: "common".to_string(),
                hp: 3,
                evasion: 10,
                armor: 1,
                attack_modifier: 1,
                damage: "1d6".to_string(),
                description: "Small, cunning raiders with crude weapons".to_string(),
            },
            AdversaryTemplate {
                id: "bandit".to_string(),
                name: "Bandit".to_string(),
                tier: "common".to_string(),
                hp: 4,
                evasion: 11,
                armor: 2,
                attack_modifier: 1,
                damage: "1d6+1".to_string(),
                description: "Opportunistic outlaws and thieves".to_string(),
            },
            AdversaryTemplate {
                id: "wolf".to_string(),
                name: "Wolf".to_string(),
                tier: "common".to_string(),
                hp: 3,
                evasion: 12,
                armor: 0,
                attack_modifier: 2,
                damage: "1d6".to_string(),
                description: "Swift pack hunters with sharp fangs".to_string(),
            },
            // Medium enemies
            AdversaryTemplate {
                id: "orc_warrior".to_string(),
                name: "Orc Warrior".to_string(),
                tier: "medium".to_string(),
                hp: 5,
                evasion: 10,
                armor: 3,
                attack_modifier: 2,
                damage: "1d8+2".to_string(),
                description: "Brutal melee combatants clad in heavy armor".to_string(),
            },
            AdversaryTemplate {
                id: "shadow_beast".to_string(),
                name: "Shadow Beast".to_string(),
                tier: "medium".to_string(),
                hp: 4,
                evasion: 13,
                armor: 1,
                attack_modifier: 3,
                damage: "1d8".to_string(),
                description: "Ethereal predators from the shadowlands".to_string(),
            },
            // Boss enemies
            AdversaryTemplate {
                id: "ogre".to_string(),
                name: "Ogre".to_string(),
                tier: "boss".to_string(),
                hp: 8,
                evasion: 9,
                armor: 4,
                attack_modifier: 3,
                damage: "2d6+3".to_string(),
                description: "Massive, dim-witted brutes with devastating strength".to_string(),
            },
            AdversaryTemplate {
                id: "dragon_wyrmling".to_string(),
                name: "Dragon Wyrmling".to_string(),
                tier: "boss".to_string(),
                hp: 10,
                evasion: 12,
                armor: 5,
                attack_modifier: 4,
                damage: "2d8+2".to_string(),
                description: "Young dragon with deadly breath and sharp claws".to_string(),
            },
        ]
    }

    /// Get a specific template by ID
    pub fn get_template(id: &str) -> Option<AdversaryTemplate> {
        Self::get_all_templates()
            .into_iter()
            .find(|t| t.id == id)
    }
}
