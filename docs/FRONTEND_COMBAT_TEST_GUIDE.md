# Frontend Combat UI - Testing Guide

**Date:** 2026-02-24 14:01 JST  
**Status:** ✅ READY TO TEST  
**Server:** Running on 192.168.1.119:3000

---

## 🎯 What We Built

### GM Adversary Panel
- **Template dropdown** with 7 built-in enemies
- **Click-to-spawn** workflow
- **Custom adversary creator**
- **Active adversaries list** with HP/Stress tracking
- **Combat controls** (start/end, token management)

### Canvas Rendering
- **Adversary tokens** (red circles with 💀)
- **HP bars** below each adversary
- **Name labels** above tokens
- **Visual effects** (glow, color changes)

### Combat System UI
- **Start/End Combat** buttons
- **Action Tracker** display (PC/Adversary tokens)
- **Token management** (add PC/Adversary tokens)

---

## 🧪 Testing Instructions

### Test 1: Spawn Adversary from Template

**Steps:**
1. Open GM view: http://192.168.1.119:3000/gm
2. In left sidebar, find "👹 Adversaries" panel
3. Template dropdown should show "Goblin (HP: 3, Evasion: 10)"
4. Click **"➕ Click Map to Spawn"** button
   - Button should turn red and say "❌ Cancel Spawn"
5. Click anywhere on the map canvas
6. **Expected Results:**
   - ✅ Red circle with 💀 appears on map at click position
   - ✅ "Goblin #1" appears in Active Adversaries list
   - ✅ Adversary shows: HP: 3/3, Stress: 0/3, Evasion: 10, Armor: 1
   - ✅ Event log shows "Goblin #1 spawned"
   - ✅ Button returns to "➕ Click Map to Spawn"

---

### Test 2: Spawn Multiple Adversaries

