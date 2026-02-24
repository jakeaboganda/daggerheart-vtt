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

use crate::protocol::{
    AttributesData, CharacterData, Position, ResourceData, RollResult, RollTargetType, RollType,
};

/// Game event for the event log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub timestamp: std::time::SystemTime,
    pub event_type: GameEventType,
    pub message: String,
    pub character_name: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameEventType {
    CharacterCreated,
    CharacterMoved,
    RollRequested,
    RollExecuted,
    ResourceUpdate,
    CombatAction,
    SystemMessage,
}

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

/// Pending roll request from GM (Phase 1)
#[derive(Debug, Clone)]
pub struct PendingRollRequest {
    pub id: String,
    pub target_character_ids: Vec<Uuid>,
    pub roll_type: RollType,
    pub attribute: Option<String>,
    pub difficulty: u16,
    pub context: String,
    pub narrative_stakes: Option<String>,
    pub situational_modifier: i8,
    pub has_advantage: bool,
    pub is_combat: bool,
    pub completed_by: Vec<Uuid>, // Characters who have rolled
    pub timestamp: std::time::SystemTime,
}

/// Token type in the Action Tracker
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    PC,
    Adversary,
}

/// Action Tracker for combat turn order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionTracker {
    pub pc_tokens: u8,
    pub adversary_tokens: u8,
    pub queue: Vec<TokenType>, // Order of tokens (leftmost = next)
}

impl ActionTracker {
    pub fn new() -> Self {
        Self {
            pc_tokens: 3,
            adversary_tokens: 3,
            queue: vec![
                TokenType::PC,
                TokenType::PC,
                TokenType::PC,
                TokenType::Adversary,
                TokenType::Adversary,
                TokenType::Adversary,
            ],
        }
    }

    /// Get the next token to act (leftmost in queue)
    pub fn get_next(&self) -> Option<TokenType> {
        self.queue.first().copied()
    }

    /// Remove the next token from the queue
    pub fn pop_next(&mut self) -> Option<TokenType> {
        if !self.queue.is_empty() {
            Some(self.queue.remove(0))
        } else {
            None
        }
    }

    /// Advance a token (add to end of queue)
    pub fn advance_token(&mut self, token_type: TokenType) {
        match token_type {
            TokenType::PC if self.pc_tokens > 0 => {
                self.pc_tokens -= 1;
                self.queue.push(TokenType::PC);
            }
            TokenType::Adversary if self.adversary_tokens > 0 => {
                self.adversary_tokens -= 1;
                self.queue.push(TokenType::Adversary);
            }
            _ => {}
        }
    }

    /// Refill tokens when pool is depleted
    pub fn refill_if_needed(&mut self) {
        if self.queue.is_empty() {
            self.pc_tokens = 3;
            self.adversary_tokens = 3;
            self.queue = vec![
                TokenType::PC,
                TokenType::PC,
                TokenType::PC,
                TokenType::Adversary,
                TokenType::Adversary,
                TokenType::Adversary,
            ];
        }
    }

    /// Add a PC token
    pub fn add_pc_token(&mut self) {
        self.pc_tokens += 1;
        self.queue.push(TokenType::PC);
    }

    /// Add an Adversary token
    pub fn add_adversary_token(&mut self) {
        self.adversary_tokens += 1;
        self.queue.push(TokenType::Adversary);
    }
}

/// Combat encounter state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEncounter {
    pub id: String,
    pub is_active: bool,
    pub round: u32,
    pub action_tracker: ActionTracker,
}

impl CombatEncounter {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            is_active: true,
            round: 1,
            action_tracker: ActionTracker::new(),
        }
    }
}

/// Adversary (enemy) in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adversary {
    pub id: String,
    pub name: String,
    pub template: String,
    pub position: crate::protocol::Position,
    pub hp: u8,
    pub max_hp: u8,
    pub stress: u8,
    pub max_stress: u8,
    pub evasion: u8,
    pub armor: u8,
    pub attack_modifier: i8,
    pub damage_dice: String,
    pub is_active: bool,
}

