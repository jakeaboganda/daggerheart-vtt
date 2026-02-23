# Implementation Plan: GM Controls & Player Abilities

## ✅ What Already Exists (daggerheart-engine)

The engine has a complete card/ability system:

### Domain Cards
- **9 Domains:** Arcana, Blade, Bone, Codex, Grace, Midnight, Sage, Splendor, Valor
- **Each class has 2 domains** (e.g., Warrior = Blade + Bone)
- **Action costs:** Major, Minor, Reaction, Free
- **Range categories:** VeryClose, Close, Far, Any
- **Target types:** Self, Ally, Enemy, AllAllies, AllEnemies, Any

### Already Implemented
```rust
struct DomainCard {
    id: String,
    name: String,
    domain: Domain,
    level_requirement: u8,
    description: String,
    action_cost: ActionCost,
}
```

## 🚀 Implementation Order

---

## PHASE 1: GM-Initiated Dice Rolls (1-2 days)

**Why First:** Core to actual gameplay, blocks other features

### Server Changes
1. Add new protocol messages:
   ```rust
   // GM → Server
   ClientMessage::RequestRoll {
       target_character_ids: Vec<String>, // or "all"
       attribute: Option<String>,         // e.g. "agility"
       difficulty: i32,
       context: String,
       with_advantage: bool,
   }
   
   // Server → Players
   ServerMessage::RollRequested {
       request_id: String,
       attribute: Option<String>,
       difficulty: i32,
       context: String,
       with_advantage: bool,
   }
   
   // Player → Server (existing, enhanced)
   ClientMessage::RollDuality {
       request_id: Option<String>,  // links to GM request
       modifier: i32,
       with_advantage: bool,
   }
   
   // Server → All (enhanced)
   ServerMessage::RollResult {
       character_id: String,
       character_name: String,
       roll: RollResult,
       difficulty: Option<i32>,       // NEW
       outcome: Option<String>,        // NEW: "success" | "failure" | "crit"
       context: Option<String>,        // NEW: why they rolled
   }
   ```

2. Game state tracking:
   ```rust
   struct PendingRollRequest {
       id: String,
       character_ids: Vec<String>,
       attribute: Option<String>,
       difficulty: i32,
       context: String,
       timestamp: DateTime,
   }
   ```

### GM UI Changes (`gm.html`)
- **Roll Request Panel:**
  ```
  ┌───────────────────────────────────┐
  │ REQUEST DICE ROLL                 │
  ├───────────────────────────────────┤
  │ Target: [Dropdown: All / Select]  │
  │ Attribute: [Agi/Str/Fin/...]      │
  │ Difficulty: [12▼]                 │
  │ Context: [Jump the chasm___]      │
  │ Advantage: [Yes ☐] [No ☑]         │
  │                                   │
  │         [Request Roll] 🎲         │
  └───────────────────────────────────┘
  ```

- **Pending Rolls Feed:**
  ```
  ⏳ Waiting for rolls...
  • Theron - Jump the chasm (DC 12)
  • Elena - Spot the trap (DC 15)
  
  ✅ Recent Results
  • Theron rolled 14 - SUCCESS
  • Elena rolled 11 - FAILURE
  ```

### Mobile UI Changes (`mobile.html`)
- **Roll Request Modal** (blocking):
  ```
  ┌─────────────────────────────────┐
  │ 🎲 GM REQUESTS ROLL             │
  ├─────────────────────────────────┤
  │                                 │
  │  "Jump across the chasm"        │
  │                                 │
  │  Difficulty: 12                 │
  │  Use Attribute: Agility (+2)    │
  │                                 │
  │  Total Modifier: +2             │
  │                                 │
  │      [ROLL DICE] 🎲             │
  │                                 │
  └─────────────────────────────────┘
  ```

### Implementation Steps
1. ✅ Update `protocol.rs` with new messages
2. ✅ Add roll request logic to `websocket.rs`
3. ✅ Create GM roll request UI
4. ✅ Create mobile roll request modal
5. ✅ Link rolls to requests (pass request_id)
6. ✅ Calculate success/failure on server
7. ✅ Display outcome in GM view

---

## PHASE 2: Map Story Beats (2-3 days)

**Why Second:** Enables GM to create narrative structure

