# WebSocket Protocol Specification

## Overview

All WebSocket messages use JSON format with a common structure:

```json
{
    "type": "message_type",
    "payload": { ... }
}
```

---

## Client → Server Messages

### 1. Player Join

**Type:** `player_join`

**Payload:**
```json
{
    "name": "string"
}
```

**Response:** `player_joined` broadcast to all clients

---

### 2. Player Move (Phase 2)

**Type:** `player_move`

**Payload:**
```json
{
    "x": 100,
    "y": 200
}
```

**Response:** `player_moved` broadcast to all clients

---

### 3. Roll Dice (Phase 3)

**Type:** `roll_dice`

**Payload:**
```json
{
    "roll_type": "duality",
    "modifier": 3
}
```

**Response:** `dice_rolled` broadcast to all clients

---

### 4. Create Character (Phase 3)

**Type:** `create_character`

**Payload:**
```json
{
    "name": "string",
    "class": "Warrior",
    "ancestry": "Orc",
    "attributes": [2, 1, 1, 0, 0, -1]
}
```

**Response:** `character_created`

---

## Server → Client Messages

### 1. Player Joined

**Type:** `player_joined`

**Payload:**
```json
{
    "player_id": "uuid",
    "name": "string",
    "position": { "x": 0, "y": 0 }
}
```

**Broadcast:** All clients

---

### 2. Player Left

**Type:** `player_left`

**Payload:**
```json
{
    "player_id": "uuid",
    "name": "string"
}
```

**Broadcast:** All clients

---

### 3. Players List

**Type:** `players_list`

**Payload:**
```json
{
    "players": [
        {
            "player_id": "uuid",
            "name": "string",
            "position": { "x": 0, "y": 0 },
            "connected": true
        }
    ]
}
```

**Sent:** On connection, to new client only

---

### 4. Player Moved (Phase 2)

**Type:** `player_moved`

**Payload:**
```json
{
    "player_id": "uuid",
    "position": { "x": 100, "y": 200 }
}
```

**Broadcast:** All clients except sender

---

### 5. Dice Rolled (Phase 3)

**Type:** `dice_rolled`

**Payload:**
```json
{
    "player_id": "uuid",
    "player_name": "string",
    "roll_type": "duality",
    "result": {
        "hope": 8,
        "fear": 5,
        "modifier": 3,
        "total": 16,
        "controlling": "Hope",
        "is_critical": false
    }
}
```

**Broadcast:** All clients

---

### 6. Game State (Phase 4)

**Type:** `game_state`

**Payload:**
```json
{
    "session_id": "uuid",
    "hope_pool": 5,
    "fear_pool": 2,
    "players": [...],
    "npcs": [...]
}
```

**Sent:** On connection, on request, after save/load

---

### 7. Error

**Type:** `error`

**Payload:**
```json
{
    "message": "string",
    "code": "error_code"
}
```

**Sent:** To specific client only

---

## Error Codes

| Code | Description |
|------|-------------|
| `invalid_message` | Malformed JSON or unknown type |
| `player_not_found` | Player ID doesn't exist |
| `invalid_action` | Action not allowed in current state |
| `name_taken` | Player name already in use |
| `session_full` | Max players reached |

---

## Message Flow Examples

### Player Joining

```
Mobile Client                Server                    TV Client
     │                          │                          │
     │─────player_join──────────▶│                          │
     │   {"name": "Alice"}       │                          │
     │                          │                          │
     │◀────player_joined─────────│                          │
     │   {"player_id": "123"}    │                          │
     │                          │                          │
     │                          │────player_joined─────────▶│
     │                          │   {"player_id": "123"}    │
     │                          │                          │
```

### Dice Rolling

```
Mobile Client                Server                    All Clients
     │                          │                          │
     │─────roll_dice────────────▶│                          │
     │   {"roll_type": "duality"}│                          │
     │                          │                          │
     │                          │  [Call daggerheart-      │
     │                          │   engine to roll]         │
     │                          │                          │
     │◀────dice_rolled───────────│────dice_rolled──────────▶│
     │   {"hope": 8, "fear": 5}  │   {"hope": 8, "fear": 5} │
     │                          │                          │
```

---

## Phase-Specific Messages

### Phase 1: Foundation & Connection
- `player_join`
- `player_joined`
- `player_left`
- `players_list`
- `error`

### Phase 2: Basic Map & Movement
- All Phase 1 messages
- `player_move`
- `player_moved`

### Phase 3: Daggerheart Integration
- All Phase 1-2 messages
- `roll_dice`
- `dice_rolled`
- `create_character`
- `character_created`
- `update_hp`
- `update_stress`

### Phase 4: Save/Load & GM Controls
- All Phase 1-3 messages
- `save_game`
- `load_game`
- `game_state`
- `add_npc`
- `remove_npc`

---

## Data Types

### Position
```json
{
    "x": number,
    "y": number
}
```

### Player
```json
{
    "player_id": "uuid",
    "name": "string",
    "position": Position,
    "connected": boolean,
    "character": Character | null
}
```

### Character (Phase 3)
```json
{
    "name": "string",
    "class": "Warrior" | "Bard" | ...,
    "ancestry": "Orc" | "Human" | ...,
    "attributes": {
        "agility": number,
        "strength": number,
        "finesse": number,
        "instinct": number,
        "presence": number,
        "knowledge": number
    },
    "hp": number,
    "max_hp": number,
    "stress": number,
    "evasion": number
}
```

### DualityRollResult (Phase 3)
```json
{
    "hope": number (1-12),
    "fear": number (1-12),
    "modifier": number,
    "advantage_die": number | null,
    "total": number,
    "controlling": "Hope" | "Fear" | "Tied",
    "is_critical": boolean
}
```

---

## Rate Limiting (Future)

To prevent abuse:
- Max 10 messages per second per client
- Max 100 dice rolls per minute per player
- Disconnect after 3 invalid messages

---

## Versioning

Protocol version is included in handshake:

```json
{
    "type": "handshake",
    "payload": {
        "protocol_version": "1.0",
        "client_type": "mobile" | "desktop" | "gm"
    }
}
```

Server responds:
```json
{
    "type": "handshake_ack",
    "payload": {
        "server_version": "0.1.0",
        "protocol_version": "1.0"
    }
}
```

---

## Testing

### Message Validation
All messages must:
- Be valid JSON
- Have `type` field
- Have `payload` field (can be empty object)
- Match schema for their type

### Example Test Cases
```rust
#[test]
fn test_player_join_valid() {
    let msg = json!({
        "type": "player_join",
        "payload": { "name": "Alice" }
    });
    assert!(validate_message(&msg).is_ok());
}

#[test]
fn test_player_join_invalid_name() {
    let msg = json!({
        "type": "player_join",
        "payload": { "name": "" }
    });
    assert!(validate_message(&msg).is_err());
}
```

---

**Version:** 1.0  
**Last Updated:** 2026-02-21  
**Status:** Phase 1 specification
