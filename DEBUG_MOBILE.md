# Mobile Client Debugging Guide

## Current Issue
Mobile client doesn't respond when clicking "Join Game" button.

## Debug Steps

### Step 1: Start the Server
```bash
cd server
cargo run
```

You should see:
```
🎲 Daggerheart VTT Server - Phase 1
====================================
✅ Server listening on http://0.0.0.0:3000

📡 Network Access:
   Local IP:    http://192.168.x.x:3000
   Localhost:   http://localhost:3000

🖥️  TV View:     http://192.168.x.x:3000
📱 Mobile View: http://192.168.x.x:3000/mobile
```

### Step 2: Open Mobile View
Open: `http://localhost:3000/mobile` in your browser

### Step 3: Open Browser Console
- **Chrome/Edge**: F12 → Console tab
- **Firefox**: F12 → Console tab  
- **Safari**: Cmd+Option+C (enable Developer menu first)

### Step 4: Enter Name and Click Join

**Expected console output:**
```
🎲 Daggerheart VTT Client - Phase 5A
DOM loaded, initializing client...
Initializing mobile view
✅ Join button found, adding click handler

[After clicking Join button]
🖱️ Join button clicked!
📝 Player name entered: YourName
🔌 connectToGame() called with name: YourName
📡 Connecting to WebSocket...
✅ Stored pendingPlayerName: YourName
Connecting to WebSocket: ws://localhost:3000/ws
✅ WebSocket connected
📤 Sending: {type: 'connect', payload: {}}
📨 Received: {type: 'connected', payload: {connection_id: '...'}}
✅ Connected with ID: ...
📍 Current pathname: /mobile
📝 pendingPlayerName: YourName
🎯 Is mobile? true
🎯 Has pendingPlayerName? true
🎨 Showing character creation for: YourName
🎨 showCharacterCreation() called
📦 characterCreator: CharacterCreator {...}
📦 char-creation-container: <div>
✅ Character creator initialized
```

### Step 5: Report What You See

**What to check:**
1. ✅ Does "Join button clicked!" appear?
2. ✅ Does WebSocket connect successfully?
3. ✅ Does the server respond with 'connected' message?
4. ✅ Does pathname check pass? (Is mobile? true)
5. ✅ Does pendingPlayerName exist?
6. ✅ Does character creator initialize?

**If ANY of these fail, that's where the problem is!**

## Common Issues

### Issue 1: Join button doesn't respond
**Symptoms:** No "Join button clicked!" in console

**Possible causes:**
- JavaScript not loading
- Button ID mismatch
- Event listener not attached

**Fix:**
Check if scripts load:
```javascript
// In console:
console.log(typeof WebSocketClient); // should be "function"
console.log(typeof CharacterCreator); // should be "function"
```

### Issue 2: WebSocket doesn't connect
**Symptoms:** "WebSocket disconnected" or error

**Possible causes:**
- Server not running
- Port blocked
- Wrong URL

**Fix:**
- Verify server is running on port 3000
- Check firewall settings
- Try `http://127.0.0.1:3000/mobile` instead

### Issue 3: Server doesn't respond
**Symptoms:** WebSocket connects but no 'connected' message

**Possible causes:**
- Server crash
- Protocol mismatch
- Message handling error

**Fix:**
Check server logs for errors

### Issue 4: Character creator doesn't show
**Symptoms:** 'connected' received but no UI change

**Possible causes:**
- Pathname check fails
- pendingPlayerName cleared
- characterCreator not initialized
- Panel ID mismatch

**Fix:**
Check the debug logs for:
- "Is mobile?" value
- "Has pendingPlayerName?" value  
- "characterCreator:" value

## Manual Fallback Test

If the flow still doesn't work, try calling functions manually in console:

```javascript
// Test 1: Show character creation directly
showCharacterCreation();

// Test 2: Check characterCreator
console.log(characterCreator);

// Test 3: Initialize manually
const container = document.getElementById('char-creation-container');
characterCreator.init(container);

// Test 4: Check WebSocket
console.log(ws);
console.log(currentConnectionId);
```

## Next Steps

Once we identify which step fails, we can fix the specific issue!

**Please run through these steps and paste the console output.**