### Data Model
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryBeat {
    pub id: Uuid,
    pub beat_type: BeatType,
    pub position: Position,      // x, y on map
    pub label: String,
    pub description: String,
    pub icon: String,            // emoji or icon code
    pub state: BeatState,
    pub trigger: TriggerType,
    pub zone_radius: Option<f32>, // for proximity triggers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeatType {
    Marker,        // Point of interest
    Zone,          // Area trigger
    Interactive,   // Object (door, chest, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeatState {
    Active,
    Completed,
    Hidden,        // GM placed but not visible to players yet
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    OnEnter,       // Character enters zone
    Manual,        // GM triggers manually
    Proximity,     // Character gets close
}
```

### Protocol Messages
```rust
// GM → Server
ClientMessage::CreateStoryBeat {
    beat_type: BeatType,
    position: Position,
    label: String,
    description: String,
    icon: String,
    trigger: TriggerType,
    zone_radius: Option<f32>,
}

ClientMessage::TriggerStoryBeat {
    beat_id: String,
}

ClientMessage::UpdateBeatState {
    beat_id: String,
    state: BeatState,
}

// Server → All
ServerMessage::StoryBeatCreated {
    beat: StoryBeat,
}

ServerMessage::StoryBeatTriggered {
    beat_id: String,
    label: String,
    description: String,
    triggered_by: Option<String>, // character name
}

ServerMessage::StoryBeatUpdated {
    beat_id: String,
    state: BeatState,
}
```

### GM UI - Map Editor
```
[Sidebar Panel - Story Beats]

➕ ADD STORY BEAT
  Type: ● Marker ○ Zone ○ Interactive
  Icon: [🗡️▼] (emoji picker)
  Label: [Ancient Door_________]
  Desc:  [A massive stone door...]
  
  Trigger: ○ Manual ● On Enter ○ Proximity
  Radius: [50px___] (if Zone/Proximity)
  
  Visible: ☑ Show to players
  
  [Place on Map] (cursor changes to crosshair)

━━━━━━━━━━━━━━━━━━━━━━

PLACED BEATS
  🗡️ Ancient Door (Hidden)
     [Show] [Trigger] [Delete]
     
  💰 Treasure Chest (Active)
     [Hide] [Complete] [Delete]
     
  ⚔️ Combat Zone (Active)
     [Trigger] [Delete]
```

### Canvas Rendering
- Draw story beats as icons/shapes above the map
- Different visual styles for beat types:
  - **Marker:** Icon with label
  - **Zone:** Semi-transparent circle/rectangle
  - **Interactive:** Icon with glow effect
- Hidden beats only visible in GM view
- Completed beats rendered differently (grayed out, checkmark)

### Proximity Detection
```rust
impl GameState {
    pub fn check_story_beat_triggers(&self) -> Vec<(Uuid, String)> {
        let mut triggered = vec![];
        
        for beat in &self.story_beats {
            if beat.state != BeatState::Active {
                continue;
            }
            
            match beat.trigger {
                TriggerType::OnEnter | TriggerType::Proximity => {
                    for character in &self.characters {
                        let distance = calculate_distance(
                            character.position,
                            beat.position
                        );
                        
                        let threshold = beat.zone_radius.unwrap_or(30.0);
                        
                        if distance <= threshold {
                            triggered.push((beat.id, character.name.clone()));
                        }
                    }
                }
                TriggerType::Manual => {} // GM must trigger manually
            }
        }
        
        triggered
    }
}
```

### Implementation Steps
1. ✅ Add `StoryBeat` struct to `game.rs`
2. ✅ Add protocol messages
3. ✅ Create GM story beat editor UI
4. ✅ Implement beat placement (click-to-place)
5. ✅ Render beats on canvas (both views)
6. ✅ Implement proximity detection
7. ✅ Auto-trigger on character movement
8. ✅ Manual trigger button for GM
9. ✅ Beat state management (hide/show/complete)

---

## PHASE 3: Character Abilities (2-3 days)

**Why Third:** Builds on domain system already in engine

### Link Characters to Domain Cards
```rust
impl Character {
    // NEW field
    pub abilities: Vec<DomainCard>,
    
    pub fn grant_ability(&mut self, card: DomainCard) {
        if card.can_use(self.level) {
            self.abilities.push(card);
        }
    }
    
    pub fn available_abilities(&self) -> Vec<&DomainCard> {
        self.abilities.iter()
            .filter(|card| card.can_use(self.level))
            .collect()
    }
}
```

### Starter Abilities by Class
Create a catalog of level-1 abilities for each class:
```rust
// In a new file: server/src/abilities_catalog.rs

pub fn starter_abilities(class: Class) -> Vec<DomainCard> {
    match class {
        Class::Warrior => vec![
            DomainCard::new(
                "mighty_strike",
                "Mighty Strike",
                Domain::Blade,
                1,
                "Deal +2 damage on a successful attack",
                ActionCost::Major,
            ),
            DomainCard::new(
                "defensive_stance",
                "Defensive Stance",
                Domain::Bone,
                1,
                "+2 Evasion until end of turn (1/rest)",
                ActionCost::Minor,
            ),
        ],
        // ... other classes
    }
}
```

### Protocol Messages
```rust
// Server → Client (on character creation/load)
ServerMessage::CharacterAbilities {
    character_id: String,
    abilities: Vec<DomainCard>,
}