impl Adversary {
    /// Create from template
    pub fn from_template(
        template: &crate::adversaries::AdversaryTemplate,
        position: crate::protocol::Position,
        instance_number: usize,
    ) -> Self {
        let name = if instance_number > 0 {
            format!("{} #{}", template.name, instance_number)
        } else {
            template.name.clone()
        };

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            template: template.id.clone(),
            position,
            hp: template.hp,
            max_hp: template.hp,
            stress: 0,
            max_stress: template.hp, // Stress max = HP max in Daggerheart
            evasion: template.evasion,
            armor: template.armor,
            attack_modifier: template.attack_modifier,
            damage_dice: template.damage.clone(),
            is_active: true,
        }
    }

    /// Create custom adversary
    pub fn custom(
        name: String,
        position: crate::protocol::Position,
        hp: u8,
        evasion: u8,
        armor: u8,
        attack_modifier: i8,
        damage_dice: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            template: "custom".to_string(),
            position,
            hp,
            max_hp: hp,
            stress: 0,
            max_stress: hp,
            evasion,
            armor,
            attack_modifier,
            damage_dice,
            is_active: true,
        }
    }

    /// Take damage (returns true if taken out)
    pub fn take_damage(&mut self, hp_loss: u8, stress_gain: u8) -> bool {
        if hp_loss > 0 {
            self.hp = self.hp.saturating_sub(hp_loss);
        }

        if stress_gain > 0 {
            self.stress = (self.stress + stress_gain).min(self.max_stress);
        }

        // Taken out if HP = 0 and Stress = max
        if self.hp == 0 && self.stress >= self.max_stress {
            self.is_active = false;
            true
        } else {
            false
        }
    }
}

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

    // Phase 1: Experience system
    pub level: u8,
    pub experiences: Vec<String>,

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
            level: 1,                // Start at level 1
            experiences: Vec::new(), // Start with no Experiences
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
            level: 1,
            experiences: Vec::new(),
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

    /// Get proficiency bonus based on level (Phase 1)
    pub fn proficiency_bonus(&self) -> i8 {
        match self.level {
            1..=3 => 1,
            4..=6 => 2,
            7..=9 => 3,
            _ => 4,
        }
    }

    /// Get attribute modifier by name (Phase 1)
    pub fn get_attribute(&self, attr_name: &str) -> Option<i8> {
        match attr_name.to_lowercase().as_str() {
            "agility" => Some(self.attributes.agility),
            "strength" => Some(self.attributes.strength),
            "finesse" => Some(self.attributes.finesse),
            "instinct" => Some(self.attributes.instinct),
            "presence" => Some(self.attributes.presence),
            "knowledge" => Some(self.attributes.knowledge),
            _ => None,
        }
    }
}

/// A WebSocket connection (ephemeral)
#[derive(Debug, Clone)]
pub struct Connection {
    pub id: Uuid,
}

impl Connection {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
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

    /// Phase 1: Pending roll requests
    pub pending_roll_requests: HashMap<String, PendingRollRequest>,

    /// Phase 1: GM Fear pool
    pub fear_pool: u8,
    
    /// Game event log
    pub event_log: Vec<GameEvent>,
    
    /// Combat encounter (if active)
    pub combat_encounter: Option<CombatEncounter>,
    
