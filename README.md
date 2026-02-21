# Daggerheart VTT

**A couch-coop virtual tabletop for Daggerheart TTRPG**

Built on top of [daggerheart-engine](../daggerheart-engine) - a complete Rust implementation of the Daggerheart rules system.

---

## 🎯 Vision

A collaborative storytelling tool where:
- **One screen** (TV/monitor) shows the shared game world
- **Multiple players** join from their phones via QR code
- **GM controls** the session from any client
- **Real-time sync** keeps everyone on the same page
- **Visual-first** gameplay with map-based interactions

**Think:** Jackbox games meets D&D Beyond meets Roll20, but for couch co-op.

---

## 🏗️ Architecture

### **Tech Stack**
- **Backend:** Rust + Axum (web server) + WebSockets
- **Frontend:** HTML5/JavaScript/Canvas (works on phones + TV browsers)
- **Game Engine:** [daggerheart-engine](../daggerheart-engine) (Rust library)
- **Real-time:** WebSocket for state synchronization
- **Rendering:** HTML5 Canvas for 2D maps

### **Repository Structure**
```
daggerheart-vtt/
├── server/           # Rust backend (Axum + WebSocket)
│   ├── src/
│   │   ├── main.rs
│   │   ├── game.rs       # Game state management
│   │   ├── websocket.rs  # WebSocket handlers
│   │   └── routes.rs     # HTTP routes
│   └── Cargo.toml
├── client/           # Web frontend
│   ├── index.html        # TV/Desktop view
│   ├── mobile.html       # Phone view
│   ├── js/
│   │   ├── app.js        # Main client logic
│   │   ├── websocket.js  # WebSocket client
│   │   ├── canvas.js     # Map rendering
│   │   └── ui.js         # UI components
│   └── css/
│       └── style.css
├── shared/           # Shared types (Rust ↔ JSON)
│   └── protocol.rs       # Message protocol definitions
├── docs/             # Documentation
│   ├── ARCHITECTURE.md
│   ├── PROTOCOL.md       # WebSocket message spec
│   └── PHASES.md         # Development phases
├── demo.sh           # Phase demo runner
├── Cargo.toml        # Workspace config
└── README.md         # This file
```

---

## 🚀 Development Phases

### **Phase 1: Foundation & Connection** ✅
**Goal:** Players can connect to the server

**Features:**
- [x] Rust server running on localhost
- [x] Web client accessible from browser
- [x] QR code generation for phone joining
- [x] WebSocket connection
- [x] Connected players list

**Demo:** `./demo.sh phase1`

---

### **Phase 2: Basic Map & Movement**
**Goal:** See and move players on a 2D map

**Features:**
- [ ] 2D canvas map (top-down view)
- [ ] Players rendered as colored circles
- [ ] Name labels above circles
- [ ] Phone controls: tap to move
- [ ] TV shows synchronized movement
- [ ] Simple grid or background image

**Demo:** `./demo.sh phase2`

---

### **Phase 3: Daggerheart Integration**
**Goal:** Use the engine for dice rolling and character sheets

**Features:**
- [ ] Character creation (name, class, ancestry)
- [ ] "Roll Duality" button on phone
- [ ] Server processes rolls via daggerheart-engine
- [ ] TV shows roll results (Hope/Fear visualization)
- [ ] Basic character sheet view
- [ ] HP/Stress tracking

**Demo:** `./demo.sh phase3`

---

### **Phase 4: Save/Load & GM Controls**
**Goal:** Full session lifecycle with persistence

**Features:**
- [ ] Save game state to JSON
- [ ] Load previous sessions
- [ ] GM view: manage NPCs, see all stats
- [ ] Add/remove players mid-session
- [ ] Session history

**Demo:** `./demo.sh phase4`

---

## 🎮 Usage

### **Quick Start**

