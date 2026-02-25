# Daggerheart VTT - Project Status

**Last Updated:** 2026-02-25 22:10 JST  
**Current Phase:** Phase 5A - Character-Centric Architecture + Combat System

---

## 📋 Project Overview

A real-time Virtual Tabletop (VTT) for the Daggerheart TTRPG system, built with:
- **Backend:** Rust (Axum, WebSocket, Tokio)
- **Frontend:** Vanilla JavaScript + Canvas
- **Engine:** Custom `daggerheart-engine` crate (dice mechanics, characters, combat)
- **Views:** GM view, TV view (display), Mobile view (players)

---

## ✅ What Currently Works

### **Core System**
- ✅ WebSocket real-time communication (server → all clients)
- ✅ Multi-view support (GM, TV, Mobile)
- ✅ Character creation & management
- ✅ Character movement (click-to-move)
- ✅ Position synchronization across all clients
- ✅ Connection/disconnection handling
- ✅ Character removal on disconnect (broadcasts to all clients)

### **Adversary System**
- ✅ Spawn adversaries from templates (Goblin, Bandit, Wolf, Orc, Ogre, Dragon)
- ✅ Custom adversary creation (name, HP, evasion, armor, attack, damage)
- ✅ Click-to-spawn on map (GM view)
- ✅ Adversary rendering on canvas (red tokens with skull icon, HP bars)
- ✅ Adversary list synchronization on client join
- ✅ Adversaries visible on TV view on load
- ✅ Adversaries visible on mobile view (after character selection)
- ✅ Adversary removal

### **Combat System** (NEW!)
- ✅ Start/End combat
- ✅ Action tracker (PC/Adversary tokens)
- ✅ **Click-to-attack** on map:
  - Click character → Select as attacker (gold ring highlight)
  - Click adversary → Roll attack (Hope/Fear vs Evasion)
  - Attack result overlay (dice, total, hit/miss, critical)
  - If hit → Roll damage button
  - Damage result overlay (HP loss, stress, armor reduction)
- ✅ Attack rolls broadcast to all clients
- ✅ Damage rolls update adversary HP
- ✅ HP updates visible across all views
- ✅ "Taken Out" detection

### **Duality Dice Rolls**
- ✅ GM can request rolls from players (attribute, difficulty, modifiers)
- ✅ Players receive roll requests on mobile
- ✅ Players can spend Hope for +1d6 bonus
- ✅ Roll results broadcast to TV view (large overlay)
- ✅ Hope/Fear dice, controlling die, success/failure
- ✅ Critical success detection

### **UI/UX**
- ✅ Responsive canvas rendering (800x600, grid overlay)
- ✅ Character tokens with color-coded rings
- ✅ Adversary tokens with HP bars
- ✅ Smooth movement animations
- ✅ Event log (GM view, TV view)
- ✅ QR code for mobile connection
- ✅ LocalStorage session persistence (mobile)
- ✅ Character sheet display (mobile)

---

## 🆕 Recent Session Accomplishments (2026-02-25)

### **Bug Fixes**
1. ✅ **Fixed adversary spawning** - Corrected WebSocket message format (`ws.send(type, payload)`)
2. ✅ **Fixed TV/mobile adversary sync** - Added `adversaries_list` message on connection
3. ✅ **Fixed character persistence bug** - Server now broadcasts `character_removed` when player disconnects
4. ✅ **Fixed mobile adversary timing** - Store adversaries in `allAdversaries` array, apply when canvas is created

### **New Features**
1. ✅ **Adversary list on join** - New connections receive full adversary state
2. ✅ **Click-to-attack combat**:
   - Canvas click detection for characters/adversaries
   - Visual selection (gold ring)
   - Attack/damage flow with result overlays
   - Automatic HP updates
3. ✅ **Combat feedback banner** - Shows current action/instruction during combat

### **Code Quality**
- ✅ Added debug logging for adversary state tracking
- ✅ Bumped script versions (v12) to force cache refresh
- ✅ Improved code organization (combat handlers separated)
- ✅ Git commits with clear messages

---

## 🏗️ Architecture Overview