    /// Adversaries in the game
    pub adversaries: HashMap<String, Adversary>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            connections: HashMap::new(),
            control_mapping: HashMap::new(),
            color_index: 0,
            pending_roll_requests: HashMap::new(),
            fear_pool: 5, // Starting Fear pool
            event_log: Vec::new(),
            combat_encounter: None,
            adversaries: HashMap::new(),
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
        if let Some((controlling_conn_id, _)) = self
            .control_mapping
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
        self.characters.values().filter(|c| !c.is_npc).collect()
    }

    /// Get all NPCs
    pub fn get_npcs(&self) -> Vec<&Character> {
        self.characters.values().filter(|c| c.is_npc).collect()
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
    
    // ===== Event Log System =====
    
    /// Add an event to the game log
    pub fn add_event(&mut self, event_type: GameEventType, message: String, character_name: Option<String>, details: Option<String>) {
        let event = GameEvent {
            timestamp: std::time::SystemTime::now(),
            event_type,
            message,
            character_name,
            details,
        };
        self.event_log.push(event);
        
        // Keep log size reasonable (last 500 events)
        if self.event_log.len() > 500 {
            self.event_log.drain(0..100); // Remove oldest 100
        }
    }
    
    /// Get recent events (last N)
    pub fn get_recent_events(&self, count: usize) -> Vec<GameEvent> {
        let total = self.event_log.len();
        if total <= count {
            self.event_log.clone()
        } else {
            self.event_log[total - count..].to_vec()
        }
    }
    
    /// Get all events
    pub fn get_all_events(&self) -> &[GameEvent] {
        &self.event_log
    }
    
    /// Clear event log
    pub fn clear_events(&mut self) {
        self.event_log.clear();
    }

    // ===== Phase 1: GM-Initiated Dice Rolls =====

    /// Execute a dice roll for a character
    pub fn execute_roll(
        &mut self,
        character_id: &Uuid,
        request_id: &str,
        spend_hope: bool,
    ) -> Result<crate::protocol::DetailedRollResult, String> {
        // Get the request
        let request = self
            .pending_roll_requests
            .get(request_id)
            .ok_or_else(|| "Roll request not found".to_string())?
            .clone();

        // Get the character (immutable first to calculate modifiers)
        let character = self
            .characters
            .get(character_id)
            .ok_or_else(|| "Character not found".to_string())?;

        // Check if already rolled
        if request.completed_by.contains(character_id) {
            return Err("Character has already rolled for this request".to_string());
        }

        // Calculate modifiers (while character is borrowed immutably)
        let (attr_mod, prof_mod, mut total_mod) = {
            let attr_mod = if let Some(ref attr) = request.attribute {
                character.get_attribute(attr).unwrap_or(0)
            } else {
                0
            };

            let prof_mod = match request.roll_type {
                RollType::Attack | RollType::Spellcast => character.proficiency_bonus(),
                _ => 0,
            };

            let total_mod = attr_mod + prof_mod + request.situational_modifier;
            (attr_mod, prof_mod, total_mod)
        };

        // Now get mutable reference to handle Hope spending
        let character = self
            .characters
            .get_mut(character_id)
            .ok_or_else(|| "Character not found".to_string())?;

        // Handle Hope spending
        let hope_bonus = if spend_hope {
            if character.hope.current >= 1 {
                let _ = character.hope.spend(1);
                character.sync_resources();
                2
            } else {
                return Err("Not enough Hope to spend".to_string());
            }
        } else {
            0
        };

        total_mod += hope_bonus;

        // Roll the dice
        let roll = DualityRoll::roll();
        let hope_die = roll.hope;
        let fear_die = roll.fear;

        // Handle advantage
        let (advantage_die, total) = if request.has_advantage {
            use rand::Rng;
            let d6 = rand::thread_rng().gen_range(1..=6);
            let total = hope_die as u16 + fear_die as u16 + d6 as u16 + total_mod as u16;
            (Some(d6), total)
        } else {
            let total = hope_die as u16 + fear_die as u16 + total_mod as u16;
            (None, total)
        };

        // Determine outcome
        let is_critical = hope_die == fear_die;
        let controlling_die = if hope_die > fear_die {
            crate::protocol::ControllingDie::Hope
        } else if fear_die > hope_die {
            crate::protocol::ControllingDie::Fear
        } else {
            crate::protocol::ControllingDie::Tied
        };

        let success_type = if is_critical {
            crate::protocol::SuccessType::CriticalSuccess
        } else if total < request.difficulty {
            crate::protocol::SuccessType::Failure
        } else if controlling_die == crate::protocol::ControllingDie::Hope {
            crate::protocol::SuccessType::SuccessWithHope
        } else {
            crate::protocol::SuccessType::SuccessWithFear
        };

        // Update Hope/Fear
        let (hope_change, fear_change) = match success_type {
            crate::protocol::SuccessType::SuccessWithHope => {
                character.hope.gain(1);
                character.sync_resources();
                (1, 0)
            }
            crate::protocol::SuccessType::SuccessWithFear => {
                self.fear_pool = self.fear_pool.saturating_add(1);
                (0, 1)
            }
            _ => (0, 0), // Critical or Failure = no resource change
        };

        // Subtract Hope bonus if it was spent
        let final_hope_change = hope_change - (if spend_hope { 1 } else { 0 });

        // Mark as completed
        if let Some(req) = self.pending_roll_requests.get_mut(request_id) {
            req.completed_by.push(*character_id);
        }

        Ok(crate::protocol::DetailedRollResult {
            hope_die,
            fear_die,
            advantage_die,
            attribute_modifier: attr_mod,
            proficiency_modifier: prof_mod,
            situational_modifier: request.situational_modifier,
            hope_bonus,
            total_modifier: total_mod,
            total,
            difficulty: request.difficulty,
            success_type,
            controlling_die,
            is_critical,
            hope_change: final_hope_change,
            fear_change,
        })
    }

    // ===== Combat Management =====

    /// Start a new combat encounter
    pub fn start_combat(&mut self) -> String {
        let encounter = CombatEncounter::new();
        let encounter_id = encounter.id.clone();
        
        self.combat_encounter = Some(encounter);
        
        // Log event
        self.add_event(
            GameEventType::SystemMessage,
            "Combat started".to_string(),
            None,
            Some(format!("Round {}", 1)),
        );
        
        encounter_id
    }

    /// End the current combat encounter
    pub fn end_combat(&mut self, reason: &str) {
        if let Some(_encounter) = self.combat_encounter.take() {
            self.add_event(
                GameEventType::SystemMessage,
                format!("Combat ended: {}", reason),
                None,
                None,
            );
        }
    }

    /// Get the current combat encounter
    pub fn get_combat(&self) -> Option<&CombatEncounter> {
        self.combat_encounter.as_ref()
    }

    /// Get mutable reference to combat
    pub fn get_combat_mut(&mut self) -> Option<&mut CombatEncounter> {
        self.combat_encounter.as_mut()
    }

    /// Advance the action tracker based on roll result
    pub fn advance_tracker(&mut self, success_with_hope: bool) {
        if let Some(encounter) = &mut self.combat_encounter {
            let token_type = if success_with_hope {
                TokenType::PC
            } else {
                TokenType::Adversary
            };
            
            encounter.action_tracker.advance_token(token_type);
            encounter.action_tracker.refill_if_needed();
        }
    }

    /// Get next actor in combat
    pub fn get_next_actor(&self) -> Option<TokenType> {
        self.combat_encounter
            .as_ref()
            .and_then(|e| e.action_tracker.get_next())
    }

    // ===== Adversary Management =====

    /// Spawn an adversary from template
    pub fn spawn_adversary(
        &mut self,
        template_id: &str,
        position: crate::protocol::Position,
    ) -> Result<Adversary, String> {
        let template = crate::adversaries::AdversaryTemplate::get_template(template_id)
            .ok_or_else(|| format!("Template not found: {}", template_id))?;

        // Count existing adversaries with this template for instance numbering
        let instance_count = self
            .adversaries
            .values()
            .filter(|adv| adv.template == template_id)
            .count();

        let adversary = Adversary::from_template(&template, position, instance_count + 1);
        let adversary_id = adversary.id.clone();
        
        // Log event
        self.add_event(
            GameEventType::SystemMessage,
            format!("{} spawned", adversary.name),
            None,
            Some(format!(
                "HP: {}/{}, Evasion: {}, Armor: {}",
                adversary.hp, adversary.max_hp, adversary.evasion, adversary.armor
            )),
        );

        self.adversaries.insert(adversary_id.clone(), adversary.clone());
        Ok(adversary)
    }

    /// Create a custom adversary
    pub fn create_custom_adversary(
        &mut self,
        name: String,
        position: crate::protocol::Position,
        hp: u8,
        evasion: u8,
        armor: u8,
        attack_modifier: i8,
        damage_dice: String,
    ) -> Adversary {
        let adversary = Adversary::custom(
            name.clone(),
            position,
            hp,
            evasion,
            armor,
            attack_modifier,
            damage_dice,
        );

        // Log event
        self.add_event(
            GameEventType::SystemMessage,
            format!("{} spawned (custom)", adversary.name),
            None,
            Some(format!(
                "HP: {}/{}, Evasion: {}, Armor: {}",
                adversary.hp, adversary.max_hp, adversary.evasion, adversary.armor
            )),
        );

        let adversary_id = adversary.id.clone();
        self.adversaries.insert(adversary_id, adversary.clone());
        adversary
    }

    /// Remove an adversary
    pub fn remove_adversary(&mut self, adversary_id: &str) -> Option<Adversary> {
        if let Some(adversary) = self.adversaries.remove(adversary_id) {
            self.add_event(
                GameEventType::SystemMessage,
                format!("{} removed", adversary.name),
                None,
                None,
            );
            Some(adversary)
        } else {
            None
        }
    }

    /// Get all adversaries
    pub fn get_adversaries(&self) -> Vec<&Adversary> {
        self.adversaries.values().collect()
    }

    /// Get active adversaries only
    pub fn get_active_adversaries(&self) -> Vec<&Adversary> {
        self.adversaries
            .values()
            .filter(|adv| adv.is_active)
            .collect()
    }

    /// Update adversary HP after damage
    pub fn update_adversary_hp(&mut self, adversary_id: &str, hp_loss: u8, stress_gain: u8) -> Result<bool, String> {
        let adversary = self
            .adversaries
            .get_mut(adversary_id)
            .ok_or_else(|| format!("Adversary not found: {}", adversary_id))?;

        let taken_out = adversary.take_damage(hp_loss, stress_gain);
        let adversary_name = adversary.name.clone(); // Clone before borrowing self again

        if taken_out {
            self.add_event(
                GameEventType::CombatAction,
                format!("{} taken out!", adversary_name),
                None,
                None,
            );
        }

        Ok(taken_out)
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

        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

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
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

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
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

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
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

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
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

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

        let c1 = state.create_character(
            "C1".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs.clone(),
        );
        let c2 = state.create_character(
            "C2".to_string(),
            Class::Warrior,
            Ancestry::Human,
            attrs.clone(),
        );
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
        assert!(
            result.controlling_die == "Hope"
                || result.controlling_die == "Fear"
                || result.controlling_die == "Tied"
        );
    }

    #[test]
    fn test_resource_sync_and_restore() {
        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

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

    // ===== Phase 1: Dice Roll Tests =====

    #[test]
    fn test_proficiency_bonus_progression() {
        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let mut character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

        // Level 1-3: +1
        character.level = 1;
        assert_eq!(character.proficiency_bonus(), 1);
        character.level = 3;
        assert_eq!(character.proficiency_bonus(), 1);

        // Level 4-6: +2
        character.level = 4;
        assert_eq!(character.proficiency_bonus(), 2);
        character.level = 6;
        assert_eq!(character.proficiency_bonus(), 2);

        // Level 7-9: +3
        character.level = 7;
        assert_eq!(character.proficiency_bonus(), 3);
        character.level = 9;
        assert_eq!(character.proficiency_bonus(), 3);

        // Level 10+: +4
        character.level = 10;
        assert_eq!(character.proficiency_bonus(), 4);
        character.level = 15;
        assert_eq!(character.proficiency_bonus(), 4);
    }

    #[test]
    fn test_get_attribute() {
        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

        assert_eq!(character.get_attribute("agility"), Some(2));
        assert_eq!(character.get_attribute("strength"), Some(1));
        assert_eq!(character.get_attribute("knowledge"), Some(-1));
        assert_eq!(character.get_attribute("invalid"), None);
        assert_eq!(character.get_attribute("AGILITY"), Some(2)); // case insensitive
    }

    #[test]
    fn test_experience_initialization() {
        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

        assert_eq!(character.level, 1);
        assert!(character.experiences.is_empty());
    }

    #[test]
    fn test_fear_pool_initialization() {
        let state = GameState::new();
        assert_eq!(state.fear_pool, 5); // Starting Fear pool
    }

    #[test]
    fn test_pending_roll_requests() {
        let state = GameState::new();
        assert!(state.pending_roll_requests.is_empty());
    }

    #[test]
    fn test_execute_roll_without_request() {
        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

        // Try to execute a roll for a non-existent request
        let result = state.execute_roll(&character.id, "fake-request-id", false);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Roll request not found");
    }

    #[test]
    fn test_execute_roll_with_insufficient_hope() {
        use crate::protocol::{RollTargetType, RollType};

        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

        // Spend all Hope
        let char_mut = state.get_character_mut(&character.id).unwrap();
        let _ = char_mut.hope.spend(5);
        char_mut.sync_resources();

        // Create a roll request
        let request = PendingRollRequest {
            id: "test-request".to_string(),
            target_character_ids: vec![character.id],
            roll_type: RollType::Action,
            attribute: Some("agility".to_string()),
            difficulty: 14,
            context: "Test roll".to_string(),
            narrative_stakes: None,
            situational_modifier: 0,
            has_advantage: false,
            is_combat: false,
            completed_by: Vec::new(),
            timestamp: std::time::SystemTime::now(),
        };

        state
            .pending_roll_requests
            .insert("test-request".to_string(), request);

        // Try to execute with spend_hope=true but no Hope
        let result = state.execute_roll(&character.id, "test-request", true);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not enough Hope to spend");
    }

    #[test]
    fn test_execute_roll_success() {
        use crate::protocol::{RollType, SuccessType};

        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

        // Create a roll request
        let request = PendingRollRequest {
            id: "test-request".to_string(),
            target_character_ids: vec![character.id],
            roll_type: RollType::Action,
            attribute: Some("agility".to_string()),
            difficulty: 14,
            context: "Test roll".to_string(),
            narrative_stakes: None,
            situational_modifier: 0,
            has_advantage: false,
            is_combat: false,
            completed_by: Vec::new(),
            timestamp: std::time::SystemTime::now(),
        };

        state
            .pending_roll_requests
            .insert("test-request".to_string(), request);

        // Execute the roll
        let result = state.execute_roll(&character.id, "test-request", false);
        assert!(result.is_ok());

        let roll_result = result.unwrap();

        // Verify dice are in valid range
        assert!(roll_result.hope_die >= 1 && roll_result.hope_die <= 12);
        assert!(roll_result.fear_die >= 1 && roll_result.fear_die <= 12);

        // Verify modifiers
        assert_eq!(roll_result.attribute_modifier, 2); // Agility
        assert_eq!(roll_result.proficiency_modifier, 0); // Not an attack
        assert_eq!(roll_result.situational_modifier, 0);
        assert_eq!(roll_result.hope_bonus, 0); // Didn't spend Hope

        // Verify success type is one of the valid types
        match roll_result.success_type {
            SuccessType::Failure
            | SuccessType::SuccessWithHope
            | SuccessType::SuccessWithFear
            | SuccessType::CriticalSuccess => {}
        }

        // Verify critical detection
        if roll_result.hope_die == roll_result.fear_die {
            assert!(roll_result.is_critical);
            assert_eq!(roll_result.success_type, SuccessType::CriticalSuccess);
        }

        // Verify the request is marked as completed
        let req = state.pending_roll_requests.get("test-request").unwrap();
        assert!(req.completed_by.contains(&character.id));
    }

    #[test]
    fn test_hope_fear_changes_on_success() {
        use crate::protocol::RollType;

        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

        // Reduce Hope below max so we can test the gain
        let char_mut = state.get_character_mut(&character.id).unwrap();
        let _ = char_mut.hope.spend(2); // Spend 2 Hope (5 → 3)
        char_mut.sync_resources();

        let initial_hope = state.characters.get(&character.id).unwrap().hope.current;
        let initial_fear = state.fear_pool;

        assert_eq!(initial_hope, 3); // Verify starting Hope is 3

        // Create a roll request with very low DC to ensure success
        let request = PendingRollRequest {
            id: "test-request".to_string(),
            target_character_ids: vec![character.id],
            roll_type: RollType::Action,
            attribute: Some("agility".to_string()),
            difficulty: 1, // Very low DC, almost guaranteed success
            context: "Easy test roll".to_string(),
            narrative_stakes: None,
            situational_modifier: 0,
            has_advantage: false,
            is_combat: false,
            completed_by: Vec::new(),
            timestamp: std::time::SystemTime::now(),
        };

        state
            .pending_roll_requests
            .insert("test-request".to_string(), request);

        // Execute the roll
        let result = state.execute_roll(&character.id, "test-request", false);
        assert!(result.is_ok());

        let roll_result = result.unwrap();

        // Check resource changes based on success type
        let character = state.characters.get(&character.id).unwrap();
        match roll_result.success_type {
            crate::protocol::SuccessType::SuccessWithHope => {
                // Hope should increase by 1 (3 → 4)
                assert_eq!(character.hope.current, initial_hope + 1);
                assert_eq!(state.fear_pool, initial_fear);
                assert_eq!(roll_result.hope_change, 1);
                assert_eq!(roll_result.fear_change, 0);
            }
            crate::protocol::SuccessType::SuccessWithFear => {
                // Fear should increase by 1
                assert_eq!(character.hope.current, initial_hope);
                assert_eq!(state.fear_pool, initial_fear + 1);
                assert_eq!(roll_result.hope_change, 0);
                assert_eq!(roll_result.fear_change, 1);
            }
            crate::protocol::SuccessType::CriticalSuccess => {
                // No resource changes on critical
                assert_eq!(character.hope.current, initial_hope);
                assert_eq!(state.fear_pool, initial_fear);
                assert_eq!(roll_result.hope_change, 0);
                assert_eq!(roll_result.fear_change, 0);
            }
            crate::protocol::SuccessType::Failure => {
                // No resource changes on failure
                assert_eq!(character.hope.current, initial_hope);
                assert_eq!(state.fear_pool, initial_fear);
                assert_eq!(roll_result.hope_change, 0);
                assert_eq!(roll_result.fear_change, 0);
            }
        }
    }

    #[test]
    fn test_attack_roll_uses_proficiency() {
        use crate::protocol::RollType;

        let mut state = GameState::new();
        let attrs = Attributes::from_array([2, 1, 1, 0, 0, -1]).unwrap();
        let character =
            state.create_character("Theron".to_string(), Class::Warrior, Ancestry::Human, attrs);

        // Create an attack roll request
        let request = PendingRollRequest {
            id: "test-request".to_string(),
            target_character_ids: vec![character.id],
            roll_type: RollType::Attack, // Attack should use proficiency
            attribute: Some("strength".to_string()),
            difficulty: 14,
            context: "Attack roll".to_string(),
            narrative_stakes: None,
            situational_modifier: 0,
            has_advantage: false,
            is_combat: true,
            completed_by: Vec::new(),
            timestamp: std::time::SystemTime::now(),
        };

        state
            .pending_roll_requests
            .insert("test-request".to_string(), request);

        // Execute the roll
        let result = state.execute_roll(&character.id, "test-request", false);
        assert!(result.is_ok());

        let roll_result = result.unwrap();

        // Attack rolls should include proficiency
        assert_eq!(roll_result.proficiency_modifier, 1); // Level 1 = +1 proficiency
        assert_eq!(roll_result.attribute_modifier, 1); // Strength
        assert_eq!(roll_result.total_modifier, 2); // 1 + 1
    }

    // ===== Combat & Adversary Tests =====

    #[test]
    fn test_spawn_adversary_from_template() {
        let mut state = GameState::new();
        let position = crate::protocol::Position::new(100.0, 100.0);

        let result = state.spawn_adversary("goblin", position);
        assert!(result.is_ok());

        let adversary = result.unwrap();
        assert_eq!(adversary.template, "goblin");
        assert!(adversary.name.contains("Goblin"));
        assert_eq!(adversary.hp, 3);
        assert_eq!(adversary.max_hp, 3);
        assert_eq!(adversary.evasion, 10);
        assert_eq!(adversary.armor, 1);
        assert_eq!(adversary.attack_modifier, 1);
        assert_eq!(adversary.damage_dice, "1d6");
        assert!(adversary.is_active);

        // Check it was added to game state
        assert_eq!(state.adversaries.len(), 1);
        assert!(state.adversaries.contains_key(&adversary.id));

        // Check event log
        assert_eq!(state.event_log.len(), 1);
    }

    #[test]
    fn test_spawn_multiple_adversaries_instance_numbers() {
        let mut state = GameState::new();
        let pos1 = crate::protocol::Position::new(100.0, 100.0);
        let pos2 = crate::protocol::Position::new(200.0, 100.0);

        let goblin1 = state.spawn_adversary("goblin", pos1).unwrap();
        let goblin2 = state.spawn_adversary("goblin", pos2).unwrap();

        assert_eq!(goblin1.name, "Goblin #1");
        assert_eq!(goblin2.name, "Goblin #2");
        assert_eq!(state.adversaries.len(), 2);
    }

    #[test]
    fn test_spawn_invalid_template() {
        let mut state = GameState::new();
        let position = crate::protocol::Position::new(100.0, 100.0);

        let result = state.spawn_adversary("invalid_template", position);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Template not found: invalid_template");
    }

    #[test]
    fn test_create_custom_adversary() {
        let mut state = GameState::new();
        let position = crate::protocol::Position::new(100.0, 100.0);

        let adversary = state.create_custom_adversary(
            "Custom Boss".to_string(),
            position,
            10,  // hp
            15,  // evasion
            5,   // armor
            3,   // attack_modifier
            "2d8+3".to_string(),
        );

        assert_eq!(adversary.name, "Custom Boss");
        assert_eq!(adversary.template, "custom");
        assert_eq!(adversary.hp, 10);
        assert_eq!(adversary.evasion, 15);
        assert_eq!(adversary.armor, 5);
        assert_eq!(adversary.attack_modifier, 3);
        assert_eq!(adversary.damage_dice, "2d8+3");

        assert_eq!(state.adversaries.len(), 1);
    }

    #[test]
    fn test_remove_adversary() {
        let mut state = GameState::new();
        let position = crate::protocol::Position::new(100.0, 100.0);

        let adversary = state.spawn_adversary("goblin", position).unwrap();
        let adversary_id = adversary.id.clone();

        assert_eq!(state.adversaries.len(), 1);

        let removed = state.remove_adversary(&adversary_id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, adversary_id);
        assert_eq!(state.adversaries.len(), 0);

        // Check event log (spawn + remove)
        assert_eq!(state.event_log.len(), 2);
    }

    #[test]
    fn test_adversary_take_damage_hp_loss() {
        let position = crate::protocol::Position::new(100.0, 100.0);
        let mut adversary = Adversary::custom(
            "Test Enemy".to_string(),
            position,
            5, // hp
            10, // evasion
            2, // armor
            1, // attack_modifier
            "1d6".to_string(),
        );

        // Deal 1 HP damage
        let taken_out = adversary.take_damage(1, 0);
        assert_eq!(adversary.hp, 4);
        assert_eq!(adversary.stress, 0);
        assert!(!taken_out);
        assert!(adversary.is_active);
    }

    #[test]
    fn test_adversary_take_damage_stress_gain() {
        let position = crate::protocol::Position::new(100.0, 100.0);
        let mut adversary = Adversary::custom(
            "Test Enemy".to_string(),
            position,
            5, // hp
            10, // evasion
            2, // armor
            1, // attack_modifier
            "1d6".to_string(),
        );

        // Deal stress damage (scratch)
        let taken_out = adversary.take_damage(0, 1);
        assert_eq!(adversary.hp, 5);
        assert_eq!(adversary.stress, 1);
        assert!(!taken_out);
    }

    #[test]
    fn test_adversary_taken_out() {
        let position = crate::protocol::Position::new(100.0, 100.0);
        let mut adversary = Adversary::custom(
            "Test Enemy".to_string(),
            position,
            3, // hp
            10, // evasion
            2, // armor
            1, // attack_modifier
            "1d6".to_string(),
        );

        // Reduce HP to 0
        adversary.take_damage(3, 0);
        assert_eq!(adversary.hp, 0);
        assert!(adversary.is_active); // Still active until stress fills

        // Fill stress to max
        let taken_out = adversary.take_damage(0, 3);
        assert_eq!(adversary.stress, 3);
        assert!(taken_out);
        assert!(!adversary.is_active);
    }

    #[test]
    fn test_start_combat() {
        let mut state = GameState::new();
        
        assert!(state.combat_encounter.is_none());

        let encounter_id = state.start_combat();
        
        assert!(state.combat_encounter.is_some());
        let encounter = state.combat_encounter.as_ref().unwrap();
        assert_eq!(encounter.id, encounter_id);
        assert!(encounter.is_active);
        assert_eq!(encounter.round, 1);
        assert_eq!(encounter.action_tracker.pc_tokens, 3);
        assert_eq!(encounter.action_tracker.adversary_tokens, 3);
        assert_eq!(encounter.action_tracker.queue.len(), 6);

        // Check event log
        assert_eq!(state.event_log.len(), 1);
    }

    #[test]
    fn test_end_combat() {
        let mut state = GameState::new();
        
        state.start_combat();
        assert!(state.combat_encounter.is_some());

        state.end_combat("victory");
        assert!(state.combat_encounter.is_none());

        // Check event log (start + end)
        assert_eq!(state.event_log.len(), 2);
    }

    #[test]
    fn test_action_tracker_get_next() {
        let tracker = ActionTracker::new();
        
        // First token should be PC (from initial queue)
        let next = tracker.get_next();
        assert!(next.is_some());
        assert_eq!(next.unwrap(), TokenType::PC);
    }

    #[test]
    fn test_action_tracker_add_tokens() {
        let mut tracker = ActionTracker::new();
        
        let initial_pc = tracker.pc_tokens;
        let initial_adv = tracker.adversary_tokens;
        let initial_queue_len = tracker.queue.len();

        tracker.add_pc_token();
        assert_eq!(tracker.pc_tokens, initial_pc + 1);
        assert_eq!(tracker.queue.len(), initial_queue_len + 1);

        tracker.add_adversary_token();
        assert_eq!(tracker.adversary_tokens, initial_adv + 1);
        assert_eq!(tracker.queue.len(), initial_queue_len + 2);
    }

    #[test]
    fn test_update_adversary_hp() {
        let mut state = GameState::new();
        let position = crate::protocol::Position::new(100.0, 100.0);
        
        let adversary = state.spawn_adversary("goblin", position).unwrap();
        let adversary_id = adversary.id.clone();

        // Apply damage
        let result = state.update_adversary_hp(&adversary_id, 1, 0);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Not taken out

        let updated = state.adversaries.get(&adversary_id).unwrap();
        assert_eq!(updated.hp, 2); // 3 - 1
    }

    #[test]
    fn test_get_active_adversaries() {
        let mut state = GameState::new();
        let pos1 = crate::protocol::Position::new(100.0, 100.0);
        let pos2 = crate::protocol::Position::new(200.0, 100.0);

        let goblin1 = state.spawn_adversary("goblin", pos1).unwrap();
        let goblin2 = state.spawn_adversary("goblin", pos2).unwrap();

        // Both active
        assert_eq!(state.get_active_adversaries().len(), 2);

        // Take out goblin1
        state.update_adversary_hp(&goblin1.id, 3, 0).ok(); // Reduce HP to 0
        state.update_adversary_hp(&goblin1.id, 0, 3).ok(); // Fill stress

        // Only goblin2 active
        assert_eq!(state.get_active_adversaries().len(), 1);
        assert_eq!(state.get_adversaries().len(), 2); // Both still exist
    }

    #[test]
    fn test_all_adversary_templates_valid() {
        use crate::adversaries::AdversaryTemplate;
        
        let templates = AdversaryTemplate::get_all_templates();
        assert!(!templates.is_empty());

        // Test each template can spawn
        let mut state = GameState::new();
        let position = crate::protocol::Position::new(100.0, 100.0);

        for template in templates {
            let result = state.spawn_adversary(&template.id, position);
            assert!(result.is_ok(), "Failed to spawn: {}", template.id);
            
            let adversary = result.unwrap();
            assert_eq!(adversary.hp, adversary.max_hp);
            assert!(adversary.is_active);
        }
    }
}
