# Architecture

## System Overview

```
┌─────────────────┐
│   TV/Desktop    │
│   (Browser)     │
└────────┬────────┘
         │
         │ WebSocket
         │
┌────────▼────────┐      ┌──────────────────┐
│                 │      │  daggerheart-    │
│  Axum Server    │─────▶│     engine       │
│  (Rust/Tokio)   │      │   (Rust lib)     │
│                 │      └──────────────────┘
└────────┬────────┘
         │
         │ WebSocket
         │
┌────────▼────────┐
│  Mobile Phone   │
│   (Browser)     │
└─────────────────┘
```

---

## Components

### 1. Server (Rust)

**Technology:** Axum + Tokio + WebSocket

**Responsibilities:**
- WebSocket connection management
- Game state management (in-memory, then persistent)
- Message routing between clients
- Game logic via `daggerheart-engine`
- QR code generation
- Static file serving

**Key Modules:**
- `main.rs` - Server entry point, routing
- `game.rs` - Game state and logic
- `websocket.rs` - WebSocket handlers
- `routes.rs` - HTTP routes
- `protocol.rs` - Message types (shared with client)

---

### 2. Client (HTML/CSS/JavaScript)

**Technology:** Vanilla web stack (no framework for MVP)

**Views:**
- **Desktop/TV** (`index.html`) - Main game view, QR code, map
- **Mobile** (`mobile.html`) - Player controls, character sheet
- **GM** (`gm.html`) - Admin controls (Phase 4)

**Key Modules:**
- `app.js` - Main application logic
- `websocket.js` - WebSocket client
- `canvas.js` - Map rendering (Phase 2)
- `ui.js` - UI components

---

### 3. Game Engine (Rust Library)

**Technology:** `daggerheart-engine` (existing library)

**Usage:**
- Character creation and validation
- Dice rolling (duality, damage, etc.)
- Combat resolution
- Progression/leveling
- Save/load serialization

**Integration:**
- Server calls engine functions directly
- Results serialized to JSON
- Sent to clients via WebSocket

---

## Data Flow

### Player Joining

```
Mobile Browser
    │
    │ 1. Load /mobile
    ▼
Server (HTTP)
    │
    │ 2. Serve mobile.html
    ▼
Mobile Browser
    │
    │ 3. User enters name, clicks Join
    │ 4. WebSocket connect
    ▼
Server (WebSocket)
    │
    │ 5. Create player session
    │ 6. Add to game state
    │ 7. Broadcast "player_joined"
    ▼
All Clients
    │
    │ 8. Update player list
    └──▶ Display new player
```

### Dice Rolling

```
Mobile
    │ 1. Tap "Roll Duality"
    ▼
Server
    │ 2. Call daggerheart_engine::DualityRoll::roll()
    │ 3. Calculate result
    ▼
All Clients
    │ 4. Broadcast "dice_rolled"
    │ 5. Animate on TV
    └──▶ 6. Show result
```

---

## State Management

### Game State Structure

```rust
struct GameState {
    session_id: Uuid,
    players: HashMap<Uuid, Player>,
    npcs: Vec<Npc>,
    map: Map,
    hope_pool: u8,
    fear_pool: u8,
}

struct Player {
    id: Uuid,
    name: String,
    character: Option<Character>,
    position: Position,
    connected: bool,
}
```

### State Transitions

- **Phase 1:** In-memory HashMap
- **Phase 2:** Add position tracking
- **Phase 3:** Add character data
- **Phase 4:** Serialize to JSON, load from disk

---

## Communication Protocol

### WebSocket Messages

All messages are JSON:

```json
{
    "type": "message_type",
    "payload": { ... }
}
```

See [PROTOCOL.md](PROTOCOL.md) for full specification.

---

## Scalability Considerations

### Current (MVP)
- Single server instance
- In-memory state (Phase 1-3)
- Local network only
- ~10 concurrent players

### Future
- Redis for shared state
- Load balancer for multiple instances
- Database for persistence
- Cloud deployment
- ~100+ concurrent players per instance

---

## Security

### MVP
- No authentication (trust-based)
- Local network only
- Session IDs for player identification

### Future
- User accounts
- Session passwords
- TLS/HTTPS
- Rate limiting
- Input validation

---

## Performance Targets

### Phase 1-2
- WebSocket latency: <50ms
- Player join: <1s
- Movement update: <16ms (60 FPS)

### Phase 3-4
- Dice roll: <100ms
- Character load: <500ms
- Save file: <1s

---

## Technology Choices

### Why Axum?
- Excellent WebSocket support
- Fast (async Rust)
- Type-safe routing
- Tower middleware ecosystem

### Why Vanilla JS?
- No build step (faster iteration)
- Smaller bundle size
- Direct browser APIs
- Easy to upgrade later (React/Vue)

### Why WebSocket?
- Real-time bidirectional communication
- Lower latency than HTTP polling
- Native browser support
- Efficient for frequent updates

### Why Canvas?
- Full control over rendering
- Good performance for 2D
- Can upgrade to WebGL later
- Easier than SVG for game graphics

---

## Deployment

### Development
```bash
# Terminal 1: Server
cd server && cargo run

# Terminal 2: Open browsers
# TV: http://localhost:3000
# Mobile: http://localhost:3000/mobile
```

### Production (Future)
- Docker container
- Nginx reverse proxy
- systemd service
- Cloud VPS (DigitalOcean, AWS, etc.)

---

## Directory Structure

```
daggerheart-vtt/
├── server/                 # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── game.rs
│   │   ├── websocket.rs
│   │   └── routes.rs
│   └── Cargo.toml
├── client/                 # Web frontend
│   ├── index.html
│   ├── mobile.html
│   ├── gm.html
│   ├── css/
│   │   └── style.css
│   └── js/
│       ├── app.js
│       ├── websocket.js
│       ├── canvas.js
│       └── ui.js
├── shared/                 # Shared types
│   └── protocol.rs
├── docs/                   # Documentation
│   ├── ARCHITECTURE.md     # This file
│   ├── PROTOCOL.md
│   └── PHASES.md
├── saves/                  # Game saves
│   └── .gitkeep
├── demo.sh                 # Demo runner
└── README.md
```

---

## Testing Strategy

### Server
- Unit tests for game logic
- Integration tests for WebSocket
- Property tests for state transitions

### Client
- Manual testing via demo
- Browser console logging
- Multiple device testing

### End-to-End
- Playwright/Puppeteer (future)
- Manual multi-client scenarios

---

## Development Workflow

1. **Write tests** (server-side)
2. **Implement feature** (server + client)
3. **Test manually** via demo script
4. **Iterate** based on testing
5. **Commit** with descriptive message
6. **Tag** at end of phase

---

**Version:** 0.1.0  
**Last Updated:** 2026-02-21  
**Status:** Phase 1 in progress
