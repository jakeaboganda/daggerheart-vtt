# Gameplay Features Roadmap

## Current State
✅ Character creation (name, class, ancestry, attributes)
✅ Token movement on map
✅ Player-initiated dice rolls (duality dice)
✅ Basic resource tracking (HP, Stress, Hope)
✅ GM view with player monitoring

## Missing Core Gameplay

---

## 1. GM-Initiated Dice Rolls ⭐ HIGH PRIORITY

### Current Problem
- Players can roll whenever they want
- Success threshold is hardcoded (typically 12)
- No context for why they're rolling
- GM has no control over when/what to roll

### Proposed Solution

#### GM Actions
```typescript
// GM initiates a roll request
{
  type: "request_roll",
  payload: {
    target_character_ids: ["char-123", "char-456"], // or "all"
    roll_type: "duality",
    attribute: "agility",  // optional: which attribute to use
    difficulty: 15,         // success threshold
    context: "Leap across the chasm",
    with_advantage: false,
    with_disadvantage: false
  }
}
```

#### Player Response
- Mobile UI shows: **"GM requests roll: Leap across the chasm (DC 15)"**
- Player can see which attribute to add
- **One button:** "Roll" (applies modifiers automatically)
- Can't dismiss until rolled (or GM cancels)

#### Result Broadcast
```typescript
{
  type: "roll_result",
  payload: {
    character_id: "char-123",
    character_name: "Theron",
    context: "Leap across the chasm",
    difficulty: 15,
    roll: { /* RollResult */ },
    outcome: "success" | "failure" | "critical_success" | "critical_failure"
  }
}
```

#### Implementation Plan
1. Add `RequestRoll` to `ClientMessage` (GM only)
2. Add `RollRequest` to `ServerMessage` (pushed to specific players)
3. Mobile UI: Show modal with roll request
4. GM UI: Show pending rolls + results in feed
5. Store difficulty in roll context
6. Calculate success/failure on server

---

## 2. Map Story Beats & Interactions ⭐ HIGH PRIORITY

### Current Problem
- Map is just an empty canvas
- No context or story elements
- No objectives or points of interest

### Proposed Solution

#### Story Beat Types
1. **Markers/Pins** - Points of interest
   - Quest objectives
   - NPCs
   - Loot/treasure
   - Hazards/traps

2. **Zones** - Area-based triggers
   - Combat encounters
   - Environmental effects
   - Safe zones / rest areas
   - Secret areas

3. **Interactive Objects**
   - Doors (locked/unlocked)
   - Levers/switches
   - Treasure chests
   - Campfires

#### GM Interface
```
[Map Editor Panel]
- Add Marker
  - Icon: 🗡️ ⚔️ 💰 🚪 🔥 ⭐ ❓
  - Label: "Ancient Door"
  - Type: Interactive | Info | Danger
  
- Add Zone
  - Shape: Circle | Rectangle | Polygon
  - Effect: Combat | Hazard | Story
  - Trigger: On Enter | On Exit | Manual
  
- Pre-set Maps
  - Load: Tavern | Forest | Dungeon | Castle
  - Comes with predefined story beats
```

#### Player Interaction
- When player token **enters a zone**: Server broadcasts event
- GM can **manually trigger** story beats
- Interactive objects show **action prompt** on mobile

#### Data Structure
```rust
struct StoryBeat {
    id: Uuid,
    beat_type: BeatType, // Marker | Zone | Interactive
    position: Position,  // for markers/objects
    zone_shape: Option<Shape>, // for zones
    icon: String,
    label: String,
    description: String,
    trigger_type: TriggerType, // OnEnter | Manual | Proximity
    state: BeatState, // Active | Completed | Hidden
}
```

#### Implementation Plan
1. Add `StoryBeat` struct to game state
2. GM UI: Story beat editor panel
3. Render beats on canvas (different from tokens)
4. Collision detection for zones
5. Event system for triggers
6. Pre-made map templates with beats

---

## 3. Character Backstories & Details 📝 MEDIUM PRIORITY

### Current Problem
- Characters are just stats
- No personality or story
- Players can't express who their character is

### Proposed Solution