### **Server Structure**
```
server/src/
├── main.rs              # Axum server, routes, static files
├── websocket.rs         # WebSocket handlers, message routing
├── protocol.rs          # ClientMessage/ServerMessage enums
├── game.rs              # GameState, character/adversary logic
└── adversaries.rs       # Adversary templates
```

### **Client Structure**
```
client/
├── gm.html              # GM view (control panel + map)
├── index.html           # TV view (display only)
├── mobile.html          # Mobile view (player control)
├── js/
│   ├── websocket.js     # WebSocket client wrapper
│   ├── canvas.js        # MapCanvas class (rendering, click detection)
│   ├── gm.js            # GM view logic (combat, spawning, rolls)
│   ├── app.js           # TV/mobile logic (shared)
│   └── character.js     # Character creation UI
└── css/
    └── style.css        # Global styles
```

### **Message Flow**
```
Client → Server:
  - connect
  - create_character, select_character
  - move_character
  - roll_duality, execute_roll
  - spawn_adversary, remove_adversary
  - start_combat, end_combat
  - attack, roll_damage

Server → Client:
  - connected
  - characters_list, adversaries_list
  - character_spawned, character_removed, character_moved
  - adversary_spawned, adversary_removed, adversary_updated
  - combat_started, combat_ended, tracker_updated
  - attack_result, damage_result
  - roll_result, roll_requested
  - game_event, error
```

---

## 🐛 Known Issues

### **Minor**
1. ⚠️ No modifier input for attacks (currently hardcoded to 0)
2. ⚠️ No advantage checkbox for attacks (currently false)
3. ⚠️ Character damage dice not implemented (uses placeholder "1d8")
4. ⚠️ Character armor not implemented (uses placeholder "1")
5. ⚠️ Action tracker token draw not implemented (manual token management only)
6. ⚠️ Adversary attack on player not implemented (only PC → Adversary works)

### **Edge Cases**
1. ⚠️ If attacker is deleted mid-combat, `rollDamageForLastAttack()` may fail
2. ⚠️ No confirmation when ending combat
3. ⚠️ Adversary HP can go negative (cosmetic only)

