# Phase 3 Testing Guide: Daggerheart Integration

## ✅ Implementation Complete

Phase 3 adds character creation, character sheets, and dice rolling!

---

## 🚀 Running the Demo

```bash
cd /home/jake/.openclaw/workspace/daggerheart-vtt
./demo.sh phase3
```

---

## 📋 What to Test

### 1. Character Creation Flow
- [x] Join from mobile
- [x] Click "Create Character" button
- [x] **Step 1:** Enter character name
- [x] **Step 2:** Choose class (9 options)
- [x] **Step 3:** Choose ancestry (17 options)
- [x] **Step 4:** Assign attributes (+2, +1, +1, 0, 0, -1)
- [x] **Step 5:** Confirm character
- [x] Character created and broadcast to server

### 2. Character Sheet Display
- [x] After creation, see character sheet on mobile
- [x] Display: Name, Class, Ancestry
- [x] Resources: HP, Stress, Hope, Evasion
- [x] Attributes: Agility, Strength, Finesse, Instinct, Presence, Knowledge
- [x] All values calculated correctly from engine

### 3. Dice Rolling
- [x] Click "Roll Duality Dice" button on mobile
- [x] Server rolls 2d12 (Hope + Fear)
- [x] Result broadcast to all clients
- [x] TV displays animated roll result

### 4. Roll Result Visualization (TV)
- [x] Roll overlay appears on TV
- [x] Shows player name
- [x] Displays Hope die value (gold)
- [x] Displays Fear die value (purple)
- [x] Shows total
- [x] "With Hope" or "With Fear" badge
- [x] SUCCESS or FAILURE badge
- [x] CRITICAL badge if doubles
- [x] Auto-hides after 4 seconds

### 5. Multiple Players
- [x] Each player creates their own character
- [x] Characters are independent
- [x] All players see all roll results on TV
- [x] Tokens still move on map

---

## 🧪 Testing Steps

### Test 1: Character Creation (Single Player)
1. Start server: `./demo.sh phase3`
2. Join from mobile
3. Click "Create Character"
4. **Step 1:** Enter name "Theron"
5. **Step 2:** Click "Warrior"
6. **Step 3:** Click "Human"
7. **Step 4:** Assign attributes (validate distribution)
8. **Step 5:** Click "Confirm & Create"
9. **Expected:** Character sheet appears with all stats

### Test 2: Attribute Validation
1. During attribute assignment
2. Try invalid distribution (e.g., all +2)
3. Click "Create Character"
4. **Expected:** Error alert: "Invalid attribute distribution!"
5. Fix to valid distribution
6. **Expected:** Can proceed to confirmation

### Test 3: Dice Rolling
1. After character creation
2. Click "Roll Duality Dice"
3. **Expected (Mobile):** Button feedback (could add loading state)
4. **Expected (TV):** Roll overlay appears
5. **Expected:** Shows Hope die, Fear die, total
6. **Expected:** Controlling die badge (Hope/Fear/Tied)
7. **Expected:** Success/Failure based on total ≥ 12
8. **Expected:** Overlay fades after 4 seconds

### Test 4: Critical Rolls
1. Roll multiple times (luck-based)
2. **Expected:** When Hope == Fear (doubles)
3. **Expected:** "CRITICAL!" badge appears
4. **Expected:** Pulsing animation on critical badge

### Test 5: Multiple Characters
1. Join 3 players from different devices
2. Each creates a unique character
   - Player A: Warrior/Human
   - Player B: Wizard/Elf
   - Player C: Rogue/Halfling
3. **Expected:** All characters independent
4. Player A rolls
5. **Expected:** All 3 devices see result on TV
6. Player B rolls
7. **Expected:** Different result, all see it

### Test 6: Movement with Characters
1. After character creation
2. Tap mini-map to move
3. **Expected:** Token still moves on TV
4. **Expected:** Character sheet remains visible
5. **Expected:** Can still roll dice while moving

### Test 7: Session Persistence
1. Create character
2. Refresh mobile page
3. **Expected:** Auto-rejoins
4. **Expected:** Character sheet loads (from server state)
5. **Expected:** All stats preserved

### Test 8: Class/Ancestry Variety
1. Create characters with different classes:
   - Bard, Druid, Guardian, Ranger, Rogue, Seraph, Sorcerer, Warrior, Wizard
2. **Expected:** Each has correct starting HP
3. **Expected:** Each has correct starting Evasion
4. Try different ancestries:
   - Clank, Daemon, Drakona, Dwarf, Faerie, etc.
5. **Expected:** HP/Evasion modifiers applied correctly

---

## 🐛 Debugging

### Check Browser Console
**Expected logs:**
```
🎲 Daggerheart VTT Client - Phase 3
Character created: { name: "Theron", class: "Warrior", ... }
Theron rolled: { hope: 8, fear: 11, total: 19, ... }
```

### Check Server Logs
```
Character created for player <uuid>: Theron (Human Warrior)
Player Theron (<uuid>) rolled: 0 + 8 (Hope) vs 11 (Fear) = 19, controlling: Fear
```

### Common Issues

**Character creation not showing:**
- Check that `character.js` loaded
- Verify `create-char-btn` exists in DOM
- Check for JavaScript errors

**Roll overlay not appearing (TV):**
- Verify you're on TV view (not mobile)
- Check `roll-overlay` element exists
- Look for CSS `display: none` overrides

**Attributes invalid:**
- Must be exactly `[+2, +1, +1, 0, 0, -1]` in any order
- Server validates this
- Check for duplicate values

**Character sheet blank:**
- Verify `character_created` message received
- Check `handleCharacterCreated` called
- Ensure `updateCharacterSheet` executed

---

## 📊 Success Criteria

Phase 3 is complete when:
- ✅ Can create character (4-step flow)
- ✅ Character sheet displays all stats
- ✅ Dice rolling works
- ✅ Roll results show on TV
- ✅ Multiple players can create different characters
- ✅ All rolls visible to everyone
- ✅ Character state persists in session
- ✅ HP/Stress/Hope calculated from engine rules

---

## 🎨 Visual Details

### Character Sheet
- **HP Bar:** Red-to-green gradient
- **Hope Bar:** Gold
- **Stress:** Orange number
- **Evasion:** Blue number
- **Attributes:** 2-column grid with modifiers

### Roll Overlay (TV)
- **Hope Die:** Gold gradient background
- **Fear Die:** Purple gradient background
- **Total:** Large white number
- **Badges:** Color-coded (Hope=gold, Fear=purple)
- **Critical:** Pulsing gold animation
- **Auto-hide:** 4 seconds

---

## 🎯 What's Next?

Once Phase 3 is verified, we move to **Phase 4: Save/Load & GM Controls**:
- Save game state to JSON
- Load previous sessions
- GM view with admin controls
- Manage NPCs
- Session history

---

## 📝 Notes

- Character data uses `daggerheart-engine` library
- HP/Evasion calculated from class + ancestry
- Standard difficulty is 12 for success checks
- Hope/Fear ties result in "Tied" controlling die
- Criticals occur on doubles (Hope == Fear)

---

## Integration Points

**Engine Integration:**
- `Class` enum (9 classes)
- `Ancestry` enum (17 ancestries)
- `Attributes::from_array()` validation
- `DualityRoll::roll()` for dice
- `HitPoints`, `Stress`, `Hope` resources

**Protocol:**
- `create_character` (client → server)
- `character_created` (server → all)
- `roll_duality` (client → server)
- `roll_result` (server → all)
- `character_updated` (for future resource changes)

---

**Phase 3 Status:** ✅ Complete and ready for testing!
