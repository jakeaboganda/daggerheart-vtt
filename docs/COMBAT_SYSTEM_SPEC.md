# Daggerheart Combat System - Complete Specification

**Date:** 2026-02-24  
**System:** Daggerheart (NOT D&D!)  
**Complexity:** High (3-4 days implementation)  
**Prerequisites:** GM Roll Requests, Event Log

---

## ⚠️ Daggerheart-Specific Rules

Daggerheart combat is **fundamentally different** from D&D:

| Feature | D&D | Daggerheart |
|---------|-----|-------------|
| **Turn Order** | Initiative roll, static order | **Action Tracker** - dynamic based on roll results |
| **Success** | Beat AC | Beat Evasion, **plus** Hope vs Fear determines who goes next |
| **Damage** | Direct HP loss | **Threshold system**: <5 = Stress, ≥5 = HP |
| **Armor** | AC (avoid hits) | **Armor Score** (reduce damage) |
| **Death** | 0 HP = unconscious | **0 HP, then Stress fills = taken out** |

---

## 👹 Adversary Management System

### Pre-Combat: Creating Adversaries

**Before combat starts**, GM needs to spawn adversaries on the map:

#### Adversary Library
**Built-in Templates:**
```
Common:
  Goblin       - HP: 3, Evasion: 10, Armor: 1, Damage: 1d6
  Bandit       - HP: 4, Evasion: 11, Armor: 2, Damage: 1d6+1
  Wolf         - HP: 3, Evasion: 12, Armor: 0, Damage: 1d6
  
Medium:
  Orc Warrior  - HP: 5, Evasion: 10, Armor: 3, Damage: 1d8+2
  Shadow Beast - HP: 4, Evasion: 13, Armor: 1, Damage: 1d8
  
Boss:
  Ogre         - HP: 8, Evasion: 9, Armor: 4, Damage: 2d6+3
  Dragon Wyrmling - HP: 10, Evasion: 12, Armor: 5, Damage: 2d8+2
```

#### GM Spawning Flow
1. **Open Adversary Panel** (GM view sidebar)
2. **Select template** from dropdown OR **Create Custom**
3. **Set stats:**
   - Name (e.g., "Goblin Archer #1")
   - HP / Max HP
   - Evasion
   - Armor Score
   - Attack Modifier
   - Damage Dice (e.g., "1d6+1")
4. **Click on map** to place adversary
5. **Adversary appears:**
   - Token on canvas (red/purple)
   - Added to combatant list
   - Broadcasted to all clients
6. **Repeat** for multiple enemies

#### Custom Adversary Creation
```
┌─────────────────────────────────┐
│ ➕ Create Adversary             │
├─────────────────────────────────┤
│ Name: [Goblin Archer #1      ]  │
│                                 │
│ HP: [3] / Max: [3]              │
│ Evasion: [10]                   │
│ Armor: [1]                      │
│                                 │
│ Attack Modifier: [+1]           │
│ Damage: [1d6]                   │
│                                 │
│ [💾 Save Template] [🗑️ Delete]  │
│ [✅ Spawn on Map]               │
└─────────────────────────────────┘
```

### Adversary State Management

**Backend Storage:**
```rust
pub struct Adversary {
    pub id: String,
    pub name: String,
    pub template: String,  // "goblin", "custom", etc.
    pub position: Position,
    pub hp: u8,
    pub max_hp: u8,
    pub stress: u8,
    pub max_stress: u8,
    pub evasion: u8,
    pub armor: u8,
    pub attack_modifier: i8,
    pub damage_dice: String,  // "1d6+1"
    pub is_active: bool,  // false when taken out
}

impl GameState {
    pub fn spawn_adversary(&mut self, template: &str, position: Position) -> String;
    pub fn create_custom_adversary(&mut self, stats: AdversaryStats) -> String;
    pub fn remove_adversary(&mut self, id: &str);
    pub fn get_adversaries(&self) -> Vec<&Adversary>;
    pub fn get_active_adversaries(&self) -> Vec<&Adversary>;
    pub fn update_adversary_hp(&mut self, id: &str, new_hp: u8);
}
```

