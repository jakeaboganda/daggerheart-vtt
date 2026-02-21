# Development Phases

## Overview

The Daggerheart VTT is being developed in 4 distinct phases, each building on the previous one. Each phase ends with a working demo that can be tested.

---

## Phase 1: Foundation & Connection

**Goal:** Get the basic infrastructure working so players can connect.

### Features
- [x] Project structure created
- [ ] Rust web server (Axum)
- [ ] WebSocket support
- [ ] Static file serving
- [ ] QR code generation
- [ ] Basic HTML/CSS/JS client
- [ ] Connection handling
- [ ] Player list synchronization

### Demo Command
```bash
./demo.sh phase1
```

### Success Criteria
- Server starts without errors
- TV view shows connection QR code and URL
- Mobile view accessible on phone browsers
- Players can "join" and see their name
- TV shows list of connected players
- Real-time updates when players join/leave

### Technical Details
- **Server:** Axum + Tokio + WebSocket
- **Client:** Vanilla HTML/CSS/JavaScript
- **Protocol:** WebSocket JSON messages
- **State:** In-memory game state (no persistence yet)

---

## Phase 2: Basic Map & Movement

**Goal:** Visual representation of the game world with player movement.

### Features
- [ ] HTML5 Canvas rendering
- [ ] 2D top-down map view
- [ ] Player tokens (colored circles)
- [ ] Name labels above tokens
- [ ] Touch controls on mobile (tap to move)
- [ ] Synchronized movement across all clients
- [ ] Simple grid or background image

### Demo Command
```bash
./demo.sh phase2
```

### Success Criteria
- Map displays on TV with grid
- Each player appears as a colored circle with their name
- Tapping on mobile moves your character
- Movement is smooth and synchronized
- Can have 4+ players moving simultaneously

### Technical Details
- **Rendering:** HTML5 Canvas 2D API
- **Coordinates:** Grid-based or free-form (TBD)
- **Sync:** WebSocket broadcasts position updates
- **Mobile Input:** Touch events → tap position → server → all clients

---

## Phase 3: Daggerheart Integration

**Goal:** Integrate the game engine for dice rolling and character management.

### Features
- [ ] Character creation flow
- [ ] Character sheets (basic)
- [ ] Dice rolling UI on mobile
- [ ] Duality roll visualization on TV
- [ ] Hope/Fear tracking
- [ ] HP/Stress tracking
- [ ] Integration with `daggerheart-engine` library

### Demo Command
```bash
./demo.sh phase3
```

### Success Criteria
- Can create a character (name, class, ancestry)
- Character sheet displays on mobile
- "Roll Duality" button works
- TV shows animated roll results with Hope/Fear indicator
- HP and Stress update correctly
- GM can see all player stats

### Technical Details
- **Engine:** Call `daggerheart-engine` Rust library from server
- **Character Data:** JSON serialization via serde
- **Roll Animation:** CSS/Canvas animation on TV
- **State Management:** Characters stored in game state

---

## Phase 4: Save/Load & GM Controls

**Goal:** Complete session lifecycle with persistence and GM tools.

### Features
- [x] Save game state to JSON file
- [x] Load previous sessions
- [x] GM view with enhanced controls
- [x] Real-time player monitoring
- [x] Session history/logs
- [x] Map view for GM
- [x] Player statistics panel

### Demo Command
```bash
./demo.sh phase4
```

### Success Criteria
- Can save game state (creates `.json` file)
- Can load previous session and continue
- GM view shows all player stats and controls
- Can add NPC tokens to map
- Session persists across server restarts

### Technical Details
- **Persistence:** JSON files in `saves/` directory
- **Format:** Compatible with `daggerheart-engine` save format
- **GM UI:** Separate view with admin controls
- **NPC System:** Simplified character representation

---

## Post-MVP (Phase 5+)

### Planned Features
- Full combat system (initiative, attacks, damage)
- Domain cards UI
- Inventory management
- Map layers (fog of war)
- Better visuals (Phaser.js integration?)
- Text chat
- Voice chat integration
- Pre-made maps and content
- Campaign tools

### Nice-to-Have
- Animated dice rolls (3D?)
- Particle effects for abilities
- Character avatars/portraits
- Custom map uploads
- Session recordings/replays
- Mobile app (native?)

---

## Development Timeline

**Estimated Time per Phase:**
- Phase 1: 1-2 days (foundation work)
- Phase 2: 2-3 days (canvas rendering, controls)
- Phase 3: 3-4 days (engine integration, UI polish)
- Phase 4: 2-3 days (persistence, GM tools)

**Total MVP:** ~10 days of development

---

## Testing Strategy

Each phase includes:
1. **Unit tests** for server-side logic
2. **Manual testing** via demo script
3. **Integration tests** for WebSocket protocol
4. **Multi-client testing** (2+ browsers)

---

## Phase Transition Checklist

Before moving to the next phase:
- [ ] All features implemented
- [ ] Demo works reliably
- [ ] No critical bugs
- [ ] Code reviewed
- [ ] Documentation updated
- [ ] Git commit with phase tag

---

**Current Phase:** 1 (Foundation & Connection)  
**Status:** Repository structure created  
**Next:** Implement WebSocket server