**Steps:**
1. Select "Bandit (HP: 4, Evasion: 11)" from dropdown
2. Click "Click Map to Spawn"
3. Click map at different location
4. Repeat with "Wolf (HP: 3, Evasion: 12)"
5. **Expected Results:**
   - ✅ 3 adversary tokens on map (Goblin #1, Bandit #1, Wolf #1)
   - ✅ All 3 in Active Adversaries list
   - ✅ Each with correct stats
   - ✅ HP bars show 100% (green)

---

### Test 3: Spawn Custom Adversary

**Steps:**
1. Select "Custom..." from dropdown
2. **Expected:** Custom adversary panel appears
3. Fill in custom stats:
   - Name: "Fire Elemental"
   - HP: 15
   - Evasion: 14
   - Armor: 0
   - Attack Mod: +4
   - Damage: "2d6+3"
4. Click "Click Map to Spawn"
5. Click map
6. **Expected Results:**
   - ✅ "Fire Elemental" spawned on map
   - ✅ Shows HP: 15/15, Evasion: 14, Armor: 0
   - ✅ Event log shows "Fire Elemental spawned (custom)"

---

### Test 4: Remove Adversary

**Steps:**
1. In Active Adversaries list, find "Goblin #1"
2. Click **🗑️** button next to it
3. **Expected:** Confirmation dialog appears
4. Click "OK"
5. **Expected Results:**
   - ✅ Goblin #1 removed from list
   - ✅ Goblin token removed from map
   - ✅ Event log shows "Goblin #1 removed"

---

### Test 5: Start Combat

**Steps:**
1. Ensure at least 2 adversaries are spawned
2. Scroll to "⚔️ Combat" panel
3. Click **"▶️ Start Combat"** button
4. **Expected Results:**
   - ✅ "Start Combat" button hides
   - ✅ "🛑 End Combat" button appears (red)
   - ✅ Combat controls panel appears
   - ✅ Shows "PC Tokens: 3" and "Adversary: 3"
   - ✅ Event log shows "Combat started"
   - ✅ Console shows: `⚔️ Combat started! Encounter: <id>`

---

### Test 6: Add Tracker Tokens

**Steps:**
1. With combat active, find "Action Tracker" section
2. Click **"+ PC"** button
3. **Expected:** PC Tokens count increases to 4
4. Click **"+ Adversary"** button
5. **Expected:** Adversary count increases to 4
6. **Check console:**
   - ✅ Shows: `🎲 Tracker updated: PC 4, Adversary 4, Next: <type>`

---

### Test 7: End Combat

**Steps:**
1. Click **"🛑 End Combat"** button
2. **Expected:** Confirmation dialog appears
3. Click "OK"
4. **Expected Results:**
   - ✅ Combat controls panel hides
   - ✅ "End Combat" button hides
   - ✅ "Start Combat" button reappears
   - ✅ Event log shows "Combat ended: manual"
   - ✅ Console shows: `✅ Combat ended: manual`

---

### Test 8: Multi-Client Testing (Advanced)

**Prerequisites:** Need 2 browser windows/devices

**Steps:**
1. **Browser A:** Open GM view (192.168.1.119:3000/gm)
2. **Browser B:** Open TV view (192.168.1.119:3000)
3. **In GM view:** Spawn a Goblin
4. **Expected in TV view:**
   - ✅ Goblin appears on map
   - ✅ Event log shows "Goblin #1 spawned"
5. **In GM view:** Start combat
6. **Expected in TV view:**
   - ✅ Event log shows "Combat started"
7. **In GM view:** Remove Goblin
8. **Expected in TV view:**
   - ✅ Goblin removed from map
   - ✅ Event log shows "Goblin #1 removed"

---

## 🎨 Visual Checks

### Adversary Tokens on Canvas
- ✅ **Red circle** with white border
- ✅ **💀 skull emoji** centered
- ✅ **Name label** above token
- ✅ **HP bar** below token (when HP data available)
- ✅ **Red glow** effect
- ✅ **Taken-out** adversaries appear gray

### Adversary List Items
- ✅ **Left border** in fear-color (red/purple)
- ✅ **Status icon:** 🗡️ (active) or 💀 (taken out)
- ✅ **Stats grid:** HP, Stress, Evasion, Armor
- ✅ **HP progress bar** (green → yellow → red)
- ✅ **Remove button** (🗑️) on right

### Combat Controls
- ✅ **Start/End buttons** toggle visibility
- ✅ **Token counts** update in real-time
- ✅ **Add buttons** styled consistently
- ✅ **Layout** fits in sidebar without scrolling

---

## 🐛 Known Limitations

### Not Yet Implemented
- ⏭️ Attack rolls (coming in Day 2)
- ⏭️ Damage application UI
- ⏭️ Turn indicator (who acts next)
- ⏭️ Mobile combat actions
- ⏭️ Visual tracker display (token circles)

### Current Behavior
- **Reset Tracker** button logs "not yet implemented"
- **HP bars** only show after HP update (initially null)
- **Canvas click** spawns even if map has background image later

---

## 📊 Success Criteria

### Must Pass
- [x] GM can spawn adversaries from templates
- [x] GM can spawn custom adversaries
- [x] Adversaries appear on map
- [x] Adversaries appear in list
- [x] GM can remove adversaries
- [x] GM can start combat
- [x] GM can end combat
- [x] GM can add tokens to tracker
- [x] All changes broadcast to TV view
- [x] Event log tracks all actions

### Nice to Have
- [ ] HP bars show immediately (currently wait for first update)
- [ ] Visual tracker tokens display
- [ ] Hover effects on adversaries
- [ ] Drag to reposition adversaries

---

## 🔍 Browser Console Checks

### Expected Console Messages

**On adversary spawn:**
```
🎯 Spawning goblin at (123.4, 567.8)
👹 Adversary spawned: Goblin #1 (<uuid>)
```

**On combat start:**
```
▶️ Starting combat...
⚔️ Combat started! Encounter: <uuid>
```

**On tracker update:**
```
🎲 Tracker updated: PC 3, Adversary 3, Next: pc
```

**On combat end:**
```
🛑 Ending combat...
✅ Combat ended: manual
```

---

## 🚨 Troubleshooting

### Issue: Adversary doesn't spawn
**Check:**
1. Server running? (`ss -tlnp | grep 3000`)
2. Browser console for errors?
3. WebSocket connected? (check console)
4. Refresh page (Ctrl+Shift+R)

### Issue: Button stays red after spawn
**Fix:**
1. Click map to complete spawn
2. Or click button again to cancel

### Issue: Combat controls don't appear
**Check:**
1. Did combat actually start? (check event log)
2. Console errors?
3. Try refreshing (Ctrl+Shift+R)

### Issue: Changes not visible in TV view
**Check:**
1. TV view connected? (check players list in GM view)
2. WebSocket active?
3. Refresh TV view

---

## 📝 Testing Checklist

Copy this checklist when testing:

```
BASIC FUNCTIONALITY:
[ ] Spawn Goblin from template
[ ] Spawn Bandit from template
[ ] Spawn custom Fire Elemental
[ ] Remove Goblin
[ ] Start combat
[ ] Add PC token
[ ] Add Adversary token
[ ] End combat

MULTI-CLIENT:
[ ] Spawn visible in TV view
[ ] Remove visible in TV view
[ ] Combat start visible in TV view
[ ] Event log synced

VISUAL:
[ ] Adversary tokens render correctly
[ ] HP bars visible (after update)
[ ] List items styled correctly
[ ] Buttons toggle properly

EDGE CASES:
[ ] Spawn 10+ adversaries (performance)
[ ] Cancel spawn mode
[ ] Remove all adversaries
[ ] Start combat with no adversaries
```

---

## ✅ When All Tests Pass

**You're ready for Day 2:**
- Mobile combat UI (attack buttons)
- Attack roll flow
- Damage application
- Turn management
- TV combat display enhancements

---

**Test Environment:**
- Server: http://192.168.1.119:3000
- GM View: /gm
- TV View: /
- Mobile View: /mobile

**Browser:** Any modern browser (Chrome, Firefox, Safari)  
**Hard Refresh:** Ctrl+Shift+R (Cmd+Shift+R on Mac)  
**Cache Version:** v=5

---

**Ready to test!** 🎲⚔️