**WebSocket Protocol:**
```rust
// Client → Server
ClientMessage::SpawnAdversary {
    template: String,  // "goblin" or "custom"
    position: Position,
    custom_stats: Option<AdversaryStats>,
}

ClientMessage::RemoveAdversary {
    adversary_id: String,
}

ClientMessage::UpdateAdversary {
    adversary_id: String,
    stats: AdversaryStats,
}

// Server → Clients
ServerMessage::AdversarySpawned {
    adversary: AdversaryData,
}

ServerMessage::AdversaryRemoved {
    adversary_id: String,
    name: String,
}

ServerMessage::AdversaryUpdated {
    adversary_id: String,
    hp: u8,
    stress: u8,
    position: Position,
}
```

### Adversary Templates System

**JSON Template Files:**
```json
{
  "templates": [
    {
      "id": "goblin",
      "name": "Goblin",
      "tier": "common",
      "hp": 3,
      "evasion": 10,
      "armor": 1,
      "attack_modifier": 1,
      "damage": "1d6",
      "description": "Small, cunning raiders"
    },
    {
      "id": "orc_warrior",
      "name": "Orc Warrior",
      "tier": "medium",
      "hp": 5,
      "evasion": 10,
      "armor": 3,
      "attack_modifier": 2,
      "damage": "1d8+2",
      "description": "Brutal melee combatant"
    }
  ]
}
```

**Loading Templates:**
```rust
// server/src/adversaries.rs (new file)
pub fn load_templates() -> Vec<AdversaryTemplate>;
pub fn get_template(id: &str) -> Option<AdversaryTemplate>;
```

### Combat Integration

**When combat starts:**
1. Count active adversaries on map
2. Initialize Action Tracker (3 PC tokens, 3 Adversary tokens)
3. Adversaries can be targeted for attacks
4. GM can roll adversary attacks

**Adversary Attacks (GM-controlled):**
```
┌─────────────────────────────────┐
│ Goblin Archer #1's Turn         │
├─────────────────────────────────┤
│ Target: [Select PC ▼]           │
│ • Theron (Evasion 11)           │
│ • Elara (Evasion 13)            │
│                                 │
│ Attack: +1 modifier             │
│ Damage: 1d6                     │
│                                 │
│ [🎲 Roll Attack]                │
└─────────────────────────────────┘
```

---

## 🎯 Core Daggerheart Combat Flow

### Phase 1: Start Combat
1. GM clicks "Start Combat"
2. Action Tracker appears (visual bar with tokens)
3. GM places 3 PC tokens and 3 Adversary tokens on tracker
4. Combat log starts

### Phase 2: Action Resolution
1. **Next token on tracker acts** (PC or Adversary)
2. Actor declares action (attack, spell, move, etc.)
3. Roll attack (if applicable):
   - Roll 2d12 (Hope + Fear) + modifiers
   - Compare total to target's **Evasion**
   - **Success with Hope** → PC token advances on tracker
   - **Success with Fear** → Adversary token advances
   - **Failure** → Adversary token advances
4. Apply damage (if hit):
   - Roll damage dice
   - Subtract target's **Armor Score**
   - Apply threshold rules:
     - **<5 damage** → 0 HP lost, +1 Stress
     - **5-9 damage** → 1 HP lost
     - **10-14 damage** → 2 HP lost
     - **15+ damage** → 3 HP lost
5. Repeat until combat ends

### Phase 3: End Combat
- All adversaries defeated OR
- All PCs taken out OR
- GM ends combat manually

---

## 📊 Action Tracker System

### Visual Representation
```
[PC] [PC] [PC] [ADV] [ADV] [ADV]
 ↑    →    →     →     →     ↑
Next                      Current
```

### Token Movement Rules
1. **PC Success with Hope:**
   - Dealing PC token advances 1 space
   - If at end, wraps to start

2. **PC Success with Fear OR Failure:**
   - Adversary token advances 1 space

3. **Adversary Success (any):**
   - Adversary token advances 1 space

4. **Who Acts Next:**
   - Always the **leftmost** token on the tracker
   - Remove from tracker after acting
   - When token pool empty, GM refills (3 PC, 3 Adversary)