// Client → Server (player uses ability)
ClientMessage::UseAbility {
    ability_id: String,
    target_id: Option<String>, // optional target character
}

// Server → All (broadcast ability use)
ServerMessage::AbilityUsed {
    character_id: String,
    character_name: String,
    ability: DomainCard,
    target_id: Option<String>,
}
```

### Mobile UI - Abilities Tab
Add tab to character sheet:
```
[Stats] [Abilities] [Story]

━━━━━━━━━━━━━━━━━━━━━━━━━

⚔️ MAJOR ACTIONS

┌────────────────────────────┐
│ 🗡️ Mighty Strike           │
│ Cost: 1 Hope               │
│                            │
│ Deal +2 damage on a        │
│ successful attack.         │
│                            │
│ Hope: 3/5  [USE] 🎲        │
└────────────────────────────┘

🛡️ MINOR ACTIONS

┌────────────────────────────┐
│ 🛡️ Defensive Stance        │
│ Uses: 0/1 per rest         │
│                            │
│ +2 Evasion until end       │
│ of your turn.              │
│                            │
│       [USE] ✅             │
└────────────────────────────┘

⚡ REACTIONS

┌────────────────────────────┐
│ ⚡ Reactive Strike          │
│ Free                       │
│                            │
│ When attacked, make a      │
│ counterattack.             │
│                            │
│      [READY] 💫            │
└────────────────────────────┘
```

### GM View - Ability Log
Show ability usage in timeline:
```
🎲 Recent Actions

10:35 - Theron used Mighty Strike (spent 1 Hope: 4→3)
10:34 - Elena used Healing Word on Theron (HP: 4→6)
10:32 - Grax used Defensive Stance (Evasion: 12→14)
```

### Implementation Steps
1. ✅ Create `abilities_catalog.rs` with starter abilities
2. ✅ Add `abilities: Vec<DomainCard>` to `Character`
3. ✅ Grant starter abilities on character creation
4. ✅ Send abilities to client on character load
5. ✅ Create Abilities tab in mobile UI
6. ✅ Implement "Use Ability" button
7. ✅ Validate ability usage (cost, cooldowns)
8. ✅ Broadcast ability usage to all clients
9. ✅ Update Hope/resources when ability used
10. ✅ Show ability log in GM view

---

## PHASE 4: Character Backstories (1 day)

**Why Last:** Nice-to-have, doesn't block gameplay

### Data Model
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStory {
    pub backstory: Option<String>,      // max 500 chars
    pub motivation: Option<String>,     // max 200 chars
    pub connection: Option<String>,     // to another PC
    pub fear: Option<String>,           // max 200 chars
}

impl Character {
    pub story: Option<CharacterStory>,
}
```

### Character Creation Flow
Add optional backstory step after attributes:
```
[Step 3: Story (Optional)]

Tell us about your character...

Backstory:
[_________________________________]
[_________________________________]
[_________________________________]
(500 character limit)

What drives you?
[_________________________________]
(200 character limit)

[ Skip ] [ Continue ]
```

### Mobile UI - Story Tab
```
[Stats] [Abilities] [Story]

━━━━━━━━━━━━━━━━━━━━━━━━━

📖 BACKSTORY

Theron grew up in the mountain
villages of Greyreach, training
as a smith's apprentice until
raiders destroyed his home...

━━━━━━━━━━━━━━━━━━━━━━━━━

💭 MOTIVATION

"I seek redemption for failing
to protect my family."

━━━━━━━━━━━━━━━━━━━━━━━━━

🤝 CONNECTIONS

Connected to Elena (childhood
friends from Greyreach)

━━━━━━━━━━━━━━━━━━━━━━━━━

😨 FEARS

"I fear I will fail again when
it matters most."
```

### GM View
Click character token → show full profile including story

---

## Timeline Summary

- **Week 1:** GM-initiated rolls + Story beats (foundations)
- **Week 2:** Abilities system + Backstories (player depth)
- **Week 3:** Polish, testing, pre-made maps

**Total:** ~2-3 weeks to full gameplay

---

## Quick Wins (Do These First)

1. ✅ GM dice roll requests (highest priority)
2. ✅ Basic story beat markers (GM can place, players see)
3. ✅ 3-5 starter abilities per class

This gets you to **actually playable** fast, then iterate.

Want me to start with Phase 1 (GM dice rolls)? 🎲
