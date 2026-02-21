# Phase 2 Testing Guide: Map & Movement

## ✅ Implementation Complete

Phase 2 adds a 2D map with real-time player movement!

---

## 🚀 Running the Demo

```bash
cd /home/jake/.openclaw/workspace/daggerheart-vtt
./demo.sh phase2
```

---

## 📋 What to Test

### 1. Map Rendering (TV)
- [x] Canvas displays on TV (800x600 pixels)
- [x] Grid background visible (subtle 50px squares)
- [x] Dark theme background

### 2. Player Tokens
- [x] Players appear as colored circles
- [x] Each player has unique color (Blue, Green, Orange, Red, etc.)
- [x] Name labels above circles (white text with shadow)
- [x] Glowing effect around tokens

### 3. Spawn Positions
- [x] Players spawn at random locations
- [x] No overlapping spawns
- [x] Players visible on map immediately

### 4. Mobile Controls
- [x] Mini canvas shows on mobile after joining
- [x] "Tap map to move your token" instructions
- [x] Tap anywhere on canvas → your token moves
- [x] Touch controls work (mobile tap/touch)

### 5. Movement Sync
- [x] Movement appears on TV instantly
- [x] Smooth animation (500ms transition)
- [x] Other players see your movement
- [x] Multiple players can move simultaneously

### 6. Color Assignment
- [x] Players list shows color dots
- [x] Mobile view shows your color next to name
- [x] TV sidebar shows player colors

---

## 🧪 Testing Steps

### Test 1: Single Player Movement
1. Start server: `./demo.sh phase2`
2. Open TV view: http://localhost:3000
3. Join from mobile
4. **Expected:** Token appears on map with your name
5. Tap canvas on mobile → token moves smoothly
6. **Expected:** Movement animates to new position

### Test 2: Multiple Players
1. Join 3-4 players from different devices/tabs
2. **Expected:** Each gets unique color
3. **Expected:** All tokens visible on map
4. Have each player tap to move
5. **Expected:** All movements sync across all clients

### Test 3: Color Uniqueness
1. Join 5 players
2. **Expected:** All different colors
3. Check TV sidebar
4. **Expected:** Color dots match map tokens

### Test 4: Smooth Animation
1. Join from mobile
2. Tap far away from current position
3. **Expected:** Token glides smoothly (not teleport)
4. **Expected:** Animation takes ~500ms

### Test 5: Real-Time Sync
1. TV view + 2 mobile devices
2. Player A moves
3. **Expected:** TV updates immediately
4. **Expected:** Player B sees movement on their mini-map

### Test 6: Mobile Mini-Map
1. Join from phone
2. **Expected:** Mini canvas shows after joining
3. **Expected:** Can see all players on mini-map
4. **Expected:** Your token highlighted/identifiable
5. Tap mini-map → **Expected:** You move

---

## 🐛 Debugging

### Check Browser Console
**Expected logs:**
```
🎲 Daggerheart VTT Client - Phase 2
MapCanvas initialized: 800 x 600
Player joined: Alice (...) at (234.5, 456.7)
Added player to canvas: Alice { x: 234.5, y: 456.7 }
Tap to move: { x: 400, y: 300 }
Player ... moved to (400, 300)
```

### Check Server Logs
```
Player joined: Alice (uuid) at Position { x: 234.5, y: 456.7 }
Player uuid moved to (400.0, 300.0)
```

### Common Issues

**Canvas not showing:**
- Check browser console for errors
- Verify canvas.js loaded
- Check CSS (canvas might be hidden)

**Movement not working:**
- Verify WebSocket connected
- Check you're actually joined (currentPlayerId set)
- Look for JavaScript errors in console

**No animation (teleporting):**
- This might happen if render loop isn't running
- Check browser console for errors
- Try refreshing page

**Colors not unique:**
- Should cycle through 8 colors
- 9th player will repeat first color
- Check game state color_index

---

## 📊 Success Criteria

Phase 2 is complete when:
- ✅ TV shows 2D map with grid
- ✅ Players appear as colored circles + names
- ✅ Each player has unique color
- ✅ Mobile tap moves your token
- ✅ Movement syncs to TV instantly
- ✅ Movement is smooth (animated, not teleport)
- ✅ 4+ players can move simultaneously
- ✅ Mini-map visible on mobile

---

## 🎨 Visual Details

### Map
- Size: 800x600 pixels
- Grid: 50x50 pixels
- Background: #1a1a1a (dark)
- Grid lines: #2a2a2a (subtle)

### Tokens
- Radius: 20 pixels
- Border: 2px white
- Glow: 15px shadow in player color
- Name: 14px bold, white with shadow

### Colors
1. Blue (#3b82f6)
2. Green (#10b981)
3. Orange (#f59e0b)
4. Red (#ef4444)
5. Purple (#8b5cf6)
6. Pink (#ec4899)
7. Teal (#14b8a6)
8. Dark Orange (#f97316)

---

## 🎯 What's Next?

Once Phase 2 is verified, we move to **Phase 3: Daggerheart Integration**:
- Character creation (class, ancestry, attributes)
- Dice rolling UI
- Character sheets
- HP/Stress tracking
- Hope/Fear pools
- Roll animations on TV

---

## 📝 Notes

- Map state is in-memory (resets on server restart)
- Session persistence from Phase 1 still works
- Movement uses free-form coordinates (not grid-snapped)
- Animation uses cubic ease-out for smooth feel
- WebSocket broadcasts to all clients for sync

---

**Phase 2 Status:** ✅ Complete and ready for testing!