### GM Control
- **Add tokens** - Increase PC or Adversary tokens
- **Remove tokens** - Decrease tokens
- **Reset tracker** - Start fresh
- **End combat** - Clear tracker

---

## ⚔️ Attack System

### Attack Roll
**Already implemented in engine:**
```rust
pub struct Attack {
    pub modifier: i8,        // Attribute + Proficiency
    pub with_advantage: bool,
}

pub struct AttackResult {
    pub hope: u16,           // Hope die (1-12)
    pub fear: u16,           // Fear die (1-12)
    pub modifier: i8,
    pub success: bool,       // Hope > Fear
    pub critical: bool,      // Doubles rolled
    pub total: u16,          // Controlling die + modifier
}
```

### Attack Process
1. **Declare Target** - Select character on map
2. **Calculate Modifier:**
   - Attribute (Agility or Strength for melee/ranged)
   - + Proficiency bonus
   - + Situational modifiers (GM)
3. **Roll Attack:**
   - 2d12 (Hope + Fear)
   - Compare Hope vs Fear → determines controlling die
   - Add modifier to controlling die = **Total**
4. **Compare to Evasion:**
   - Total ≥ target's Evasion → **HIT**
   - Total < target's Evasion → **MISS**
5. **Determine Next Token:**
   - Hope > Fear → PC token advances
   - Fear > Hope OR Failure → Adversary token advances

---

## 💥 Damage System

### Damage Calculation
**Already implemented in engine:**
```rust
pub struct DamageResult {
    pub raw_damage: u16,      // Weapon/spell damage
    pub after_armor: u16,     // Raw - Armor Score
    pub hp_lost: u8,          // Based on threshold
    pub stress_gained: u8,    // If below threshold
}
```

### Damage Thresholds
```
Raw Damage - Armor Score = After-Armor Damage

After-Armor < 5:
  → 0 HP lost
  → +1 Stress
  → "Scratch"

After-Armor 5-9:
  → 1 HP lost
  → 0 Stress
  → "Light wound"

After-Armor 10-14:
  → 2 HP lost
  → 0 Stress
  → "Serious wound"

After-Armor 15+:
  → 3 HP lost
  → 0 Stress
  → "Critical wound"
```

### Damage Process
1. **Roll Damage:**
   - Weapon/spell dice (e.g., 1d8 for longsword)
   - Add modifiers (e.g., Strength for melee)
2. **Apply Armor:**
   - Subtract target's Armor Score
   - Cannot reduce below 0
3. **Apply Threshold:**
   - Use table above
   - Update HP or Stress
4. **Check Knockout:**
   - If HP = 0: Mark wounded, take Stress damage next
   - If HP = 0 AND Stress = Max: **Taken Out**

---

## 🎬 Action Economy

### Action Types

#### **Major Action** (1 per turn)
- **Attack** - Melee or ranged attack roll
- **Spellcast** - Cast a spell with Major cost
- **Domain Card** - Use a Major action card
- **Disengage** - Move without opportunity attacks
- **Dash** - Double movement speed

#### **Minor Action** (1 per turn)
- **Move** - Standard movement
- **Interact** - Open door, pull lever, etc.
- **Quick Action** - Use a Minor action card
- **Draw/Stow** - Weapon or item

#### **Reaction** (1 per round, triggered)
- **Opportunity Attack** - When enemy leaves reach
- **Defensive Action** - Use reaction card/ability
- **Counter** - React to enemy action

### Action Selection UI
```
┌─────────────────────────────────┐
│ YOUR TURN                       │
├─────────────────────────────────┤
│ Major Action:                   │
│  ⚔️ [Attack]  🎯 [Spellcast]   │
│  🏃 [Dash]    🛡️ [Disengage]    │
│                                 │
│ Minor Action:                   │
│  🚶 [Move]    👋 [Interact]     │
│  🗡️ [Draw]     💼 [Stow]        │
│                                 │
│         [End Turn]              │
└─────────────────────────────────┘
```

---

## 🎮 Combat UI Design

