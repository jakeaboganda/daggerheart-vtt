# GM-Initiated Dice Rolls - Detailed Specification

**Version:** 1.0  
**Date:** 2026-02-23  
**Status:** Ready for Implementation  

---

## Table of Contents

1. [Overview](#overview)
2. [Daggerheart Roll Mechanics (Reference)](#daggerheart-roll-mechanics-reference)
3. [When GMs Call for Rolls](#when-gms-call-for-rolls)
4. [Roll Request Workflow](#roll-request-workflow)
5. [Protocol Specification](#protocol-specification)
6. [Difficulty Guidelines](#difficulty-guidelines)
7. [Modifiers & Attributes](#modifiers--attributes)
8. [Advantage/Disadvantage](#advantagedisadvantage)
9. [Success Resolution](#success-resolution)
10. [Hope/Fear Tracking](#hopefear-tracking)
11. [Experience Spending](#experience-spending)
12. [Combat vs Non-Combat](#combat-vs-non-combat)
13. [UI/UX Specification](#uiux-specification)
14. [Edge Cases](#edge-cases)
15. [Implementation Checklist](#implementation-checklist)

---

## Overview

In Daggerheart, the GM calls for rolls when the outcome is **uncertain** and **meaningful**. Players don't roll whenever they want — rolls are requested by the GM in response to player-declared actions.

### Core Principle
> **"Say what you do, GM calls for a roll if needed"**

This VTT implementation must:
- Give the GM full control over when rolls happen
- Apply Daggerheart rules correctly (2d12 Hope/Fear, doubles = crit)
- Track Hope/Fear resources properly
- Show clear success/failure outcomes
- Support both combat and non-combat scenarios

---

## Daggerheart Roll Mechanics (Reference)

### The Duality Dice Roll

**Procedure:**
1. Roll **2d12** (one Hope die, one Fear die)
2. Add **attribute modifier** (e.g., Agility +2)
3. Add **proficiency bonus** (if applicable, value TBD — default +1)
4. Add **situational modifiers** (GM-granted bonuses/penalties)
5. If **Advantage**: Roll +1d6 and add to total
6. Compare **total** to **difficulty**

**Example:**
```
Hope die: 8
Fear die: 5
Attribute (Agility): +2
Proficiency: +1
Situational: +0
─────────────────
Total: 16
```

### Success Types

| Condition | Result | Effects |
|-----------|--------|---------|
| **Total < Difficulty** | **FAILURE** | Action fails, no Hope/Fear gained |
| **Total ≥ Difficulty** + **Hope > Fear** | **SUCCESS WITH HOPE** | ✅ Success<br>🌟 +1 Hope to player<br>📖 Positive narrative outcome<br>⚔️ Player keeps initiative (combat) |
| **Total ≥ Difficulty** + **Fear > Hope** | **SUCCESS WITH FEAR** | ✅ Success<br>😈 +1 Fear to GM<br>⚠️ Success with complication<br>⚔️ Initiative → enemies (combat) |
| **Total ≥ Difficulty** + **Hope == Fear** (doubles) | **CRITICAL SUCCESS** | ✅✅ Extraordinary success<br>🎉 No Hope/Fear gain<br>📖 Exceptional outcome<br>💥 Special effects (GM discretion) |

**Critical Success Note:**
- **ANY doubles** on the 2d12 (1+1, 2+2, ... 12+12) = critical
- Even if total is low, doubles = critical success
- Example: Rolling 3+3 = 6 total vs DC 12 → **Critical Success**

### Tie Resolution (Hope == Fear, non-doubles)
When Hope die value equals Fear die value **but they're not the same number on both dice**:
- Example: Hope=7, Fear=7 (same value, **IS** doubles) → Critical
- ⚠️ This is always doubles in Daggerheart

**Clarification:** In 2d12 duality, if Hope == Fear, it's **always** doubles (same number rolled on both dice). There are no "tie but not doubles" situations.

---

## When GMs Call for Rolls

### The Golden Rule
GMs call for rolls when:
1. **Outcome is uncertain** (not automatic success/failure)
2. **Stakes matter** (failure has consequences)
3. **Player declared an action** (not just passive observation)

### Examples of When to Call for Rolls

**✅ DO call for rolls:**
- "I leap across the chasm" → Agility roll
- "I try to convince the guard to let us pass" → Presence roll
- "I search the room for traps" → Instinct roll
- "I swing my sword at the goblin" → Strength/Finesse roll (combat)
- "I try to remember the ancient legend" → Knowledge roll

**❌ DON'T call for rolls:**
- "I open the unlocked door" → Automatic success
- "I try to lift a mountain" → Automatic failure (impossible)
- "I walk down the hallway" → No roll needed (trivial)
- "What do I see?" → GM description, not an action

### Passive vs Active
- **Passive observation** → GM describes, no roll
- **Active investigation** → GM may call for roll

---

## Roll Request Workflow

### Step-by-Step Flow

```
1. PLAYER: Declares action in narrative
   "I leap across the chasm to reach the ledge!"

2. GM: Evaluates if roll is needed
   - Is outcome uncertain? YES
   - Are stakes meaningful? YES (falling = damage)
   → Call for roll

3. GM: Determines parameters
   - Relevant attribute: Agility
   - Difficulty: 14 (moderate with consequences)
   - Modifiers: None (straight roll)
   - Advantage/Disadvantage: None

4. GM: Sends roll request via VTT
   - Target: Specific player (Theron)
   - Attribute: Agility
   - Difficulty: 14
   - Context: "Leap across the chasm"

5. PLAYER: Receives roll request on mobile
   - Sees: "GM requests roll: Leap across the chasm"
   - Sees: Difficulty 14, using Agility (+2)
   - Option: Spend Hope for +2 (if has Experience)
   - Clicks: [ROLL DICE]

6. SERVER: Executes roll
   - Rolls 2d12 (Hope + Fear)
   - Adds Agility (+2)
   - Adds Proficiency (+1, if applicable)
   - Calculates total
   - Determines success type
   - Updates Hope/Fear resources

7. ALL CLIENTS: See result
   - GM view: Full breakdown + Hope/Fear updates
   - Player view: Result + narrative outcome
   - Other players: See the roll in activity feed

8. GM: Narrates outcome
   - Success with Hope: "You land gracefully, momentum with you!"
   - Success with Fear: "You make it, but your ankle twists painfully..."
   - Critical: "You soar across with incredible grace, inspiring your allies!"
   - Failure: "You misjudge the distance and plummet into the ravine..."
```

---

## Protocol Specification

### New Messages

#### 1. GM → Server: Request Roll

```typescript
ClientMessage::RequestRoll {
    // WHO
    target_type: RollTargetType,  // "specific" | "all" | "npc"
    target_character_ids: Vec<String>,  // IDs if specific
    
    // WHAT
    roll_type: RollType,          // "action" | "attack" | "spellcast" | "save"
    attribute: Option<String>,     // "agility" | "strength" | etc. (null for attacks)
    difficulty: u16,               // Target number (typically 12-18)
    
    // CONTEXT
    context: String,               // "Leap across the chasm" (max 200 chars)
    narrative_stakes: Option<String>,  // Optional: "Fall damage if you fail"
    
    // MODIFIERS
    situational_modifier: i8,      // +/- bonus/penalty (default 0)
    has_advantage: bool,           // Roll +1d6
    has_disadvantage: bool,        // Cannot have both advantage and disadvantage
    
    // METADATA
    request_id: String,            // UUID for tracking
    timestamp: u64,                // Unix timestamp
    is_combat: bool,               // Affects initiative flow
}

enum RollTargetType {
    Specific,     // One or more specific PCs
    All,          // All player characters
    NPC,          // GM-controlled character
}

enum RollType {
    Action,       // General action check (use attribute)
    Attack,       // Melee/ranged attack (use weapon proficiency)
    Spellcast,    // Casting a spell (difficulty from spell)
    Save,         // Reactive save (use attribute)
}
```

**Validation Rules:**
- `difficulty` must be 1-30 (typically 12-18)
- `context` must not be empty, max 200 chars
- `attribute` required if `roll_type == Action || Save`
- `target_character_ids` must not be empty
- Cannot have both `has_advantage` and `has_disadvantage`

---

#### 2. Server → Player: Roll Requested

```typescript
ServerMessage::RollRequested {
    // METADATA
    request_id: String,
    
    // WHAT TO ROLL
    roll_type: RollType,
    attribute: Option<String>,     // "agility" etc.
    difficulty: u16,
    
    // CONTEXT
    context: String,               // "Leap across the chasm"
    narrative_stakes: Option<String>,
    
    // MODIFIERS (pre-calculated by server)
    base_modifier: i8,             // Attribute + Proficiency
    situational_modifier: i8,      // GM-granted bonus/penalty
    total_modifier: i8,            // Sum of all modifiers
    has_advantage: bool,
    
    // CHARACTER INFO (for player's reference)
    your_attribute_value: i8,      // e.g., Agility +2
    your_proficiency: i8,          // e.g., +1
    
    // OPTIONS
    can_spend_hope: bool,          // True if player has Hope and relevant Experience
    relevant_experiences: Vec<String>,  // Experiences that could apply
    
    // TIMING
    expires_at: Option<u64>,       // Optional expiry (for combat urgency)
}
```

---

#### 3. Player → Server: Execute Roll

```typescript
ClientMessage::ExecuteRoll {
    request_id: String,            // Links to RollRequested
    
    // PLAYER CHOICES
    spend_hope_for_bonus: bool,    // Spend 1 Hope for +2 (via Experience)
    chosen_experience: Option<String>,  // Which Experience justifies the bonus
    
    // CONFIRMATION
    ready: bool,                   // Always true (button press)
}
```

**Validation:**
- `request_id` must match pending request
- If `spend_hope_for_bonus == true`:
  - Player must have Hope ≥ 1
  - `chosen_experience` must be in `relevant_experiences`

---

#### 4. Server → All: Roll Result

```typescript
ServerMessage::RollResult {
    // WHO & WHAT
    request_id: String,
    character_id: String,
    character_name: String,
    roll_type: RollType,
    context: String,
    
    // THE ROLL
    hope_die: u8,                  // 1-12
    fear_die: u8,                  // 1-12
    advantage_die: Option<u8>,     // 1-6 if had advantage
    
    // MODIFIERS BREAKDOWN
    attribute_modifier: i8,
    proficiency_modifier: i8,
    situational_modifier: i8,
    hope_bonus: i8,                // +2 if spent Hope
    total_modifier: i8,
    
    // RESULT
    total: u16,                    // Sum of dice + modifiers
    difficulty: u16,
    
    // OUTCOME
    success_type: SuccessType,     // Failure | SuccessWithHope | SuccessWithFear | Critical
    controlling_die: ControllingDie,  // Hope | Fear | Tied
    is_critical: bool,             // Doubles (Hope == Fear)
    
    // NARRATIVE
    outcome_description: String,   // "SUCCESS WITH HOPE" | "CRITICAL SUCCESS" | etc.
    
    // RESOURCE CHANGES
    hope_change: i8,               // +1 (Hope win), -1 (spent), 0 (Fear/crit)
    fear_change: i8,               // +1 (Fear win), 0 (Hope/crit)
    new_hope: u8,                  // Character's Hope after roll
    new_fear: u8,                  // GM's Fear pool after roll
    
    // COMBAT-SPECIFIC
    initiative_shift: Option<InitiativeShift>,  // Only if is_combat
    
    // TIMESTAMP
    timestamp: u64,
}

enum SuccessType {
    Failure,
    SuccessWithHope,
    SuccessWithFear,
    CriticalSuccess,
}

enum ControllingDie {
    Hope,
    Fear,
    Tied,  // Only when doubles
}

enum InitiativeShift {
    PlayersKeep,   // Hope success
    ToEnemies,     // Fear success
    None,          // Critical or failure
}
```

---

#### 5. GM-Only: Roll Request Status

```typescript
ServerMessage::RollRequestStatus {
    request_id: String,
    status: RequestStatus,
    pending_characters: Vec<String>,  // Who hasn't rolled yet
    completed_characters: Vec<String>,  // Who has rolled
}

enum RequestStatus {
    Pending,      // Waiting for players
    Completed,    // All players rolled
    Expired,      // Timed out
    Cancelled,    // GM cancelled
}
```

---

## Difficulty Guidelines

### Standard Difficulties (from Daggerheart SRD)

| Difficulty | Description | Examples |
|------------|-------------|----------|
| **8** | Trivial | Climb a ladder with handholds |
| **10** | Easy | Notice something obvious, lift something moderately heavy |
| **12** | Moderate (Default) | Pick a simple lock, persuade a neutral NPC |
| **14** | Challenging | Leap across a wide gap, recall obscure knowledge |
| **16** | Hard | Balance on a narrow beam while in combat, intimidate a trained guard |
| **18** | Very Hard | Convince a hostile enemy to stand down, perform acrobatics mid-fall |
| **20** | Extremely Hard | Remember a forgotten ancient language, survive a deadly trap |
| **22+** | Nearly Impossible | Reserved for legendary feats, high-level play |

### GM's Rule of Thumb
- **Most rolls should be DC 12-14**
- Use higher/lower for exceptional circumstances
- Consider: "What would happen on a bare success?" → Set difficulty accordingly

### Combat Difficulties
- **Attacks:** Target's **Evasion** (typically 12-16)
- **Spellcasts:** Spell-specific DC (often 15)

---

## Modifiers & Attributes

### Automatic Modifiers (Applied by Server)

1. **Attribute Modifier**
   - Character's relevant attribute: +2, +1, +1, +0, +0, or -1
   - GM specifies which attribute (Agility, Strength, etc.)
   - Server looks up character's value

2. **Proficiency Bonus**
   - **Default: +1** (until leveling system implemented)
   - Applies to:
     - Attacks with proficient weapons
     - Class-specific skills (TBD)
   - Does NOT apply to all checks

3. **Situational Modifiers**
   - GM-granted bonus/penalty: -5 to +5
   - Examples:
     - +2: Favorable conditions (high ground, ally assist)
     - -2: Difficult conditions (darkness, exhaustion)
     - +1: Minor advantage (proper tools)
     - -1: Minor hindrance (distraction)

### When to Apply Proficiency

**✅ Apply proficiency:**
- Attack rolls with trained weapons
- Specific class features (future implementation)

**❌ Don't apply proficiency:**
- General attribute checks (Agility to jump, Knowledge to recall, etc.)
- Rolls outside character's training

**For MVP:** 
- Apply proficiency only to **Attack** rolls
- Action/Save rolls use **attribute only**

---

## Advantage/Disadvantage

### Advantage (Roll +1d6)

**When to grant:**
- Favorable positioning (high ground, flanking)
- Ally assistance (Help action)
- Beneficial conditions (invisible to target, target restrained)
- Magical buffs
- Narrative circumstances (perfect timing, clever plan)

**Mechanics:**
- Roll **2d12 + 1d6**
- Total = Hope + Fear + d6 + modifiers
- d6 **only adds to total**, does NOT affect Hope/Fear determination
- Controlling die still based on 2d12 comparison

**Example:**
```
Hope die: 7
Fear die: 10
d6 (advantage): 4
Modifier: +2
─────────────────
Total: 23
Controlling: FEAR (10 > 7)
→ If total ≥ DC: Success with Fear
```

### Disadvantage

**Daggerheart does NOT have explicit disadvantage rules in available documentation.**

**Options:**
1. **Don't implement** (stay true to SRD)
2. **Use negative modifier** instead (-2 or -3)
3. **Future:** Check for official disadvantage rules

**Recommendation:** Use **situational penalties** (-2 to -5) instead of a separate disadvantage mechanic.

---

## Success Resolution

### Resolution Logic (Server-Side)

```rust
fn resolve_roll(
    hope: u8,
    fear: u8,
    advantage_die: Option<u8>,
    modifier: i8,
    difficulty: u16,
) -> RollOutcome {
    // 1. Calculate total
    let total = hope as u16 + fear as u16 
                + advantage_die.unwrap_or(0) as u16 
                + modifier as u16;
    
    // 2. Check critical (doubles)
    let is_critical = hope == fear;
    
    // 3. Determine controlling die
    let controlling = if hope > fear {
        ControllingDie::Hope
    } else if fear > hope {
        ControllingDie::Fear
    } else {
        ControllingDie::Tied  // Only when doubles
    };
    
    // 4. Determine success type
    let success_type = if total < difficulty {
        SuccessType::Failure
    } else if is_critical {
        SuccessType::CriticalSuccess
    } else if controlling == ControllingDie::Hope {
        SuccessType::SuccessWithHope
    } else {
        SuccessType::SuccessWithFear
    };
    
    // 5. Calculate resource changes
    let (hope_change, fear_change) = match success_type {
        SuccessType::SuccessWithHope => (1, 0),   // Player gains 1 Hope
        SuccessType::SuccessWithFear => (0, 1),   // GM gains 1 Fear
        SuccessType::CriticalSuccess => (0, 0),   // No resource change
        SuccessType::Failure => (0, 0),           // No resource change
    };
    
    RollOutcome {
        total,
        success_type,
        controlling,
        is_critical,
        hope_change,
        fear_change,
    }
}
```

### Narrative Outcomes (GM Use)

| Result | What Happens | Narrative Examples |
|--------|--------------|-------------------|
| **Critical Success** | Extraordinary outcome, PC looks amazing | • "You leap across with supernatural grace, landing perfectly poised"<br>• "The guard is not just convinced—they join your cause"<br>• "You don't just remember the legend, you recall hidden details even scholars don't know" |
| **Success with Hope** | Success + positive momentum | • "You make the jump and land with momentum to keep moving"<br>• "The guard lets you pass and even warns you of patrols ahead"<br>• "You recall the legend and a useful detail comes to mind" |
| **Success with Fear** | Success + complication | • "You make the jump but twist your ankle (1 Stress)"<br>• "The guard lets you pass but alerts their superior"<br>• "You recall the legend but it triggers an unwanted memory" |
| **Failure** | Action doesn't work | • "You misjudge the distance and plummet (take damage)"<br>• "The guard refuses and calls for backup"<br>• "The legend escapes you completely" |

---

## Hope/Fear Tracking

### Starting Values
- **Player Hope:** 5 (standard starting value)
- **GM Fear Pool:** 5 (shared pool for all enemies)

### Maximum Values
- **Hope:** 5 (standard max, can be modified by class/items)
- **Fear:** No documented maximum (GM pool grows as needed)

### Hope Spending

Players can spend Hope for:

1. **+2 Bonus to Roll (via Experience)**
   - Must have relevant Experience
   - Spend 1 Hope BEFORE rolling
   - Add +2 to total

2. **Avoid Death (0 HP)**
   - When reduced to 0 HP, choose "Avoid Death"
   - Survive but permanently lose 1 max Hope

3. **Special Abilities**
   - Some domain cards cost Hope to activate
   - Example: "Spend 1 Hope to reroll this check"

### Fear Spending (GM)

GM can spend Fear for:

1. **Enemy Activations**
   - Spend Fear to activate enemies out of turn
   - Interrupt player actions

2. **Special Enemy Abilities**
   - Boss monsters have Fear-costed abilities

3. **Complications**
   - Add narrative twists

### Tracking in VTT

**Character-level:**
- Each PC has `current_hope` and `max_hope`
- Display in character sheet and GM view

**Game-level:**
- Single `fear_pool` value (GM's shared resource)
- Display in GM panel

**Roll Results:**
- Update Hope/Fear immediately after roll resolution
- Broadcast changes to all clients
- Show +1/-1 indicators in roll result

---

## Experience Spending

### What are Experiences?

Experiences are **player-created narrative hooks** (e.g., "I've been in bar brawls before").

**Mechanics:**
- At character creation, player defines 2-3 Experiences
- When relevant, can **spend 1 Hope for +2 to roll**
- At certain levels, choose 2 Experiences to gain **permanent +1 bonus**

### In the VTT

**Character Creation:**
- Optional field: "Experiences" (text input, comma-separated)
- Example: "Former soldier, Tavern brawler, Raised by druids"

**During Roll Request:**
1. Server checks: Does character have Experiences?
2. Server sends: `relevant_experiences: Vec<String>` (GM can pre-filter or send all)
3. Player sees: "You can spend Hope (+2) if this relates to your Experiences"
4. Player chooses: Select Experience + Spend Hope
5. Server validates: Does character have ≥1 Hope?
6. Apply: +2 to roll, -1 to Hope

**GM View:**
- See each character's Experiences
- When requesting roll, option to "Suggest relevant Experience"

---

## Combat vs Non-Combat

### Non-Combat Rolls

**Characteristics:**
- No time pressure
- Narrative pacing
- No initiative implications
- Hope/Fear still tracked

**Examples:**
- Climbing a cliff
- Persuading an NPC
- Searching for traps
- Recalling knowledge

**UI:**
- Roll request can stay on screen until player responds
- No urgency indicators

---

### Combat Rolls

**Characteristics:**
- Time-sensitive
- Affects initiative flow
- Hope success = players keep initiative
- Fear success = enemies get initiative

**Examples:**
- Attack rolls (vs Evasion)
- Spellcast rolls (vs spell DC)
- Dodge/save rolls

**Initiative Flow:**
```
Player's turn:
1. Player declares action
2. GM calls for roll
3. Player rolls

Result determines next turn:
- Hope Success → Player chooses next PC to act
- Fear Success → GM activates enemy
- Critical → No initiative shift (GM discretion)
- Failure → GM activates enemy (attack failed)
```

**UI Differences:**
- Combat roll requests have **timer** (optional, 30 seconds)
- Initiative tracker updates based on result
- Clear "Next to Act" indicator

**MVP:** Track `is_combat` flag but don't implement full initiative system yet. Just show who should go next.

---

## UI/UX Specification

### GM View (Desktop/TV)

#### Roll Request Panel

```
┌─────────────────────────────────────────┐
│ ⚔️  REQUEST DICE ROLL                   │
├─────────────────────────────────────────┤
│                                         │
│ Target Players:                         │
│  ☑ Theron   ☐ Elena   ☐ Grax   ☐ Mira  │
│  ○ All Players                          │
│                                         │
│ Roll Type: ● Action ○ Attack ○ Spellcast│
│                                         │
│ Attribute: [Agility ▼]                  │
│                                         │
│ Difficulty: [12▼] (Moderate)            │
│  Quick: [Easy:10] [Hard:16] [VHard:18]  │
│                                         │
│ Context:                                │
│  [Leap across the chasm_______________] │
│                                         │
│ Modifiers:                              │
│  Situational: [-2 -1  0 +1 +2 +3]       │
│  ☐ Advantage (+1d6)                     │
│                                         │
│ Combat: ☐ Yes ☑ No                      │
│                                         │
│       [REQUEST ROLL] 🎲                 │
└─────────────────────────────────────────┘
```

#### Pending Rolls Feed

```
⏳ PENDING ROLLS

• Theron - Leap across the chasm (DC 14)
  Waiting... [Cancel]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ RECENT RESULTS (Last 5)

🌟 Theron rolled 16 vs DC 14 - SUCCESS WITH HOPE
   Hope: 8, Fear: 5, Mod: +3 = 16
   Hope: 4 → 5

⚠️ Elena rolled 13 vs DC 15 - SUCCESS WITH FEAR
   Hope: 6, Fear: 7, Mod: +0 = 13
   Fear: 3 → 4

❌ Grax rolled 8 vs DC 12 - FAILURE
   Hope: 2, Fear: 4, Mod: +2 = 8
```

#### Roll Result Breakdown (Expanded View)

```
┌─────────────────────────────────────────┐
│ 🎲 ROLL RESULT: Theron                  │
├─────────────────────────────────────────┤
│                                         │
│ Context: Leap across the chasm          │
│ Difficulty: 14                          │
│                                         │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                         │
│ 🎲 Hope Die: 8                          │
│ 🎲 Fear Die: 5                          │
│ 🎲 Advantage: —                         │
│                                         │
│ ➕ Agility: +2                          │
│ ➕ Proficiency: +0                      │
│ ➕ Situational: +0                      │
│ ➕ Experience: +0                       │
│                                         │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                         │
│ TOTAL: 16                               │
│                                         │
│ 🌟 SUCCESS WITH HOPE                    │
│                                         │
│ Controlling Die: HOPE (8 > 5)           │
│ Hope: 4 → 5 (+1)                        │
│                                         │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                         │
│ Theron lands gracefully, momentum with  │
│ them! The party keeps initiative.       │
│                                         │
└─────────────────────────────────────────┘
```

---

### Mobile View (Player)

#### Roll Request Modal (Blocking)

```
┌─────────────────────────────────────┐
│ 🎲 GM REQUESTS ROLL                 │
├─────────────────────────────────────┤
│                                     │
│  "Leap across the chasm"            │
│                                     │
│  Difficulty: 14 (Moderate)          │
│                                     │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                     │
│  You'll roll: 2d12 + modifiers      │
│                                     │
│  Agility: +2                        │
│  Proficiency: +0                    │
│  Situational: +0                    │
│  ────────────                       │
│  Total Modifier: +2                 │
│                                     │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                     │
│  💡 Spend Hope for +2?              │
│                                     │
│  ☐ Use Experience: "Former acrobat" │
│     (Hope: 5 → 4, Roll: +2)         │
│                                     │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                     │
│         [ROLL DICE] 🎲              │
│                                     │
└─────────────────────────────────────┘
```

#### Roll Result Display (Mobile)

```
┌─────────────────────────────────────┐
│ 🌟 SUCCESS WITH HOPE                │
├─────────────────────────────────────┤
│                                     │
│  You rolled: 16 vs DC 14            │
│                                     │
│  Hope: 8  Fear: 5  Mod: +2          │
│                                     │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                     │
│  You leap across with grace and     │
│  momentum on your side!             │
│                                     │
│  Hope: 4 → 5 (+1) 🌟                │
│                                     │
└─────────────────────────────────────┘
```

---

## Edge Cases

### 1. Multiple Players Rolling Same Check

**Scenario:** GM requests "Everyone roll Instinct to notice the trap"

**Handling:**
- Send same `request_id` to all players
- Each player rolls independently
- Each result broadcast separately
- GM sees all results in feed
- Each player's Hope/Fear tracked separately

**UI:**
- GM sees: "3/4 players rolled" progress indicator
- GM can cancel pending requests

---

### 2. Player Disconnects During Roll Request

**Handling:**
- Request remains pending
- When player reconnects, show pending request
- If timeout set, request expires after X seconds
- GM can manually cancel

**UI:**
- GM sees: "Theron (disconnected)" in pending list
- Option: [Cancel Request] or [Wait]

---

### 3. Player Tries to Spend Hope But Has 0

**Handling:**
- Server validates Hope ≥ 1 before allowing spend
- If Hope = 0, disable spend option

**UI:**
- Checkbox grayed out: "☐ Use Experience (no Hope left)"

---

### 4. Critical Success on Low Total

**Scenario:** Player rolls 3+3 = 6, but DC is 12

**Ruling:** **Still a critical success** (RAW in Daggerheart)

**Handling:**
- Server checks: Is doubles? → Yes → Critical
- Even if `total < difficulty` → Critical Success
- Narrative: GM decides what "critical" means in this context

**Example:**
```
"You leap across but misjudge badly. However, 
your training kicks in — you grab a handhold 
mid-fall and pull yourself up to safety!"
```

---

### 5. Advantage Die Causing Exact DC Match

**Scenario:** 2d12 = 10, advantage d6 = 4, modifier = 0 → Total = 14 vs DC 14

**Ruling:** **Success** (total ≥ difficulty)

**Controlling Die:** Still determined by 2d12 comparison (d6 doesn't affect Hope/Fear)

---

### 6. Negative Modifier Drops Total Below 0

**Scenario:** Roll 2+3 = 5, modifier -6 → Total = -1?

**Handling:**
- Total cannot go below 0
- `total = max(0, hope + fear + modifiers)`

---

### 7. GM Requests Roll for NPC

**Use Case:** GM wants to roll for enemy or ally NPC

**Handling:**
- `target_type: RollTargetType::NPC`
- Server executes roll immediately (no player input)
- Broadcast result to GM only (optional: to all)

**UI:**
- GM has "Roll for NPC" button
- Select NPC from list
- Same interface, auto-executes

---

## Implementation Checklist

### Phase 1: Protocol & Server Logic

- [ ] Define new protocol messages (`RequestRoll`, `RollRequested`, `ExecuteRoll`, `RollResult`)
- [ ] Add `pending_roll_requests` map to `GameState`
- [ ] Implement roll request handler (GM → Server)
- [ ] Implement roll execution handler (Player → Server)
- [ ] Implement roll resolution logic (calculate success type, Hope/Fear changes)
- [ ] Broadcast roll results to all clients
- [ ] Track Hope/Fear in character state
- [ ] Validate all inputs (difficulty range, Hope spending, etc.)

### Phase 2: GM UI (Desktop)

- [ ] Create "Request Roll" panel
- [ ] Attribute dropdown (Agility, Strength, Finesse, Instinct, Presence, Knowledge)
- [ ] Difficulty slider/input (8-22) with quick presets
- [ ] Context text input (max 200 chars)
- [ ] Modifier buttons (-2, -1, 0, +1, +2)
- [ ] Advantage checkbox
- [ ] Combat mode toggle
- [ ] "Request Roll" button
- [ ] Pending rolls feed (who hasn't rolled yet)
- [ ] Recent results feed (last 5 rolls)
- [ ] Expandable roll result breakdown
- [ ] Hope/Fear tracker display

### Phase 3: Mobile UI (Player)

- [ ] Roll request modal (blocking, can't dismiss)
- [ ] Display context, difficulty, modifiers
- [ ] Show calculated total modifier
- [ ] Experience spending checkbox (if applicable)
- [ ] "Roll Dice" button
- [ ] Roll result display with animation
- [ ] Hope/Fear update indicator
- [ ] Success type visual (color-coded: green=Hope, orange=Fear, gold=Crit, red=Fail)

### Phase 4: Testing

- [ ] Unit tests: Roll resolution logic
- [ ] Unit tests: Hope/Fear tracking
- [ ] Unit tests: Experience spending validation
- [ ] Integration test: Full roll request flow
- [ ] Manual test: Multi-player rolls
- [ ] Manual test: Combat vs non-combat
- [ ] Manual test: Edge cases (disconnect, negative modifiers, etc.)

### Phase 5: Polish

- [ ] Animations for dice rolls
- [ ] Sound effects (optional)
- [ ] Difficulty guideline tooltips in GM UI
- [ ] Roll history/log (save past rolls)
- [ ] Export roll log to JSON
- [ ] Keyboard shortcuts (GM: Ctrl+R to request roll)

---

## Success Criteria

### MVP is Complete When:

1. ✅ GM can request rolls with custom difficulty, attribute, and modifiers
2. ✅ Players receive roll request as blocking modal
3. ✅ Players can spend Hope for +2 (via Experience)
4. ✅ Server executes 2d12 roll with correct modifiers
5. ✅ Success type determined correctly (Hope/Fear/Critical/Failure)
6. ✅ Hope/Fear resources update automatically
7. ✅ All clients see roll results in real-time
8. ✅ Doubles = Critical Success (even if total < DC)
9. ✅ GM sees pending and completed rolls
10. ✅ Roll results show full breakdown (dice, modifiers, outcome)

---

## Assumptions & Future Work

### Assumptions (MVP)

- Proficiency = +1 (flat, no progression)
- No multi-classing (one class per character)
- Experiences are freeform text (GM validates relevance)
- No automated initiative system (just track is_combat flag)
- No spell/attack damage resolution (just roll vs DC)
- Fear pool is global (shared by GM for all enemies)

### Future Enhancements

- [ ] Automated initiative tracker (Hope → Player, Fear → Enemy)
- [ ] Spell/attack damage rolls (after successful roll)
- [ ] Condition tracking (Restrained, Stunned, etc.)
- [ ] Experience leveling (+1 permanent bonus at certain levels)
- [ ] Proficiency progression (increase with level)
- [ ] Custom attributes (for homebrewed traits)
- [ ] Roll history search/filter
- [ ] Dice roll animations (3D dice, particle effects)
- [ ] Voice feedback ("Success with Hope!")

---

**End of Specification**

**Status:** Ready for Implementation  
**Review:** Please confirm this spec matches Daggerheart rules before coding begins.  
**Questions?** Ask for clarification on any section.

---

**Prepared by:** OpenClaw Agent  
**Date:** 2026-02-23  
**Version:** 1.0  
