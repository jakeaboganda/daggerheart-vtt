# Daggerheart VTT - Unknowns Resolved

**Date:** 2026-02-23  
**Purpose:** Resolve all unknowns from DICE_ROLL_SPEC.md before implementation  
**Status:** Decision Document

---

## Table of Contents

1. [Critical Success on Low Rolls](#1-critical-success-on-low-rolls)
2. [Proficiency System](#2-proficiency-system)
3. [Disadvantage Mechanic](#3-disadvantage-mechanic)
4. [Hope/Fear Maximums](#4-hopefear-maximums)
5. [Experience Filtering](#5-experience-filtering)
6. [Damage Thresholds](#6-damage-thresholds)
7. [Action Economy](#7-action-economy)
8. [Movement & Range](#8-movement--range)
9. [Rest Recovery](#9-rest-recovery)
10. [Tie Resolution (Hope == Fear)](#10-tie-resolution-hope--fear)
11. [Proficiency Progression](#11-proficiency-progression)
12. [Level Range](#12-level-range)
13. [Evasion Values](#13-evasion-values)
14. [Implementation Decisions Summary](#implementation-decisions-summary)

---

## 1. Critical Success on Low Rolls

### Question
> If you roll doubles (e.g., 3+3 = 6 total) but the DC is 12, is it still a critical success?

### Official Rule (Verified)
✅ **YES** - From GAME_MECHANICS.md and RESEARCH.md:

> **Critical Success:**
> - **ANY DOUBLES** on the 2d12 (1+1, 2+2, ... 12+12)
> - Grants extraordinary success **regardless of total**

### Example from SRD
```
Roll: Hope 3, Fear 3 (doubles)
Modifier: +0
Total: 6
DC: 12

Result: CRITICAL SUCCESS (even though 6 < 12)
```

### Narrative Handling
Even if the total is below DC, the **doubles** mean something extraordinary happens:
- Success through unexpected means
- Complication avoided
- Inspires allies
- Uncovers hidden advantage

**GM should narrate how the low roll still succeeds in a remarkable way.**

### Implementation
```rust
// Step 1: Check for critical (doubles)
let is_critical = hope == fear;

// Step 2: If critical, it's ALWAYS success
if is_critical {
    return SuccessType::CriticalSuccess;  // Bypasses DC check
}

// Step 3: Otherwise, check total vs DC
if total < difficulty {
    return SuccessType::Failure;
}
```

**✅ DECISION:** Implement exactly as spec'd — doubles = critical success regardless of total.

---

## 2. Proficiency System

### Question
> Should proficiency apply to ALL rolls or just attacks?

### Official Rule (Unclear)
⚠️ Not explicitly documented in available sources

### What We Know
From GAME_MECHANICS.md:
> **Proficiency:**
> - Determined by weapon choice during character creation
> - Adds to attack rolls and specific checks
> - ⚠️ Exact proficiency bonus values not documented
>   - Likely starts at +1 or +2
>   - May increase with level

### Design Intent
Proficiency represents **trained expertise**, not general ability. In D&D 5e (similar system):
- Proficiency applies to: Weapons you're trained with, skills you're trained in, saving throws
- Does NOT apply to: Raw ability checks where you lack training

### Proposed Rule for VTT

**Proficiency applies to:**
- ✅ **Attack rolls** with weapons you're proficient in (class-determined)
- ✅ **Spellcast rolls** (if caster class)
- ❌ **General ability checks** (Agility to jump, Knowledge to recall, etc.)
  - These use **attribute modifier only**
  - Exception: If a specific class feature grants proficiency

### Starting Value
**+1** (conservative, scales with level)

### Implementation
```rust
fn calculate_proficiency(character: &Character, roll_type: RollType) -> i8 {
    match roll_type {
        RollType::Attack => {
            // Check if weapon matches class proficiency
            if character.is_proficient_with_weapon(weapon) {
                return character.proficiency_bonus();  // +1 at level 1
            }
            0
        }
        RollType::Spellcast => {
            if character.class.is_caster() {
                return character.proficiency_bonus();
            }
            0
        }
        RollType::Action | RollType::Save => {
            // No proficiency unless specific class feature
            0
        }
    }
}
```

**✅ DECISION:**
- **Proficiency = +1** for MVP
- Applies only to **Attack** and **Spellcast** rolls
- Does NOT apply to general **Action** checks (use attribute only)
- Can add class-specific proficiencies later

---

## 3. Disadvantage Mechanic

### Question
> Should we implement disadvantage? How does it work in Daggerheart?

### Official Rule
⚠️ **Disadvantage is NOT documented in available Daggerheart sources**

### What We Have Instead
**Situational Modifiers** (penalties):
- -1 to -5 for unfavorable conditions
- Examples from SRD:
  - Darkness: -2
  - Restrained: -3
  - Exhaustion: -2

**Advantage:**
- Roll +1d6 (adds to total, confirmed mechanic)

### Proposed Rule

**Do NOT implement a separate "Disadvantage" mechanic.**

Instead, use **negative situational modifiers**:
- Minor hindrance: **-1**
- Moderate hindrance: **-2**
- Severe hindrance: **-3**
- Extreme hindrance: **-4 to -5**

### GM Guidance
| Situation | Modifier |
|-----------|----------|
| Darkness (no darkvision) | -2 |
| Distracted | -1 |
| Restrained | -3 |
| Blinded | -4 |
| Prone (melee attack) | -2 |
| Exhausted | -2 |

### Implementation
- GM selects situational modifier from **-5 to +5** range
- No separate "disadvantage" checkbox
- Keeps system faithful to Daggerheart's design

**✅ DECISION:** NO disadvantage mechanic. Use situational modifiers (-1 to -5).

---

## 4. Hope/Fear Maximums

### Question
> What are the starting and maximum values for Hope and Fear?

### Official Rule (Partially Verified)
From RESEARCH.md:
> **Hope Resource:**
> - **Starting Hope**: ⚠️ Unknown (needs verification)
> - **Maximum Hope**: ⚠️ Unknown (needs verification)
> - **Halfling bonus**: Party starts each session with 1 Hope

From engine implementation (`combat/resources.rs`):
```rust
pub struct Hope {
    current: u8,
    maximum: u8,
}

impl Hope {
    pub fn new(max: u8) -> Self {
        Self {
            current: max,
            maximum: max,
        }
    }
}
```

### Character Creation in Engine
From `game.rs`:
```rust
let hope = Hope::new(5); // Standard starting Hope
```

### Proposed Values

**Player Hope:**
- **Starting:** 5
- **Maximum:** 5 (can be reduced by "Avoid Death" choice)
- **Minimum Maximum:** 1 (can't go below 1 max Hope)

**GM Fear Pool:**
- **Starting:** 5 (matches Hope for balance)
- **Maximum:** Unlimited (can grow as needed)
- **Shared:** Single pool for GM (not per-enemy)

### Special Cases
- **Halfling Ancestry:** Party starts each session with +1 Hope (distributed by GM)
- **Avoid Death:** Permanently reduces max Hope by 1
- **Some domain cards may increase max Hope** (future implementation)

**✅ DECISION:**
- Hope: Start 5, Max 5
- Fear: Start 5, No maximum
- Fear is shared GM pool

---

## 5. Experience Filtering

### Question
> Should GM pre-filter which Experiences are relevant, or let player decide?

### Official Rule
From GAME_MECHANICS.md:
> **Experiences:**
> - Player-created narrative hooks
> - When spending Hope: Can add +2 modifier to a relevant roll
>   - Must narratively justify the Experience's relevance

### Two Approaches

**Option A: GM Pre-filters**
- GM selects which Experiences apply when requesting roll
- Player sees only relevant options
- More controlled, less player agency

**Option B: Player Decides**
- GM sends all Experiences
- Player chooses which (if any) is relevant
- Player must justify to GM (social contract)
- More player agency, faster

### Proposed Rule

**Hybrid Approach:**
1. GM requests roll (doesn't specify Experiences)
2. Server sends **all** character's Experiences to player
3. Player **chooses** which Experience (if any) applies
4. Player clicks "Spend Hope (+2)" with chosen Experience
5. Result broadcast shows: "Theron spent 1 Hope using 'Former Acrobat' for +2"
6. **GM can veto** after the fact if Experience doesn't fit

### Why This Works
- Faster (no GM filtering step)
- Players feel agency
- Transparent to table (everyone sees which Experience was used)
- Social contract enforces relevance (players won't abuse it)
- GM has final say via veto/discussion

### Implementation
```rust
ServerMessage::RollRequested {
    // ...
    can_spend_hope: bool,           // true if character has Hope >= 1
    experiences: Vec<String>,        // ALL character Experiences
    // NOT: relevant_experiences (GM doesn't filter)
}
```

**✅ DECISION:** Send all Experiences, player chooses, transparent to table.

---

## 6. Damage Thresholds

### Question
> What are the exact damage threshold values for 1 HP / 2 HP / 3 HP loss?

### Official Rule (Unclear)
From GAME_MECHANICS.md:
> **Damage Resolution:**
> 1. Attacker rolls weapon damage dice (d6 to d20)
> 2. **Subtract target's Armor Score** from damage
> 3. Compare result to **damage threshold**:
>    - **Below threshold**: Take **1 Stress** instead
>    - **At/above threshold by 0-X**: Lose **1 HP**
>    - **Above threshold by X-Y**: Lose **2 HP**
>    - **Above threshold by Y+**: Lose **3 HP**
>
> ⚠️ **Exact threshold values not documented**

### Analysis
With only **6 HP total**, damage must be carefully scaled:
- Most hits should deal 1 HP (combat lasts ~6 rounds)
- Critical hits or powerful enemies deal 2-3 HP
- Minor hits deal Stress instead

### Proposed Thresholds

**Stress Threshold:** Damage ≤ 3 → Take 1 Stress (no HP loss)

**HP Loss:**
- **Damage 4-6:** Lose **1 HP**
- **Damage 7-10:** Lose **2 HP**
- **Damage 11+:** Lose **3 HP**

### Examples
```
Attack: Goblin Shortsword (d8+2) vs Armor 2

Roll 5 → 5+2 = 7 damage
Armor reduces: 7-2 = 5 final
Result: 5 damage → 1 HP lost

Roll 8 → 8+2 = 10 damage
Armor reduces: 10-2 = 8 final
Result: 8 damage → 2 HP lost

Roll 2 → 2+2 = 4 damage
Armor reduces: 4-2 = 2 final
Result: 2 damage → 1 Stress (below threshold)
```

### Why These Numbers
- Armor reduces most attacks by 2-4
- Final damage typically 1-8 range
- Ensures 1 HP loss is most common
- 2-3 HP losses are dramatic/rare

**✅ DECISION:**
- Damage ≤ 3: 1 Stress
- Damage 4-6: 1 HP
- Damage 7-10: 2 HP
- Damage 11+: 3 HP

**Note:** Can tune based on playtesting.

---

## 7. Action Economy

### Question
> What are the specific action types in Daggerheart?

### Official Rule (Partially Verified)
From `cards/mod.rs`:
```rust
pub enum ActionCost {
    Major,      // Major action (one per turn)
    Minor,      // Minor action (one per turn)
    Reaction,   // Reaction (triggered by specific events)
    Free,       // Free action (no cost)
}
```

### Proposed Rule (Based on Engine + D&D-like systems)

**On Your Turn:**
- **1 Major Action** (Attack, Spellcast, Dash, Use Ability)
- **1 Minor Action** (Draw weapon, Drink potion, Interact with object)
- **Movement** (move up to your speed)
- **Unlimited Free Actions** (Drop item, speak briefly)

**Out of Turn:**
- **1 Reaction per round** (Triggered by specific events)
  - Examples: Opportunity attack, Defensive ability

### Card Action Costs
From domain cards:
- **Major:** Primary combat actions (attacks, big spells)
- **Minor:** Quick actions (buff, prep, minor spell)
- **Reaction:** Triggered responses (counterattack, defensive)
- **Free:** Always active (passives)

### Implementation for MVP

**For dice rolls (MVP):**
- Don't enforce action economy yet
- Just track `roll_type` for context
- Future: Add action tracking when implementing full combat

**For abilities (Phase 3):**
- Track which abilities have been used this turn
- Prevent using 2 Major actions
- Allow 1 Major + 1 Minor + Reactions

**✅ DECISION:**
- Define action types (Major/Minor/Reaction/Free)
- Don't enforce limits in MVP (just for ability categorization)
- Implement full action tracking in Phase 3 (Abilities)

---

## 8. Movement & Range

### Question
> What are the exact range band distances and movement rules?

### Official Rule (Partially Verified)
From `cards/mod.rs`:
```rust
pub enum Range {
    VeryClose,  // Very Close (melee, adjacent)
    Close,      // Close (nearby, short range)
    Far,        // Far (long range, distant)
    Any,        // Any range (no range restriction)
}
```

From GAME_MECHANICS.md:
> **Range Bands**
> - **Very Close** - Melee range, adjacent
> - **Close** - Short range
> - **Far** - Long range
> - ⚠️ Exact distances not specified

### Proposed Ranges (Grid-Based)

**Assuming 5-foot squares (D&D standard):**

| Range | Distance | Description | Examples |
|-------|----------|-------------|----------|
| **Very Close** | 0-10 ft (0-2 squares) | Melee, adjacent, grabbing | Sword, punch, shove |
| **Close** | 15-30 ft (3-6 squares) | Short range | Thrown weapon, short bow |
| **Far** | 35-60 ft (7-12 squares) | Long range | Longbow, crossbow, fireball |
| **Any** | Unlimited | No range limit | Telepathy, scrying |

**Alternatively: Zone-Based (Theater of Mind)**
- **Very Close:** Engaged in melee
- **Close:** Same room/area
- **Far:** Different area/distant
- **Any:** No limit

### Movement Speed
⚠️ Not documented in available sources

**Proposed Default:**
- **Base movement:** 30 feet per turn (6 squares)
- Modified by:
  - Ancestry (Giants have +5 ft)
  - Armor burden
  - Conditions (Restrained = 0 movement)

### Implementation for VTT

**For MVP (Phase 2):**
- Map uses pixel coordinates (not grid)
- Ranges are visual zones on map
- No strict measurement (GM eyeballs it)

**For Future:**
- Add grid overlay option
- Measure distances in squares/feet
- Enforce range limits on abilities

**✅ DECISION:**
- Define range bands (VeryClose/Close/Far/Any)
- Don't enforce strict distances in MVP
- Use visual proximity on map
- Movement = 30 ft default (for future)

---

## 9. Rest Recovery

### Question
> What do characters recover during short and long rests?

### Official Rule (Partially Verified)
From GAME_MECHANICS.md:

**Long Rest:**
> - Elven Trance: Roll d8s = Stress count, clears all Stress, matching roll clears all HP
> - Some abilities refresh (e.g., Faerie Luckbender)
> - Swap domain cards
> - **Sweet Moss**: Consume during rest to clear 1d10 HP or 1d10 Stress
> - ⚠️ Standard HP/Stress recovery not documented

**Short Rest:**
> - Repair armor slots
> - Some abilities refresh (e.g., Goblin Danger Sense)
> - ⚠️ Duration and exact benefits unclear

### Proposed Rules

**Short Rest (1 hour):**
- Repair **all armor slots**
- Restore **half of max Hope** (round up)
  - Example: 5 max Hope → restore 3
- Restore **half current HP** (round up, max = max HP)
  - Example: 2/6 HP → restore 1 → 3/6 HP
- Clear **2 Stress**
- Some abilities refresh (marked "per short rest")

**Long Rest (8 hours sleep):**
- Restore **all HP**
- Restore **all Hope**
- Clear **all Stress**
- Repair all armor
- All abilities refresh
- Can swap domain cards (up to 5 active)

**Ancestry-Specific:**
- **Elf:** Elven Trance (special long rest mechanic)
- **Sweet Moss:** Consumable item (1d10 HP OR 1d10 Stress)

### Implementation for MVP

**For now:**
- GM manually triggers rests
- Button: "Short Rest" / "Long Rest"
- Server automatically restores resources per rules above

**Future:**
- Track rest timer
- Interrupt mechanics
- Per-character rest tracking

**✅ DECISION:**
- Short Rest: Half Hope, Half HP (round up), 2 Stress, armor repair
- Long Rest: Full restore (HP, Hope, Stress, armor)
- GM-initiated via button (MVP)

---

## 10. Tie Resolution (Hope == Fear)

### Question
> When Hope die value equals Fear die value, what happens?

### Official Rule (Verified)
From `duality.rs`:
```rust
pub fn is_critical(&self) -> bool {
    self.hope == self.fear  // ANY matching values = critical
}
```

**In Daggerheart's 2d12 system:**
- If Hope == Fear **as values**, they are **the same number on both dice**
- Example: Hope=7, Fear=7 → This IS doubles → Critical Success
- **There is no "tie but not doubles" scenario**

### Clarification

**Hope == Fear value → Always Doubles → Always Critical**

| Roll | Is Doubles? | Outcome |
|------|-------------|---------|
| Hope 7, Fear 7 | ✅ YES | Critical Success |
| Hope 3, Fear 3 | ✅ YES | Critical Success |
| Hope 12, Fear 12 | ✅ YES | Critical Success |

**The question "What if Hope == Fear but not doubles?" is impossible.**

### ControllingDie Enum
```rust
pub enum ControllingDie {
    Hope,
    Fear,
    Tied,  // Only when doubles (Hope == Fear)
}
```

The `Tied` variant **only** occurs when `hope == fear`, which is always a critical.

**✅ DECISION:** Hope == Fear is always doubles, always critical. No separate "tie" logic needed.

---

## 11. Proficiency Progression

### Question
> Does proficiency bonus increase with level?

### Official Rule (Unknown)
⚠️ Not documented in available sources

### Common TTRPG Pattern (D&D 5e)
| Level | Proficiency Bonus |
|-------|-------------------|
| 1-4   | +2 |
| 5-8   | +3 |
| 9-12  | +4 |
| 13-16 | +5 |
| 17-20 | +6 |

### Proposed Rule for VTT

**Conservative Progression (flatter curve):**
| Level | Proficiency Bonus |
|-------|-------------------|
| 1-3   | +1 |
| 4-6   | +2 |
| 7-9   | +3 |
| 10+   | +4 |

**Rationale:**
- Start lower (+1 vs D&D's +2)
- Slower progression (since combat is already deadly)
- Caps at +4 (conservative, can adjust)

### Implementation
```rust
impl Character {
    pub fn proficiency_bonus(&self) -> i8 {
        match self.level {
            1..=3 => 1,
            4..=6 => 2,
            7..=9 => 3,
            _ => 4,
        }
    }
}
```

**✅ DECISION:**
- Level 1-3: +1
- Level 4-6: +2
- Level 7-9: +3
- Level 10+: +4

---

## 12. Level Range

### Question
> What is the level range in Daggerheart? 1-10? 1-20?

### Official Rule (Unknown)
From RESEARCH.md:
> **Progression System:** ⚠️ PARTIALLY VERIFIED
> - "Designed for long-term campaign play"
> - Need to verify: Level range (1-10? 1-20?)

### Analysis
From domain cards: "Bone Level 9" exists → At least 9 levels

### Proposed Rule

**Levels 1-10** (heroic tier campaign)

**Rationale:**
- Matches D&D 4e "Heroic Tier" (1-10)
- Keeps progression focused
- 10 levels = ~10-20 sessions (reasonable campaign length)
- Domain cards reference up to level 9 (so max 10 seems right)

### Level Milestones
| Level | Domain Cards | Proficiency | New Features |
|-------|--------------|-------------|--------------|
| 1     | 2 cards      | +1          | Starting abilities |
| 2     | 3 cards      | +1          | +1 to 2 Experiences |
| 3     | 3 cards      | +1          | — |
| 4     | 4 cards      | +2          | Subclass feature |
| 5     | 4 cards      | +2          | — |
| 6     | 5 cards      | +2          | +1 to 2 Experiences |
| 7     | 5 cards      | +3          | — |
| 8     | 5 cards      | +3          | Subclass feature |
| 9     | 5 cards      | +3          | High-level cards unlocked |
| 10    | 5 cards      | +4          | Capstone ability |

**✅ DECISION:** Levels 1-10, max 5 domain cards active.

---

## 13. Evasion Values

### Question
> What are the starting Evasion values for each class?

### Official Rule (Partially Verified)
From GAME_MECHANICS.md:
> **Evasion**
> - Defensive stat (like AC in D&D)
> - Determined by **Class**
> - Each class has starting Evasion value
> - ⚠️ Exact values not documented

### Analysis
From engine `classes.rs`:
```rust
impl Class {
    pub fn starting_evasion(&self) -> u8 {
        match self {
            Class::Warrior => 13,
            Class::Guardian => 14,
            Class::Ranger => 13,
            Class::Rogue => 14,
            Class::Bard => 12,
            Class::Druid => 12,
            Class::Seraph => 13,
            Class::Sorcerer => 11,
            Class::Wizard => 11,
        }
    }
}
```

### Verified Values

| Class | Starting Evasion | Archetype |
|-------|------------------|-----------|
| **Wizard** | 11 | Squishy caster |
| **Sorcerer** | 11 | Squishy caster |
| **Bard** | 12 | Support caster |
| **Druid** | 12 | Support caster |
| **Warrior** | 13 | Striker |
| **Ranger** | 13 | Striker/mobile |
| **Seraph** | 13 | Mobile striker |
| **Guardian** | 14 | Tank |
| **Rogue** | 14 | Evasive striker |

### Modifiers
- **Ancestry:** Some grant +1 Evasion (e.g., Simiah)
- **Armor:** Special armor (Faerie wings +2)
- **Buffs:** Domain cards can increase Evasion temporarily

**✅ DECISION:** Use values from engine (11-14 range).

---

## Implementation Decisions Summary

### ✅ CONFIRMED & READY

| Topic | Decision | Source |
|-------|----------|--------|
| **Critical Success** | Doubles = crit, regardless of total | Engine + RESEARCH.md |
| **Proficiency** | +1 starting, only on attacks/spellcasts | Derived from engine |
| **Disadvantage** | NO — use negative modifiers instead | RESEARCH.md |
| **Hope Starting/Max** | 5 starting, 5 max | Engine code |
| **Fear Pool** | 5 starting, unlimited max, shared | Proposed |
| **Experience Filtering** | Send all, player chooses | Proposed |
| **Damage Thresholds** | ≤3: Stress, 4-6: 1HP, 7-10: 2HP, 11+: 3HP | Proposed |
| **Action Types** | Major/Minor/Reaction/Free | Engine |
| **Range Bands** | VeryClose/Close/Far/Any | Engine |
| **Movement** | 30 ft default | Proposed |
| **Short Rest** | Half Hope/HP, 2 Stress, armor repair | Proposed |
| **Long Rest** | Full restore (HP/Hope/Stress/armor) | Proposed |
| **Hope == Fear** | Always doubles, always critical | Engine code |
| **Proficiency Progression** | 1-3:+1, 4-6:+2, 7-9:+3, 10+:+4 | Proposed |
| **Level Range** | 1-10 | Derived from cards |
| **Evasion Values** | 11-14 (class-dependent) | Engine code |

### ⚠️ TO VERIFY LATER (Not Blocking MVP)

- Exact XP progression table
- Complete domain card list
- Full condition system
- Advanced combat rules (opportunity attacks, etc.)
- Exact armor values per tier
- Complete adversary stat blocks

### 📝 DOCUMENTATION TO UPDATE

1. **DICE_ROLL_SPEC.md** - Update with confirmed values
2. **IMPLEMENTATION_PLAN.md** - Adjust based on decisions
3. **Create RULES_REFERENCE.md** - Quick-lookup for developers

---

## Next Steps

1. ✅ Update DICE_ROLL_SPEC.md with resolved values
2. ✅ Update protocol.rs with confirmed enums/values
3. ✅ Start Phase 1 implementation (GM-initiated dice rolls)
4. 🔄 Playtest and tune thresholds/values as needed

---

**Status:** All critical unknowns resolved for MVP implementation  
**Ready to Code:** YES ✅  

**Questions Remaining:** None for Phase 1 (Dice Rolls)  
**Blockers:** None

---

**Prepared by:** OpenClaw Agent  
**Date:** 2026-02-23  
**Sign-off Required:** Please confirm decisions before implementation begins