```bash
# Terminal 1: Start server
cd server
cargo run

# Terminal 2: Open client
# TV/Desktop view: http://localhost:3000
# Mobile view: http://localhost:3000/mobile

# Phones: Scan QR code shown on TV
```

### **Running Demos**

```bash
# Run phase 1 demo
./demo.sh phase1

# Run phase 2 demo (when available)
./demo.sh phase2
```

---

## 🛠️ Development

### **Prerequisites**
- Rust 1.70+ (for server)
- Modern web browser (for client)
- Local network (for phone connections)

### **Setup**

```bash
# Clone the repository
cd /path/to/workspace
git clone <repo-url> daggerheart-vtt
cd daggerheart-vtt

# Build server
cd server
cargo build

# No build needed for client (vanilla HTML/JS)
```

### **Development Workflow**

```bash
# Server (with hot-reload)
cd server
cargo watch -x run

# Client
# Just refresh browser - no build step!
```

### **Testing**

```bash
# Server tests
cd server
cargo test

# Integration tests
./test.sh
```

---

## 📱 Client Views

### **TV/Desktop View** (`/`)
- Full map canvas
- All players visible
- Dice roll results
- Session info
- QR code for joining

### **Mobile View** (`/mobile`)
- Your character sheet
- Movement controls
- Dice roll buttons
- Chat/actions
- Minimal UI for small screens

### **GM View** (`/gm`)
- Everything from TV view
- NPC management
- Session controls (save/load/reset)
- Player stats overview

---

## 🌐 Network Setup

### **Local Network (Same WiFi)**
1. Server starts on `0.0.0.0:3000`
2. Find server IP: `ip addr` or `ifconfig`
3. TV browser: `http://<server-ip>:3000`
4. Phones scan QR code or navigate to `http://<server-ip>:3000/mobile`

### **Port Forwarding (Internet Access)**
- For remote players, set up port forwarding on router
- Or use ngrok: `ngrok http 3000`

---

## 🎨 Visual Design

### **Map System**
- 2D top-down view
- Grid-based or free-form movement
- Player tokens: colored circles with name labels
- NPC tokens: different shape (squares?)
- Future: Import custom map images

### **Color Scheme**
- **Hope:** Bright gold/yellow (`#FFD700`)
- **Fear:** Deep purple/red (`#8B008B`)
- **Background:** Dark theme for TV viewing
- **Player colors:** Rainbow palette (auto-assigned)

---

## 🔮 Future Features (Post-MVP)

### **Phase 5+: Enhanced Gameplay**
- Combat system (initiative, attacks, damage)
- Domain cards UI
- Inventory management
- NPC stat blocks
- Conditions/status effects

### **Visuals & Polish**
- Better graphics (Phaser.js or similar game engine?)
- Animated dice rolls
- Particle effects for abilities
- Character avatars
- Map layers (fog of war?)

### **Social Features**
- Voice chat integration
- Text chat
- Emotes/reactions
- Session recordings

### **Content**
- Pre-made maps
- NPC library
- Campaign templates
- Adventure modules

---

## 📊 Current Status

**Phase:** 1 (Foundation & Connection)  
**Progress:** Setting up repository structure  
**Next Step:** Implement server foundation

---

## 🤝 Contributing

This is a personal project for learning and fun. Suggestions welcome!

---

## 📜 License

MIT OR Apache-2.0 (same as daggerheart-engine)

---

## 🙏 Acknowledgments

- **[daggerheart-engine](../daggerheart-engine)** - The core rules engine
- **Daggerheart SRD** - Game mechanics
- **Critical Role** - For creating Daggerheart

---

## 🔗 Links

- **Repository:** https://github.com/jakeaboganda/daggerheart-vtt
- **Engine Repository:** https://github.com/jakeaboganda/daggerheart-engine
- **Daggerheart Official:** https://www.daggerheart.com/
- **Daggerheart SRD:** https://www.daggerheart.com/srd/

---

**Built with ❤️ for couch co-op storytelling** 🎲🗡️❤️