### TV View
```
┌──────────────────────────────────────────────────────────┐
│ COMBAT: Boss Fight                           🛑 End      │
├──────────────────────────────────────────────────────────┤
│                                                          │
│ ┌──────────────────────────────────────────────────────┐│
│ │ Action Tracker:                                      ││
│ │ [PC] [PC] [ADV] [PC] [ADV] [ADV]                    ││
│ │  ↑ Next                                              ││
│ └──────────────────────────────────────────────────────┘│
│                                                          │
│ ┌──────────────────────────────────────────────────────┐│
│ │ [MAP WITH CHARACTER TOKENS]                          ││
│ │                                                      ││
│ │  Theron ⚔️ ❤️3/5 ⚡3/5    Goblin 💀 ❤️2/3          ││
│ └──────────────────────────────────────────────────────┘│
│                                                          │
│ ► Theron's turn                                         │
│   Attacking Goblin...                                   │
│   Roll: Hope 8, Fear 5 → Success with Hope!            │
│   Damage: 7 → 5 after armor → 1 HP lost                │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### Mobile View (Active Turn)
```
┌─────────────────────────────────┐
│ 🎯 YOUR TURN                    │
├─────────────────────────────────┤
│ You: Theron ⚔️                  │
│ ❤️ HP: 3/5  ⚡ Stress: 3/5      │
│                                 │
│ Target: Goblin 💀               │
│ Evasion: 10  Armor: 2           │
│                                 │
│ ┌───────────────────────────┐   │
│ │ Major Action:             │   │
│ │  ⚔️ [Attack]              │   │
│ │  🎯 [Spellcast]           │   │
│ │  🏃 [Dash]                │   │
│ └───────────────────────────┘   │
│                                 │
│ ┌───────────────────────────┐   │
│ │ Minor Action:             │   │
│ │  🚶 [Move]                │   │
│ │  👋 [Interact]            │   │
│ └───────────────────────────┘   │
│                                 │
│      [End Turn]                 │
└─────────────────────────────────┘
```

### Mobile View (Attack Roll)
```
┌─────────────────────────────────┐
│ ⚔️ ATTACK GOBLIN                │
├─────────────────────────────────┤
│ Your Modifier: +3               │
│ (Agility +2, Prof +1)           │
│                                 │
│ Target Evasion: 10              │
│                                 │
│     [🎲 Roll Attack]            │
│                                 │
│ Or...                           │
│ ☑️ Spend 1 Hope for +2          │
│ ☑️ Use Advantage                │
│                                 │
│      [Cancel]                   │
└─────────────────────────────────┘
```

### Mobile View (Damage Roll)
```
┌─────────────────────────────────┐
│ 💥 HIT! Roll Damage             │
├─────────────────────────────────┤
│ Hope: 9  Fear: 5                │
│ Total: 12 (Success with Hope!)  │
│                                 │
│ Weapon: Longsword (1d8)         │
│ Modifier: +2 (Strength)         │
│                                 │
│     [🎲 Roll Damage]            │
│                                 │
│ Result will be:                 │
│ • Rolled damage                 │
│ • - 2 (Goblin's armor)          │
│ • = Final damage                │
│                                 │
│      [Cancel]                   │
└─────────────────────────────────┘
```

### GM View
```
┌─────────────────────────────────┐
│ 👹 Adversaries                  │
├─────────────────────────────────┤
│ Template: [Goblin ▼]            │
│ • Goblin                        │
│ • Bandit                        │
│ • Orc Warrior                   │
│ • [Custom...]                   │
│                                 │
│ [➕ Spawn on Map]               │
│                                 │
│ Active Adversaries:             │
│                                 │
│ Goblin #1 💀                    │
│ ❤️ 3/3  ⚡ 0/3                  │
│ Evasion: 10  Armor: 1           │
│ [⚔️ Attack] [🗑️ Remove]         │
│                                 │
│ Goblin #2 💀                    │
│ ❤️ 2/3  ⚡ 1/3                  │
│ Evasion: 10  Armor: 1           │
│ [⚔️ Attack] [🗑️ Remove]         │
└─────────────────────────────────┘

┌─────────────────────────────────┐
│ 🎮 Combat Control               │
├─────────────────────────────────┤
│ Status: Active                  │
│ Round: 2                        │
│                                 │
│ Action Tracker:                 │
│ PC tokens: 2                    │
│ Adversary tokens: 3             │
│                                 │
│ [+ PC Token]  [+ Adv Token]     │
│ [- PC Token]  [- Adv Token]     │
│                                 │
│ [Reset Tracker]                 │
│ [🛑 End Combat]                 │
│                                 │
│ ───────────────────────────────│
│ Player Characters:              │
│                                 │
│ Theron (PC) ⚔️                  │
│ ❤️ 3/5  ⚡ 3/5                  │
│ Evasion: 11  Armor: 3           │
└─────────────────────────────────┘
```

---

## 🔧 Implementation Plan

### Phase 1: Backend (1.5 days)

#### 1.1 Combat State Management
**File:** `server/src/game.rs`

```rust
/// Combat encounter state
pub struct CombatEncounter {
    pub id: String,
    pub is_active: bool,
    pub round: u32,
    pub action_tracker: ActionTracker,
    pub combatants: Vec<String>,  // Character IDs
}

pub struct ActionTracker {
    pub pc_tokens: u8,
    pub adversary_tokens: u8,
    pub queue: VecDeque<TokenType>,  // Order of tokens
}

pub enum TokenType {
    PC,
    Adversary,
}

impl GameState {
    pub fn start_combat(&mut self) -> String;
    pub fn end_combat(&mut self);
    pub fn get_next_actor(&self) -> Option<TokenType>;
    pub fn advance_tracker(&mut self, result: &AttackResult);
    pub fn add_pc_token(&mut self);
    pub fn add_adversary_token(&mut self);
}
```

#### 1.1.5 Adversary Management
**File:** `server/src/game.rs` + `server/src/adversaries.rs` (new)

```rust
// game.rs
pub struct Adversary {
    pub id: String,
    pub name: String,
    pub template: String,
    pub position: Position,
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

impl GameState {
    pub fn spawn_adversary(&mut self, template: &str, pos: Position) -> String;
    pub fn create_custom_adversary(&mut self, stats: AdversaryStats) -> String;
    pub fn remove_adversary(&mut self, id: &str);
    pub fn get_adversaries(&self) -> Vec<&Adversary>;
    pub fn update_adversary_hp(&mut self, id: &str, new_hp: u8);
}

// adversaries.rs (new file)
pub struct AdversaryTemplate {
    pub id: String,
    pub name: String,
    pub tier: String,
    pub hp: u8,
    pub evasion: u8,
    pub armor: u8,
    pub attack_modifier: i8,
    pub damage: String,
    pub description: String,
}

pub fn load_templates() -> Vec<AdversaryTemplate>;
pub fn get_template(id: &str) -> Option<AdversaryTemplate>;
```

#### 1.2 Attack Protocol
**File:** `server/src/protocol.rs`

```rust
// Client → Server
ClientMessage::SpawnAdversary {
    template: String,  // "goblin", "bandit", "custom"
    position: Position,
    custom_stats: Option<AdversaryStats>,
}

ClientMessage::RemoveAdversary {
    adversary_id: String,
}

ClientMessage::AdversaryAttack {
    adversary_id: String,
    target_id: String,  // PC character ID
}

ClientMessage::Attack {
    attacker_id: String,
    target_id: String,
    weapon_damage_dice: String,  // "1d8"
    spend_hope: bool,
}

ClientMessage::RollDamage {
    attack_result_id: String,
}

ClientMessage::EndTurn {
    character_id: String,
}

// Server → Clients
ServerMessage::AdversarySpawned {
    adversary_id: String,
    name: String,
    template: String,
    position: Position,
    hp: u8,
    max_hp: u8,
    evasion: u8,
    armor: u8,
    attack_modifier: i8,
    damage_dice: String,
}

ServerMessage::AdversaryRemoved {
    adversary_id: String,
    name: String,
}

ServerMessage::AdversaryUpdated {
    adversary_id: String,
    hp: u8,
    stress: u8,
}

ServerMessage::CombatStarted {
    encounter_id: String,
    initial_tracker: ActionTracker,
}

ServerMessage::TurnStart {
    character_id: String,
    character_name: String,
    token_type: String,  // "PC" or "Adversary"
}

ServerMessage::AttackResult {
    attack_id: String,
    attacker_name: String,
    target_name: String,
    hope: u16,
    fear: u16,
    total: u16,
    target_evasion: u16,
    hit: bool,
    controlling_die: String,  // "hope" or "fear"
}

ServerMessage::DamageResult {
    target_name: String,
    raw_damage: u16,
    after_armor: u16,
    hp_lost: u8,
    stress_gained: u8,
    new_hp: u8,
    new_stress: u8,
}

ServerMessage::TrackerUpdated {
    pc_tokens: u8,
    adversary_tokens: u8,
    next_token: String,  // "PC" or "Adversary"
}

ServerMessage::CombatEnded {
    reason: String,  // "victory", "defeat", "manual"
}
```

#### 1.3 Attack/Damage Handlers
**File:** `server/src/websocket.rs`

```rust
async fn handle_spawn_adversary(
    state: &AppState,
    template: String,
    position: Position,
    custom_stats: Option<AdversaryStats>,
);

async fn handle_remove_adversary(
    state: &AppState,
    adversary_id: String,
);

async fn handle_adversary_attack(
    state: &AppState,
    adversary_id: String,
    target_id: String,
);

async fn handle_attack(
    state: &AppState,
    attacker_id: String,
    target_id: String,
    weapon_damage: String,
    spend_hope: bool,
);

async fn handle_roll_damage(
    state: &AppState,
    attack_result_id: String,
);

async fn handle_end_turn(
    state: &AppState,
    character_id: String,
);
```

---

### Phase 2: Frontend UI (1.5 days)

#### 2.1 TV View Combat Display
**File:** `client/index.html`

```html
<!-- Combat Header -->
<div id="combat-header" style="display: none;">
    <h2>⚔️ Combat Active</h2>
    <span>Round <span id="combat-round">1</span></span>
</div>

<!-- Action Tracker -->
<div id="action-tracker" style="display: none;">
    <h3>Action Tracker</h3>
    <div class="tracker-tokens" id="tracker-tokens">
        <!-- Dynamically populated -->
    </div>
</div>

<!-- Current Turn Display -->
<div id="current-turn" style="display: none;">
    <p>► <span id="current-actor">Theron</span>'s turn</p>
</div>
```

#### 2.2 Mobile Combat Actions
**File:** `client/mobile.html`

```html
<!-- Combat Actions Panel -->
<section id="combat-panel" style="display: none;">
    <h2>🎯 Your Turn</h2>
    
    <div id="major-actions">
        <h3>Major Action</h3>
        <button id="action-attack">⚔️ Attack</button>
        <button id="action-spell">🎯 Spellcast</button>
        <button id="action-dash">🏃 Dash</button>
    </div>
    
    <div id="minor-actions">
        <h3>Minor Action</h3>
        <button id="action-move">🚶 Move</button>
        <button id="action-interact">👋 Interact</button>
    </div>
    
    <button id="end-turn-btn">End Turn</button>
</section>

<!-- Attack Roll Panel -->
<section id="attack-panel" style="display: none;">
    <h2>⚔️ Attack</h2>
    <p>Target: <span id="attack-target">Enemy</span></p>
    <p>Evasion: <span id="target-evasion">10</span></p>
    <p>Your Modifier: <span id="attack-modifier">+3</span></p>
    
    <label>
        <input type="checkbox" id="spend-hope-attack">
        Spend 1 Hope for +2
    </label>
    
    <button id="roll-attack-btn">🎲 Roll Attack</button>
    <button id="cancel-attack-btn">Cancel</button>
</section>

<!-- Damage Roll Panel -->
<section id="damage-panel" style="display: none;">
    <h2>💥 Roll Damage</h2>
    <p>You hit! Roll damage.</p>
    <p>Weapon: <span id="weapon-name">Longsword</span> (1d8)</p>
    <p>Modifier: +<span id="damage-modifier">2</span></p>
    
    <button id="roll-damage-btn">🎲 Roll Damage</button>
</section>
```

#### 2.3 GM Combat Controls
**File:** `client/gm.html`

```html
<div class="control-panel">
    <h3>👹 Adversaries</h3>
    
    <label>Template:</label>
    <select id="adversary-template">
        <option value="goblin">Goblin (HP: 3, Evasion: 10)</option>
        <option value="bandit">Bandit (HP: 4, Evasion: 11)</option>
        <option value="wolf">Wolf (HP: 3, Evasion: 12)</option>
        <option value="orc_warrior">Orc Warrior (HP: 5, Evasion: 10)</option>
        <option value="custom">Custom...</option>
    </select>
    
    <button id="spawn-adversary-btn">➕ Spawn on Map</button>
    
    <div id="custom-adversary-panel" style="display: none;">
        <h4>Custom Adversary</h4>
        <label>Name: <input type="text" id="adv-name" value="Custom Enemy"></label>
        <label>HP: <input type="number" id="adv-hp" value="3"></label>
        <label>Evasion: <input type="number" id="adv-evasion" value="10"></label>
        <label>Armor: <input type="number" id="adv-armor" value="1"></label>
        <label>Attack Mod: <input type="number" id="adv-attack-mod" value="1"></label>
        <label>Damage: <input type="text" id="adv-damage" value="1d6"></label>
        <button id="spawn-custom-btn">✅ Spawn Custom</button>
    </div>
    
    <h4>Active Adversaries</h4>
    <div id="adversaries-list">
        <p class="empty-state">No adversaries spawned</p>
    </div>
</div>

<div class="control-panel">
    <h3>⚔️ Combat</h3>
    
    <button id="start-combat-btn">Start Combat</button>
    <button id="end-combat-btn" style="display: none;">🛑 End Combat</button>
    
    <div id="combat-controls" style="display: none;">
        <h4>Action Tracker</h4>
        <p>PC Tokens: <span id="pc-tokens">3</span></p>
        <p>Adversary Tokens: <span id="adv-tokens">3</span></p>
        
        <button id="add-pc-token">+ PC</button>
        <button id="add-adv-token">+ Adversary</button>
        <button id="remove-pc-token">- PC</button>
        <button id="remove-adv-token">- Adversary</button>
        
        <button id="reset-tracker">Reset Tracker</button>
    </div>
</div>
```

---

### Phase 3: Integration & Polish (1 day)

#### 3.1 Combat Flow JavaScript
**File:** `client/js/combat.js` (new file)

```javascript
class CombatManager {
    constructor() {
        this.isActive = false;
        this.currentTurn = null;
        this.tracker = null;
    }
    
    startCombat(encounterData) {
        // Show combat UI
        // Initialize tracker
        // Log event
    }
    
    handleTurnStart(data) {
        // Highlight current actor
        // Enable actions for player
        // Show turn indicator
    }
    
    handleAttack(attackerId, targetId) {
        // Send attack message
        // Show attack roll UI
    }
    
    handleAttackResult(result) {
        // Display result on TV
        // Enable damage roll if hit
        // Update tracker
    }
    
    handleDamageResult(result) {
        // Update HP/Stress displays
        // Log damage
        // Advance to next turn
    }
    
    endCombat(reason) {
        // Hide combat UI
        // Show summary
        // Log event
    }
}

class AdversaryManager {
    constructor() {
        this.adversaries = new Map();
        this.templates = null;
    }
    
    async loadTemplates() {
        // Fetch adversary templates from server
        // Store in this.templates
    }
    
    spawnAdversary(template, position) {
        // Send spawn message to server
        // Add to local map
    }
    
    handleAdversarySpawned(data) {
        // Add adversary to map
        // Render token on canvas
        // Update GM adversary list
        // Log event
    }
    
    removeAdversary(id) {
        // Send remove message
        // Remove from map
        // Remove token from canvas
    }
    
    handleAdversaryUpdated(data) {
        // Update HP/Stress displays
        // Update canvas token
    }
    
    rollAdversaryAttack(adversaryId, targetId) {
        // Send adversary attack message
        // Show roll result
    }
}
```

#### 3.2 Visual Feedback
- **Attack animations** - Flash on hit
- **Damage numbers** - Pop up on character tokens
- **HP bars** - Real-time health updates
- **Tracker animation** - Tokens move smoothly
- **Turn indicator** - Highlight active character

#### 3.3 Event Logging
Every combat action logged:
- "Combat started"
- "Theron attacks Goblin"
- "Hit! Hope: 9, Fear: 5"
- "Dealt 7 damage (5 after armor) → 1 HP lost"
- "PC token advances"
- "Goblin's turn"
- "Combat ended: Victory!"

---

## 📝 Example Combat Flow

### Pre-Combat Setup
1. **GM opens adversary panel**
2. **Selects "Goblin" template** from dropdown
3. **Clicks on map** where goblin should appear (position: 10, 10)
4. Server creates adversary: `id: "adv-001", name: "Goblin #1"`
5. **Broadcasts `adversary_spawned`** to all clients
6. **Goblin token appears** on TV and mobile maps
7. **GM spawns second goblin** at position (15, 10)
8. **Event log:** "Goblin #1 spawned", "Goblin #2 spawned"

### Combat Start
1. GM clicks "Start Combat"
2. Server creates encounter
3. Broadcasts `combat_started`
4. Action Tracker initializes: [PC] [PC] [PC] [ADV] [ADV] [ADV]

### Round 1
1. **PC token is next** → Theron's turn starts
2. Mobile shows "Your Turn" with actions
3. Player taps "⚔️ Attack", selects Goblin
4. Player taps "🎲 Roll Attack"
5. Server rolls: Hope 9, Fear 5, Modifier +3 = 12
6. Target Evasion 10 → **HIT**
7. Success with Hope → **PC token advances**
8. Mobile shows "Roll Damage"
9. Player taps "🎲 Roll Damage"
10. Server rolls 1d8 + 2 = 7 damage
11. Goblin armor 2 → 5 after armor
12. 5-9 range → **1 HP lost**
13. Goblin HP 3 → 2
14. Event log: "Theron hit Goblin for 5 damage (1 HP lost)"

### Round 2
1. **PC token is next** (advanced from last success) → Elara's turn
2. (Repeat flow...)

---

## 🎯 Success Criteria

### Must Have
- [x] **Adversary spawning system** - GM can create and place enemies
- [x] **Adversary templates** - Pre-defined enemy types (Goblin, Bandit, etc.)
- [x] **Custom adversaries** - GM can create custom enemies with stats
- [x] Action Tracker with PC/Adversary tokens
- [x] Attack rolls using Duality system
- [x] Damage calculation with thresholds
- [x] HP and Stress tracking
- [x] Turn-by-turn flow
- [x] Mobile action selection
- [x] TV combat display (including adversary tokens)
- [x] GM combat controls
- [x] **GM adversary attacks** - Roll attacks for enemies
- [x] Event logging for all actions

### Nice to Have
- [ ] Adversary AI (auto-attacks)
- [ ] Opportunity attacks
- [ ] Movement tracking
- [ ] Range checking
- [ ] Status effects
- [ ] Domain card integration
- [ ] Reaction system
- [ ] Save/load adversary templates

---

## 🚀 Implementation Order

1. **Day 1 Morning:** 
   - Backend combat state + action tracker
   - **Adversary data structures + templates**
2. **Day 1 Afternoon:** 
   - Attack/damage handlers + protocol
   - **Adversary spawning/removal handlers**
3. **Day 2 Morning:** 
   - **GM adversary panel (spawn UI)**
   - Mobile combat UI (actions, rolls)
4. **Day 2 Afternoon:** 
   - TV combat display + tracker visualization
   - **Adversary tokens on canvas**
5. **Day 3 Morning:** 
   - GM controls + integration
   - **Adversary attack rolls**
6. **Day 3 Afternoon:** 
   - Polish + testing + bug fixes
   - **Event logging for adversary actions**

---

## 📚 References

- **Engine Code:** `daggerheart-engine/src/combat/`
- **Attack System:** `attack.rs`
- **Damage System:** `damage.rs`
- **Duality Dice:** `core/dice/duality.rs`

---

**Ready to implement?** This is the full Daggerheart combat system spec, true to the game's unique mechanics!