#### Extended Character Profile
```rust
struct CharacterProfile {
    // Existing fields...
    name: String,
    class: Class,
    ancestry: Ancestry,
    
    // NEW: Story fields
    backstory: Option<String>,      // Freeform text (500 char limit)
    motivation: Option<String>,      // "Why are you here?"
    connection: Option<String>,      // Connection to another PC
    fear: Option<String>,            // Character's fear/flaw
    
    // NEW: Visual
    portrait_url: Option<String>,    // Avatar image URL
    token_style: TokenStyle,         // Custom token appearance
}
```

#### UI Changes
1. **Character Creation:** Add optional backstory step
   - "Tell us about your character (optional)"
   - Templated prompts: "I seek..." / "I fear..." / "I left behind..."

2. **Character Sheet View (Mobile):**
   - Tab system: Stats | Story | Abilities
   - Story tab shows backstory, motivation, etc.

3. **GM View:**
   - Click character → See full profile
   - Quick reference for roleplaying

#### Implementation
- Add fields to `Character` struct
- Update creation flow UI
- Add story tab to mobile character sheet
- Store in save files

---

## 4. Traits & Abilities System ⭐⭐ HIGH PRIORITY

### Current Problem
- Characters have no special abilities
- Just rolling dice with stat modifiers
- No Daggerheart "cards" (Domain cards, etc.)

### Proposed Solution

#### Data Model
```rust
struct CharacterAbility {
    id: String,
    name: String,
    ability_type: AbilityType, // Action | Reaction | Passive
    description: String,
    cost: Option<AbilityCost>, // Hope cost, HP cost, etc.
    cooldown: Option<u32>,     // rounds
    range: Option<String>,     // "Melee" | "Close" | "Far"
    
    // Usage tracking
    uses_per_rest: Option<u32>,
    current_uses: u32,
    on_cooldown: bool,
}

enum AbilityType {
    Action,      // Use on your turn
    Reaction,    // Use in response to trigger
    Passive,     // Always active
}

enum AbilityCost {
    Hope(u8),
    HP(u8),
    Stress(u8),
    Free,
}
```

#### Player Interface (Mobile)
```
[Character Sheet - Abilities Tab]

⚔️ Actions
  ┌─────────────────────────────┐
  │ 🗡️ Mighty Strike            │
  │ Cost: 1 Hope                │
  │ Deal +2 damage on hit       │
  │ [Use Ability] ✅            │
  └─────────────────────────────┘
  
🛡️ Reactions
  ┌─────────────────────────────┐
  │ 🛡️ Defensive Stance (1/rest)│
  │ +2 Evasion until next turn  │
  │ [Ready] (used: 0/1)         │
  └─────────────────────────────┘

🌟 Passive Traits
  • Darkvision
  • Armor Proficiency (Medium)
```

#### GM View
- See all active abilities/buffs
- Track cooldowns and uses
- Ability usage log in timeline

#### Integration with daggerheart-engine
Check if engine already has cards/abilities:
```bash
cd daggerheart-engine
grep -r "card\|ability\|trait" src/
```

If cards exist in engine → map to UI
If not → create lightweight ability system in VTT

#### Implementation Plan
1. Check what exists in `daggerheart-engine/src/cards/`
2. Add `abilities: Vec<CharacterAbility>` to Character
3. Create abilities panel in mobile UI
4. "Use Ability" button → sends to server
5. Server validates (has cost? not on cooldown?)
6. Broadcast ability usage to all clients
7. Auto-reset uses on rest

---

## Implementation Priority

### Phase 5A (Next): GM Control & Story Beats
1. ✅ GM-initiated dice rolls with difficulty
2. ✅ Story beat markers on map
3. ✅ Basic zone triggers

### Phase 5B: Character Depth
4. Character backstories
5. Traits/abilities system (basic)
6. Ability usage tracking

### Phase 5C: Advanced Features
7. Pre-made maps with story beats
8. Ability effects (buffs/debuffs)
9. Advanced zone interactions
10. Campaign/session notes

---

## Questions for You

1. **Dice Rolls:** Should players ever be able to roll freely, or should ALL rolls be GM-initiated?

2. **Story Beats:** Do you want pre-made maps, or just tools to build your own?

3. **Abilities:** Should we:
   - Pull from daggerheart-engine cards?
   - Let GM create custom abilities per character?
   - Have class-specific ability lists?

4. **Backstory:** Where should it be visible?
   - Just in character sheet?
   - Shared with other players?
   - Only visible to GM?

Let me know which features you want first, and I'll start implementing! 🚀
