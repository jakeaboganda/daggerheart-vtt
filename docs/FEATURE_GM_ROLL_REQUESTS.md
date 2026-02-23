# GM Roll Request System - Feature Complete! ✅

**Date:** 2026-02-24  
**Feature:** GM-initiated dice roll requests  
**Status:** ✅ Fully Implemented (Backend + Frontend)

---

## 🎯 What It Does

The GM can now request rolls from players, and the system handles the entire flow:

1. **GM requests a roll** (e.g., "Everyone roll Agility DC 12 to leap across the chasm")
2. **Players receive the request** on their phones with all details
3. **Players execute their rolls** with their modifiers automatically calculated
4. **Results are broadcast** to TV view and GM with success/failure
5. **Status tracking** shows who has/hasn't rolled yet

---

## 🎮 How to Use (Testing Guide)

### Setup
1. **Open 3 browser windows:**
   - Window 1: http://192.168.1.119:3000/gm (GM View)
   - Window 2: http://192.168.1.119:3000/mobile (Player 1)
   - Window 3: http://192.168.1.119:3000/mobile (Player 2) - incognito/private

2. **Create characters** in both mobile windows

---

### Test Flow

#### Step 1: GM Requests a Roll
**In GM View (Window 1):**
1. Find the "🎲 Request Roll" panel in left sidebar
2. Fill in:
   - **Target:** "All Players"
   - **Attribute:** "Agility"
   - **Difficulty:** 12
   - **Context:** "Leap across the chasm"
   - **Advantage:** (unchecked)
3. Click **"🎲 Request Roll"**
4. **Expected:** Status panel appears showing "Pending: Player1, Player2"

#### Step 2: Players See the Request
**In Mobile Views (Windows 2 & 3):**
1. **Expected:** Roll request panel appears (gradient Hope/Fear background)
2. **Displayed info:**
   - Context: "Leap across the chasm"
   - Attribute: "Agility"
   - Difficulty: 12
   - Your Modifier: +X (auto-calculated based on their Agility)
3. **Optional:** Check "Spend Hope for +2" if they have Experiences
4. Click **"🎲 Roll Now!"**

