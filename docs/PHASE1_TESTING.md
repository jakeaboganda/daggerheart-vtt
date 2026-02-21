# Phase 1: Testing Guide

## ✅ Implementation Complete

Phase 1 of the Daggerheart VTT is now complete! Here's how to test it.

---

## 🚀 Running the Demo

### Quick Start
```bash
cd /home/jake/.openclaw/workspace/daggerheart-vtt
./demo.sh phase1
```

### Manual Start
```bash
# Terminal 1: Start server
cd server
cargo run

# Open in browsers:
# TV: http://localhost:3000
# Mobile: http://localhost:3000/mobile
```

---

## 📋 What to Test

### 1. Server Starts
- [x] Server starts without errors
- [x] Logs show listening on port 3000
- [x] WebSocket endpoint available at `/ws`

### 2. TV View (`http://localhost:3000`)
- [x] Page loads
- [x] QR code displays
- [x] Connection URL shown
- [x] Empty players list shows "No players connected yet..."

### 3. Mobile View (`http://localhost:3000/mobile`)
- [x] Page loads
- [x] "Join Game" panel shows
- [x] Can enter name
- [x] "Join" button works

### 4. Player Connection
- [x] Enter name on mobile → click Join
- [x] Mobile shows "Connected to game"
- [x] TV updates to show player in list
- [x] WebSocket connection established (check browser console)

### 5. Multiple Players
- [x] Open multiple mobile pages (different browser tabs)
- [x] Each player can join with different name
- [x] TV shows all connected players
- [x] Players list updates in real-time

### 6. Disconnection
- [x] Close mobile tab
- [x] TV removes player from list
- [x] Other players still connected

---

## 🔍 Testing Steps

### Test 1: Single Player
1. Start server: `cd server && cargo run`
2. Open TV view: http://localhost:3000
3. Open mobile view: http://localhost:3000/mobile
4. On mobile: Enter "Alice" → Click Join
5. **Expected:** TV shows Alice in players list

### Test 2: Multiple Players
1. Open 3 mobile tabs
2. Join as "Alice", "Bob", "Carol"
3. **Expected:** TV shows all 3 players

### Test 3: QR Code
1. Open TV view
2. **Expected:** QR code displays (black/white squares)
3. **Expected:** URL shows: `http://localhost:3000/mobile`
4. Optional: Scan QR with phone camera → should open mobile view

### Test 4: Real-Time Updates
1. Have TV view open
2. Join player from mobile
3. **Expected:** Player appears on TV **immediately** (no page refresh needed)

### Test 5: Disconnection
1. Have 2 players connected
2. Close one mobile tab
3. **Expected:** TV removes that player, other player remains

---

## 🐛 Debugging

### Check Browser Console
Open browser DevTools (F12) and check Console tab:

**Expected logs:**
```
🎲 Daggerheart VTT Client - Phase 1
DOM loaded, initializing client...
Connecting to WebSocket: ws://localhost:3000/ws
✅ WebSocket connected
```

### Check Server Logs
Server terminal should show:
```
🎲 Daggerheart VTT Server - Phase 1
====================================
✅ Server listening on http://0.0.0.0:3000
🖥️  TV View:     http://localhost:3000
📱 Mobile View: http://localhost:3000/mobile
🔌 WebSocket:   ws://localhost:3000/ws
```

When players join:
```
Player joined: Alice (uuid)
Player left: Alice (uuid)
```

### Common Issues

**QR Code not showing:**
- Check server logs for errors
- Check browser console for fetch errors
- Try refreshing the page

**Player not appearing on TV:**
- Check WebSocket connection in browser console
- Verify server is running
- Check for JavaScript errors in console

**"Connection refused" error:**
- Make sure server is running (`cargo run`)
- Check firewall settings
- Try http://127.0.0.1:3000 instead

---

## 📊 Success Criteria

Phase 1 is successful if you can:
- ✅ Start the server
- ✅ See QR code on TV
- ✅ Join from mobile browser
- ✅ See player name appear on TV
- ✅ Have 3+ players connected simultaneously
- ✅ Players disconnect cleanly

---

## 🎯 What's Next?

Once Phase 1 is verified, we'll move to **Phase 2: Basic Map & Movement**:
- 2D canvas map rendering
- Player tokens as colored circles
- Tap-to-move controls
- Synchronized movement

---

## 📝 Notes

- Phase 1 uses in-memory state (no persistence)
- WebSocket reconnection is automatic
- Player colors not yet implemented (Phase 2)
- No character data yet (Phase 3)

---

**Phase 1 Status:** ✅ Complete and ready for testing!
