# Frontend Testing Guide

**Server URL:** `http://192.168.1.119:3000`  
**Status:** Ready for testing

---

## Quick Test Checklist

### 1. TV/Desktop View Test
**URL:** http://192.168.1.119:3000

**Expected:**
- [ ] Page loads with "🗡️❤️ Daggerheart VTT" header
- [ ] QR code displays in sidebar
- [ ] "Join the Game" section visible
- [ ] Empty game canvas visible
- [ ] WebSocket connects (check browser console)

**Console should show:**
```
🎲 Daggerheart VTT Client - Phase 5A
DOM loaded, initializing client...
Initializing desktop/TV view
Connecting to WebSocket: ws://192.168.1.119:3000/ws
✅ WebSocket connected
📤 Sending: {type: 'connect'}
📨 Received: {type: 'connected', connection_id: '...'}
```

---

### 2. Mobile View Test
**URL:** http://192.168.1.119:3000/mobile

**Expected:**
- [ ] Mobile-optimized layout loads
- [ ] "Enter your name" input field visible
- [ ] "Join Game" button visible
- [ ] WebSocket connects

**Test flow:**
1. Enter name: `TestPlayer`
2. Click "Join Game"
3. Character creation form should appear
4. Fill out character details:
   - Name: `Theron`
   - Class: `Warrior`
   - Ancestry: `Human`
   - Attributes: Adjust sliders
5. Click "Create Character"
6. Character sheet should appear
7. "Roll Duality" button should be visible

---

### 3. Character Creation Test

**In mobile view, after joining:**

**Expected form fields:**
- [ ] Name input
- [ ] Class dropdown (Bard, Guardian, Ranger, etc.)
- [ ] Ancestry dropdown (Human, Faerie, etc.)
- [ ] Attribute sliders (6 attributes)
- [ ] Create button

**Console should show:**
```
📤 Sending: {type: 'create_character', name: '...', class: '...', ...}
📨 Received: {type: 'character_created', character_id: '...'}
📨 Received: {type: 'character_selected', character_id: '...'}
```

---

### 4. Dice Rolling Test

**In mobile view, with character created:**

1. Click "Roll Duality" button
2. Check console for WebSocket messages
3. TV view should show roll result overlay

**Console should show:**
```
📤 Sending: {type: 'roll_duality', modifier: 0, with_advantage: false}
📨 Received: {type: 'roll_result', ...}
```

---

### 5. Multi-Client Test

**Requirements:** 2 browser windows/tabs

**Window 1 (TV):** http://192.168.1.119:3000  
**Window 2 (Mobile):** http://192.168.1.119:3000/mobile

**Test:**
1. In Window 2: Join game and create character
2. In Window 1: Character should appear in sidebar
3. In Window 2: Click somewhere on canvas (if movement implemented)
4. In Window 1: Character position should update

---

### 6. WebSocket Connection Test

**Open browser DevTools → Console**

**Expected messages:**
1. `✅ WebSocket connected`
2. `📤 Sending: {type: 'connect'}`
3. `📨 Received: {type: 'connected', connection_id: '...'}`
4. `📨 Received: {type: 'characters_list', characters: [...]}`

**Check Network tab:**
- Look for WebSocket connection to `/ws`
- Status should be `101 Switching Protocols` (success)
- Connection should stay open (not closed)

---

### 7. Reconnection Test

**Test browser refresh handling:**

1. Join game and create character
2. Note your character ID (check localStorage)
3. Refresh the page (F5)
4. Character should auto-reconnect

**Console should show:**
```
Found saved character, auto-reconnecting: <character-id>
```

---

### 8. Error Handling Test

**Test invalid inputs:**

1. Try to join without entering name → Should show alert
2. Try to create character with invalid data → Should show error message
3. Disconnect server → Should attempt reconnection

**Console should show reconnection attempts:**
```
WebSocket disconnected
Reconnecting... (1/5)
```

---

## Manual Testing Commands

### Check WebSocket from Command Line

```bash
# Install websocat (if available)
websocat ws://192.168.1.119:3000/ws

# Or use Node.js
node -e "
const WebSocket = require('ws');
const ws = new WebSocket('ws://192.168.1.119:3000/ws');
ws.on('open', () => {
  console.log('Connected');
  ws.send(JSON.stringify({ type: 'connect' }));
});
ws.on('message', (data) => console.log('Received:', data.toString()));
"
```

### Check HTTP Endpoints

```bash
# Get game state
curl -s http://192.168.1.119:3000/api/game-state | jq .

# Get QR code
curl -s 'http://192.168.1.119:3000/api/qr-code?url=http://test.com' | jq .

# List saves
curl -s http://192.168.1.119:3000/api/saves | jq .

# Check if server is responding
curl -I http://192.168.1.119:3000/
```

---

## Browser DevTools Debugging

### Console Logging

All WebSocket messages are logged:
- `📤 Sending:` - Messages sent to server
- `📨 Received:` - Messages received from server
- `✅` - Success indicators
- `❌` - Error indicators

### Network Tab

Check WebSocket frames:
1. Open DevTools → Network tab
2. Filter by "WS" (WebSocket)
3. Click on `/ws` connection
4. View "Messages" tab to see all frames

### Application Tab

Check localStorage:
- `dh_vtt_character_id` - Your character ID
- `dh_vtt_session_active` - Session status

---

## Common Issues & Solutions

### Issue: Page doesn't load
**Solution:** Check that server is running on port 3000
```bash
ss -tlnp | grep 3000
```

### Issue: WebSocket fails to connect
**Solution:** Check browser console for errors. Verify URL protocol (ws:// not wss://)

### Issue: Character doesn't appear
**Solution:** Check browser console for error messages. Verify character was created successfully.

### Issue: Dice rolls don't show
**Solution:** Check if roll overlay is implemented in current phase. Verify console logs.

---

## Expected Console Output (Full Flow)

```
🎲 Daggerheart VTT Client - Phase 5A
DOM loaded, initializing client...
Initializing mobile view
Connecting to WebSocket: ws://192.168.1.119:3000/ws
✅ WebSocket connected
📤 Sending: {type: 'connect'}
📨 Received: {type: 'connected', connection_id: 'abc123...'}
📨 Received: {type: 'characters_list', characters: []}
🖱️ Join button clicked!
📝 Player name entered: TestPlayer
📤 Sending: {type: 'create_character', name: 'Theron', class: 'Warrior', ...}
📨 Received: {type: 'character_created', character_id: 'def456...', character: {...}}
📨 Received: {type: 'character_selected', character_id: 'def456...', character: {...}}
Character created and selected!
```

---

## Success Criteria

Frontend is working if:
- ✅ Pages load without errors
- ✅ WebSocket connects successfully
- ✅ Character creation works
- ✅ Dice rolling sends messages
- ✅ Multi-client sync works
- ✅ Reconnection after refresh works

---

## Testing Tools

### Recommended Browsers
- ✅ Chrome/Chromium (best DevTools)
- ✅ Firefox (good WebSocket debugging)
- ✅ Safari (for iOS testing)

### Mobile Testing
- Use actual phone for best results
- Scan QR code from TV view
- Test touch controls
- Verify responsive layout

---

## Next Steps After Frontend Test

1. Document any bugs found
2. Test all Phase 5A features
3. Verify character persistence
4. Test edge cases (disconnects, etc.)
5. Prepare for Phase 2 development

---

**Ready to test!** Open http://192.168.1.119:3000 in your browser.