#### Step 3: Results Display
**On TV View (http://192.168.1.119:3000):**
1. **Expected:** Roll overlay appears for each player showing:
   - Player name
   - Hope die (1-12)
   - Fear die (1-12)
   - Total (Hope or Fear + modifier)
   - Success/Failure based on DC 12
   - Controlling die (Hope/Fear/Tied)

**In GM View (Window 1):**
1. **Expected:** Status panel updates as each player rolls
2. **Completed list** fills in with player names
3. **Pending list** shrinks
4. **When all done:** Panel auto-hides after 3 seconds

---

## 📊 Technical Details

### Backend (Already Implemented in Phase 5A)

**Protocol Messages:**
```rust
// GM → Server
ClientMessage::RequestRoll {
    target_type: RollTargetType,      // "all" or "specific"
    target_character_ids: Vec<String>,
    roll_type: RollType,              // "action", "attack", "spellcast", "save"
    attribute: Option<String>,         // "agility", "strength", etc.
    difficulty: u16,                   // DC value
    context: String,                   // "Leap across the chasm"
    narrative_stakes: Option<String>,  // Optional story stakes
    situational_modifier: i8,          // +/- modifier
    has_advantage: bool,
    is_combat: bool,
}

// Server → Player
ServerMessage::RollRequested {
    request_id: String,
    roll_type: RollType,
    attribute: Option<String>,
    difficulty: u16,
    context: String,
    base_modifier: i8,                // Calculated: attribute + proficiency
    situational_modifier: i8,
    total_modifier: i8,
    has_advantage: bool,
    your_attribute_value: i8,
    your_proficiency: i8,
    can_spend_hope: bool,            // True if player has Experiences
    experiences: Vec<String>,
}

// Player → Server
ClientMessage::ExecuteRoll {
    request_id: String,
    spend_hope_for_bonus: bool,      // If checked, costs 1 Hope, adds +2
    chosen_experience: Option<String>,
}

// Server → All
ServerMessage::DetailedRollResult {
    request_id: String,
    character_id: String,
    character_name: String,
    roll_type: RollType,
    context: String,
    roll_details: DetailedRollResult,  // Hope/Fear dice, modifiers, total
    outcome_description: String,       // "SUCCESS WITH HOPE", "FAILURE", etc.
    new_hope: u8,
    new_fear: u8,
}

// Server → GM
ServerMessage::RollRequestStatus {
    request_id: String,
    pending_characters: Vec<String>,
    completed_characters: Vec<String>,
}
```

**Game Logic:**
- Calculates base modifier: `attribute_value + proficiency_bonus`
- Adds situational modifier
- Adds +2 if Hope spent (costs 1 Hope)
- Rolls 2d12 (Hope & Fear)
- Determines controlling die (highest)
- Checks if total >= difficulty
- Updates Hope/Fear pools based on success type

---

### Frontend (Just Implemented)

**Mobile UI (`client/mobile.html` + `client/js/app.js`):**
- `roll-request-panel` div (hidden by default)
- Displays when `roll_requested` message received
- Shows all roll info with styling
- "Execute Roll" button sends `execute_roll` message
- Auto-hides after submission

**GM UI (`client/gm.html` + `client/js/gm.js`):**
- Roll request form in sidebar
- Target dropdown (populated with current characters)
- Sends `request_roll` message
- Status panel tracks completion
- Auto-hides when all rolls complete

**TV Display (`client/js/app.js`):**
- `showDetailedRollResultOnTV()` function
- Shows Hope/Fear dice with gradient styling
- Displays success/failure
- Critical success indicator
- Controlling die badge

**CSS (`client/css/style.css`):**
- `.roll-request-panel` - Gradient Hope/Fear background
- Slide-in animation
- Responsive sizing
- Difficulty/modifier value styling

---

## ✨ Features

### Automatic Modifier Calculation
- ✅ Attribute value (e.g., Agility +2)
- ✅ Proficiency bonus (for Attack/Spellcast rolls)
- ✅ Situational modifiers (from GM)
- ✅ Hope bonus (+2 if spent)

### Roll Types Supported
- ✅ **Action** - General skill check
- ✅ **Attack** - Adds proficiency bonus
- ✅ **Spellcast** - Adds proficiency bonus
- ✅ **Save** - Reactive roll

### Target Types
- ✅ **All Players** - Send to everyone
- ✅ **Specific Character** - Target one player
- ✅ **NPC** - (GM-controlled, for future)

### Success Determination
- ✅ Failure (total < DC)
- ✅ Success with Hope (Hope die controls, total >= DC)
- ✅ Success with Fear (Fear die controls, total >= DC)
- ✅ Critical Success (doubles rolled)

### Resource Management
- ✅ Success with Hope → Player gains +1 Hope
- ✅ Success with Fear → GM gains +1 Fear
- ✅ Spend Hope for +2 → Costs 1 Hope
- ✅ Hope/Fear automatically sync across all clients

---

## 🎨 UI/UX Highlights

### Mobile (Player) View
- **Gradient panel** (Hope gold → Fear purple)
- **Clear context** - "Why am I rolling?"
- **Your stats** - Auto-calculated modifier shown
- **Advantage indicator** - Yellow highlight if applicable
- **Hope option** - Only shows if you have Experiences
- **One-click roll** - Executes and hides

### GM View
- **Quick request panel** - All options in one place
- **Smart dropdown** - Auto-populates with current characters
- **Live status** - See who's rolled and who hasn't
- **Auto-dismiss** - Status panel hides when complete
- **No clutter** - Clean, focused interface

### TV View
- **Big, bold results** - Easy to read from couch
- **Gradient dice** - Hope (gold) vs Fear (purple)
- **Success indicator** - Green/Red badges
- **Critical flash** - Special styling for crits
- **Auto-dismiss** - 6 second display

---

## 🧪 Testing Scenarios

### Basic Roll
- GM requests roll
- Player rolls
- Success/failure determined
- Result displayed on TV

### Multiple Players
- GM requests "All Players" roll
- Each player sees request
- Each rolls independently
- GM sees status update as each completes
- TV shows each result in sequence

### With Advantage
- GM checks "With Advantage"
- Player rolls 3 dice (2 Hope, 1 Fear, 1 Advantage d6)
- Advantage die added to Hope
- Result calculated correctly

### Spend Hope
- Player has at least 1 Experience
- GM requests roll
- Player sees "Spend Hope for +2" option
- Player checks box and rolls
- Gets +2 to total
- Loses 1 Hope

### Edge Cases
- Player with 0 Hope → Can't spend Hope (option hidden)
- Player with 0 Experiences → Can't spend Hope (option hidden)
- DC > 24 (very hard) → Still works
- Critical success (doubles) → Special display

---

## 📝 Known Limitations (By Design)

### Current MVP Scope
- ⚠️ No "Decline to Roll" option (assumes cooperation)
- ⚠️ No roll history panel (console logs only)
- ⚠️ No Experience selection UI (auto-uses first Experience)
- ⚠️ No group roll results summary (shows individually)
- ⚠️ No narrative stakes display on TV (just console)

### Future Enhancements (Post-MVP)
- [ ] Roll history log
- [ ] Experience selection dropdown
- [ ] Narrative stakes display
- [ ] Group roll summary view
- [ ] Decline/postpone roll option
- [ ] Roll request templates (save common rolls)
- [ ] Macro support ("Everyone roll Perception DC 10")

---

## 🎯 Success Criteria ✅

- [x] GM can request rolls from players
- [x] Players see requests with full details
- [x] Modifiers calculated automatically
- [x] Players can execute rolls
- [x] Results broadcast to all clients
- [x] Success/failure determined correctly
- [x] Hope/Fear resources update
- [x] Status tracking shows completion
- [x] TV displays results beautifully
- [x] UI is intuitive and responsive

---

## 🚀 What's Next

With GM Roll Requests complete, you can now:

1. **Run actual game sessions** - GM can request skill checks, saves, etc.
2. **Test real scenarios** - "Roll Agility to dodge", "Roll Instinct to notice"
3. **Move to Combat** - Initiative, attack rolls, damage (next feature)
4. **Add Movement** - Click-to-move on map (quick win)

---

## 📚 Documentation

- **Backend Protocol:** `server/src/protocol.rs`
- **Game Logic:** `server/src/game.rs` (`execute_roll` function)
- **Mobile UI:** `client/mobile.html` + `client/js/app.js`
- **GM UI:** `client/gm.html` + `client/js/gm.js`
- **Styling:** `client/css/style.css`

---

## 🎉 Feature Complete!

The GM Roll Request system is **fully functional** and ready for testing!

**Test it now:**
1. Refresh all browser windows (Ctrl+Shift+R)
2. Open GM view: http://192.168.1.119:3000/gm
3. Create 2 characters in mobile views
4. Request a roll from GM
5. Watch the magic happen! 🎲✨

---

**Implemented by:** OpenClaw Agent  
**Date:** 2026-02-24  
**Status:** ✅ **COMPLETE**