### **Polish**
1. ⚠️ No sound effects
2. ⚠️ No attack animations
3. ⚠️ Combat result overlay has no backdrop (can click through)
4. ⚠️ Mobile combat UI not implemented (players can't attack from mobile)

---

## 🎯 Future Roadmap

### **Phase 6: Combat Polish** (Next Session)
**Priority: High**

1. **Combat UI Improvements**
   - [ ] Add modifier input field (popup or sidebar)
   - [ ] Add advantage checkbox for attacks
   - [ ] Add backdrop to combat result overlay
   - [ ] Confirmation dialog for "End Combat"
   - [ ] Visual attack animation (line from attacker → target)
   - [ ] Sound effects (attack, hit, miss, damage)

2. **Mobile Combat**
   - [ ] Attack button on character sheet (mobile)
   - [ ] Target selection UI (mobile)
   - [ ] Attack result display (mobile)
   - [ ] Damage roll button (mobile)

3. **Adversary Attacks**
   - [ ] Click adversary → Select as attacker
   - [ ] Click character → Roll adversary attack
   - [ ] Adversary damage applies to character HP/Stress
   - [ ] Character "Taken Out" detection

4. **Action Tracker**
   - [ ] Implement token draw mechanism
   - [ ] Visual token bag animation
   - [ ] "Next Turn" button (draws token)
   - [ ] Highlight whose turn it is
   - [ ] Reset tracker button functionality

### **Phase 7: Character Development**
**Priority: Medium**

1. **Equipment System**
   - [ ] Weapon slots (melee, ranged)
   - [ ] Armor slots
   - [ ] Weapon damage dice
   - [ ] Armor values
   - [ ] Equipment UI (mobile character sheet)

2. **Abilities & Experiences**
   - [ ] Foundation ability selection
   - [ ] Experience cards
   - [ ] Domain cards
   - [ ] Ability activation UI
   - [ ] Resource tracking (Hope, Armor, Fear)

3. **Character Progression**
   - [ ] XP tracking
   - [ ] Level up system
   - [ ] Mark experiences
   - [ ] Skill improvements

### **Phase 8: Advanced Combat**
**Priority: Medium**

1. **Combat Features**
   - [ ] Stress overflow → HP damage
   - [ ] Mark experiences from combat
   - [ ] Multiple attacks per turn
   - [ ] Area-of-effect abilities
   - [ ] Reactions (defensive rolls)

2. **GM Tools**
   - [ ] Initiative tracker (alternative to action tracker)
   - [ ] Adversary stat editing mid-combat
   - [ ] Temporary conditions/effects
   - [ ] Status markers on tokens

### **Phase 9: World Building**
**Priority: Low**

1. **Maps & Scenes**
   - [ ] Upload custom map images
   - [ ] Multiple scenes/maps
   - [ ] Fog of war
   - [ ] Token size options
   - [ ] Grid overlay toggle

2. **Campaign Management**
   - [ ] Multiple campaigns
   - [ ] Save/load game states
   - [ ] Session notes
   - [ ] Campaign journal

### **Phase 10: Polish & Release**
**Priority: Low**

1. **Performance**
   - [ ] Optimize canvas rendering
   - [ ] Reduce WebSocket message size
   - [ ] Connection recovery on disconnect
   - [ ] Lazy loading for large campaigns

2. **UX Polish**
   - [ ] Tutorial/onboarding
   - [ ] Keyboard shortcuts
   - [ ] Mobile gesture support (pinch-zoom)
   - [ ] Accessibility (screen reader support)

3. **Deployment**
   - [ ] Docker container
   - [ ] Reverse proxy setup (nginx)
   - [ ] HTTPS/WSS support
   - [ ] Environment config (.env)

---

## 🧪 Testing Instructions

### **Setup**
```bash
cd ~/.openclaw/workspace/daggerheart-vtt
cargo run --bin daggerheart-vtt-server
```

**Open in browser:**
- GM: `http://localhost:3000/gm`
- TV: `http://localhost:3000/`
- Mobile: `http://localhost:3000/mobile` (or scan QR from TV)

### **Test Sequence: Combat Flow**

1. **GM View:**
   - Click "➕ Click Map to Spawn"
   - Select "Goblin"
   - Click on map → Goblin appears
   - Click "▶️ Start Combat"

2. **Mobile View:**
   - Join game, create character
   - Character appears on map

3. **GM View (Attack Test):**
   - Click player character → Gold ring appears
   - Click Goblin → Attack result overlay shows
   - If hit → Click "💥 Roll Damage"
   - Damage result shows, Goblin HP updates

4. **TV View:**
   - Verify Goblin appears
   - Verify character appears
   - Verify HP bar updates on damage

5. **Disconnect Test:**
   - Close mobile tab
   - Verify character disappears from TV/GM
   - Verify Goblin stays on map

### **Test Sequence: Adversary Persistence**

1. Spawn 3 Goblins on GM view
2. Close TV tab completely
3. Open new TV tab
4. Verify all 3 Goblins appear immediately

---

## 🔧 Technical Details

### **Key Files Modified (Last Session)**
```
client/js/canvas.js        # Added click detection, selection highlighting
client/js/gm.js            # Added combat click handlers, result overlays
client/js/app.js           # Added adversary list handler, storage
client/gm.html             # Added combat feedback banner, result overlay
server/src/websocket.rs    # Added character_removed broadcast, adversaries_list
server/src/protocol.rs     # Added AdversariesList, AdversaryInfo
```

### **Important Variables**
```javascript
// gm.js
let characters = [];          // All characters in game
let adversaries = [];         // All adversaries in game
let selectedAttackerId = null; // Currently selected attacker (combat)
let combatActive = false;      // Is combat mode active?
let spawnMode = false;         // Is spawn mode active?

// app.js (TV/Mobile)
let allCharacters = [];       // All characters (for canvas repopulation)
let allAdversaries = [];      // All adversaries (for canvas repopulation)
let mapCanvas = null;         // MapCanvas instance

// canvas.js
this.players = Map;           // character_id → {id, name, position, color}
this.adversaryPositions = Map; // adversary_id → {name, x, y, hp, maxHp}
this.selectedAttackerId = null; // Gold ring highlight
```

### **WebSocket Message Examples**
```javascript
// Attack
ws.send('attack', {
  attacker_id: 'char-uuid',
  target_id: 'adv-uuid',
  modifier: 0,
  with_advantage: false
});

// Damage
ws.send('roll_damage', {
  attacker_id: 'char-uuid',
  target_id: 'adv-uuid',
  damage_dice: '1d6',
  armor: 1
});

// Spawn adversary
ws.send('spawn_adversary', {
  template: 'goblin',
  position: { x: 100.0, y: 200.0 }
});
```

### **Server Response Examples**
```json
// Attack result
{
  "type": "attack_result",
  "payload": {
    "attacker_id": "...",
    "attacker_name": "Elara",
    "target_id": "...",
    "target_name": "Goblin #1",
    "hope": 12,
    "fear": 6,
    "modifier": 0,
    "total": 12,
    "target_evasion": 10,
    "hit": true,
    "controlling_die": "hope",
    "is_critical": false
  }
}

// Damage result
{
  "type": "damage_result",
  "payload": {
    "target_id": "...",
    "target_name": "Goblin #1",
    "raw_damage": 4,
    "after_armor": 3,
    "hp_lost": 3,
    "stress_gained": 0,
    "new_hp": 0,
    "new_stress": 0,
    "taken_out": true
  }
}
```

---

## 📝 Development Notes

### **Adversary Templates** (`server/src/adversaries.rs`)
```rust
// Available templates:
- goblin:         HP 3,  Evasion 10, Armor 1, Attack +1, Damage 1d6
- bandit:         HP 4,  Evasion 11, Armor 1, Attack +2, Damage 1d6+1
- wolf:           HP 3,  Evasion 12, Armor 0, Attack +2, Damage 1d6
- orc_warrior:    HP 5,  Evasion 10, Armor 2, Attack +2, Damage 1d8
- shadow_beast:   HP 4,  Evasion 13, Armor 1, Attack +3, Damage 1d6+1
- ogre:           HP 8,  Evasion 9,  Armor 3, Attack +3, Damage 1d10
- dragon_wyrmling: HP 10, Evasion 12, Armor 2, Attack +4, Damage 1d12
```

### **Canvas Coordinates**
- Map size: 800x600 pixels
- Grid: 50px squares
- Player radius: 20px
- Adversary radius: 20px
- Click hitbox: radius + 5px

### **Style Variables** (CSS)
```css
--bg-dark: #1a1a1a
--bg-medium: #2c2c2c
--text-light: #e0e0e0
--accent: #3498db
--success: #2ecc71
--danger: #e74c3c
--warning: #f39c12
```

---

## 🚀 Quick Start (Next Session)

### **To Resume Development:**
```bash
cd ~/.openclaw/workspace/daggerheart-vtt
cargo run --bin daggerheart-vtt-server
# Open http://localhost:3000/gm
```

### **High-Priority TODOs:**
1. Add modifier input UI for attacks
2. Add advantage checkbox for attacks
3. Implement mobile attack button
4. Implement adversary → character attacks
5. Add action tracker token draw

### **Low-Hanging Fruit:**
- Add backdrop to combat result overlay (1 line CSS)
- Add confirmation to "End Combat" (2 lines JS)
- Fix adversary HP clamping (prevent negative, 3 lines Rust)
- Add sound effects (HTML5 Audio API, ~20 lines)

---

## 📚 Resources

- **Daggerheart Rules:** https://darringtonpress.com/daggerheart/
- **Project Repository:** (TBD - not yet pushed)
- **Rust Axum Docs:** https://docs.rs/axum/
- **Canvas API:** https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API

---

## 🎉 Achievements Unlocked

- ✅ Real-time multiplayer VTT
- ✅ Cross-device sync (GM/TV/Mobile)
- ✅ Character creation & movement
- ✅ Adversary spawning & management
- ✅ Click-to-attack combat system
- ✅ Duality dice mechanics
- ✅ HP/Stress tracking
- ✅ Action tracker framework
- ✅ ~2,500 lines of Rust backend
- ✅ ~1,500 lines of JavaScript frontend

---

**Session End:** 2026-02-25 22:10 JST  
**Next Priority:** Combat UI polish & mobile attack support  
**Status:** ✅ Fully functional combat prototype ready for testing!
