// Daggerheart VTT - GM View
// Phase 5A: Character-Centric Architecture

console.log('🎮 GM View Initialized');

let ws = null;
let mapCanvas = null;
let characters = [];
let saves = [];
let connectionCount = 0;

document.addEventListener('DOMContentLoaded', () => {
    console.log('GM DOM loaded');
    
    // Initialize canvas
    mapCanvas = new MapCanvas('gm-canvas');
    
    // Connect to WebSocket
    ws = new WebSocketClient(handleServerMessage);
    ws.connect();
    
    // Setup event listeners
    setupEventListeners();
    
    // Load saves list
    loadSaves();
});

function setupEventListeners() {
    // Save button
    document.getElementById('save-btn').addEventListener('click', saveGame);
    
    // Refresh saves button
    document.getElementById('refresh-saves-btn').addEventListener('click', loadSaves);
    
    // Clear all stress
    document.getElementById('clear-stress-all').addEventListener('click', clearAllStress);
    
    // Refresh clients
    document.getElementById('refresh-clients').addEventListener('click', refreshClients);
    
    // Request roll button
    document.getElementById('request-roll-btn').addEventListener('click', requestRoll);
}

async function saveGame() {
    try {
        const response = await fetch('/api/save', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' }
        });
        
        const result = await response.json();
        
        if (result.success) {
            console.log('Game saved:', result.path);
            alert(`✅ Game saved to:\n${result.path}`);
            loadSaves(); // Refresh saves list
        } else {
            console.error('Save failed:', result.error);
            alert(`❌ Save failed: ${result.error}`);
        }
    } catch (error) {
        console.error('Save error:', error);
        alert(`❌ Error: ${error.message}`);
    }
}

async function loadSaves() {
    try {
        const response = await fetch('/api/saves');
        const result = await response.json();
        
        if (result.success) {
            saves = result.saves;
            renderSavesList();
        } else {
            console.error('Failed to load saves:', result.error);
        }
    } catch (error) {
        console.error('Load saves error:', error);
    }
}

function renderSavesList() {
    const container = document.getElementById('saves-list');
    
    if (saves.length === 0) {
        container.innerHTML = '<p class="empty-state">No saves yet</p>';
        return;
    }
    
    container.innerHTML = saves.map(save => {
        const date = new Date(save.timestamp);
        const timeStr = date.toLocaleString();
        
        return `
            <div class="save-item" data-path="${save.path}">
                <strong>${save.name}</strong>
                <br>
                <small>${timeStr}</small>
            </div>
        `;
    }).join('');
    
    // Add click handlers
    document.querySelectorAll('.save-item').forEach(item => {
        item.addEventListener('click', () => {
            const path = item.dataset.path;
            loadGameSession(path);
        });
    });
}

async function loadGameSession(path) {
    if (!confirm(`Load this session? Current game will be replaced.\n\n${path}`)) {
        return;
    }
    
    try {
        const response = await fetch('/api/load', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ path })
        });
        
        const result = await response.json();
        
        if (result.success) {
            console.log('Session loaded:', result.session);
            alert('✅ Session loaded! All clients will refresh.');
            
            // Reload GM view
            setTimeout(() => {
                window.location.reload();
            }, 2000);
        } else {
            console.error('Load failed:', result.error);
            alert(`❌ Load failed: ${result.error}`);
        }
    } catch (error) {
        console.error('Load error:', error);
        alert(`❌ Error: ${error.message}`);
    }
}

function clearAllStress() {
    if (!confirm('Clear stress for all characters?')) {
        return;
    }
    
    // TODO: Implement via WebSocket message
    console.log('Clear all stress - not implemented yet');
    alert('⚠️ Feature not implemented yet');
}

function refreshClients() {
    if (!confirm('Refresh all connected clients?')) {
        return;
    }
    
    // Send error message to trigger client refresh
    // (This is a hack, but works for now)
    alert('⚠️ Feature not implemented - ask clients to refresh manually');
}

function handleServerMessage(message) {
    const { type, payload } = message;
    
    switch (type) {
        case 'connected':
            handleConnected(payload);
            break;
        case 'characters_list':
            handleCharactersList(payload);
            break;
        case 'character_spawned':
            handleCharacterSpawned(payload);
            break;
        case 'character_moved':
            handleCharacterMoved(payload);
            break;
        case 'character_created':
        case 'character_updated':
            // Character was updated, will get new list
            break;
        case 'roll_request_status':
            updateRollStatus(payload);
            break;
        case 'detailed_roll_result':
            console.log('Roll result:', payload);
            // Results are shown on TV view
            break;
        default:
            console.log('GM received:', type, payload);
    }
}

function handleConnected(payload) {
    const { connection_id } = payload;
    console.log('✅ GM Connected with ID:', connection_id);
}

function handleCharactersList(payload) {
    characters = payload.characters;
    console.log('Characters list:', characters);
    
    // Update canvas
    if (mapCanvas) {
        mapCanvas.clearPlayers();
        characters.forEach(char => {
            mapCanvas.addPlayer(char.id, char.name, char.position, char.color);
        });
    }
    
    // Update sidebar
    renderCharactersList();
    updateSessionInfo();
    
    // Update roll target dropdown
    updateTargetDropdown(characters);
}

function handleCharacterSpawned(payload) {
    const { character_id, name, position, color, is_npc } = payload;
    console.log(`Character spawned: ${name} (${is_npc ? 'NPC' : 'PC'})`);
    
    // Add to characters list
    characters.push({
        id: character_id,
        name,
        position,
        color,
        is_npc,
    });
    
    // Add to canvas
    if (mapCanvas) {
        mapCanvas.addPlayer(character_id, name, position, color);
    }
    
    // Update sidebar
    renderCharactersList();
    updateSessionInfo();
}

function handleCharacterMoved(payload) {
    const { character_id, position } = payload;
    
    // Update in characters list
    const char = characters.find(c => c.id === character_id);
    if (char) {
        char.position = position;
    }
    
    // Update canvas
    if (mapCanvas) {
        mapCanvas.updatePlayerPosition(character_id, position);
    }
}

function renderCharactersList() {
    const container = document.getElementById('players-list-gm');
    
    if (characters.length === 0) {
        container.innerHTML = '<p class="empty-state">No characters in game</p>';
        return;
    }
    
    container.innerHTML = characters.map(char => {
        const typeLabel = char.is_npc ? 'NPC' : 'PC';
        const typeClass = char.is_npc ? 'char-npc' : 'char-pc';
        const controlInfo = char.controlled_by_me ? '(You)' : 
                           char.controlled_by_other ? '(Controlled)' : 
                           '(Available)';
        
        return `
            <div class="player-item-gm ${typeClass}">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <strong>
                        <span style="display:inline-block;width:12px;height:12px;border-radius:50%;background:${char.color};margin-right:8px;"></span>
                        ${char.name}
                    </strong>
                    <span style="font-size: 0.75rem; color: var(--text-dim);">${typeLabel}</span>
                </div>
                <div class="player-stats">
                    <div class="stat">${char.class} • ${char.ancestry}</div>
                    <div class="stat">${controlInfo}</div>
                </div>
            </div>
        `;
    }).join('');
}

function updateSessionInfo() {
    const pcCount = characters.filter(c => !c.is_npc).length;
    const npcCount = characters.filter(c => c.is_npc).length;
    
    document.getElementById('player-count').textContent = connectionCount;
    document.getElementById('character-count').textContent = `${pcCount} PCs, ${npcCount} NPCs`;
}

async function fetchGameState() {
    try {
        const response = await fetch('/api/game-state');
        const data = await response.json();
        
        connectionCount = data.connection_count || 0;
        updateSessionInfo();
    } catch (error) {
        console.error('Failed to fetch game state:', error);
    }
}

// Fetch game state periodically
setInterval(fetchGameState, 5000);
fetchGameState(); // Initial fetch

// Roll Request Functions
function requestRoll() {
    const target = document.getElementById('roll-target').value;
    const attribute = document.getElementById('roll-attribute').value || null;
    const difficulty = parseInt(document.getElementById('roll-difficulty').value);
    const context = document.getElementById('roll-context').value || 'GM requested roll';
    const hasAdvantage = document.getElementById('roll-advantage').checked;
    
    console.log('Requesting roll:', { target, attribute, difficulty, context, hasAdvantage });
    
    // Determine target type and IDs
    let targetType = 'all';
    let targetIds = [];
    
    if (target !== 'all') {
        targetType = 'specific';
        targetIds = [target];
    }
    
    // Send roll request
    ws.send('request_roll', {
        target_type: targetType,
        target_character_ids: targetIds,
        roll_type: 'action',
        attribute: attribute,
        difficulty: difficulty,
        context: context,
        narrative_stakes: null,
        situational_modifier: 0,
        has_advantage: hasAdvantage,
        is_combat: false,
    });
    
    // Show status panel
    const statusPanel = document.getElementById('roll-status-panel');
    if (statusPanel) {
        statusPanel.style.display = 'block';
    }
    
    // Clear form
    document.getElementById('roll-context').value = '';
}

function updateRollStatus(status) {
    const completedList = document.getElementById('completed-list');
    const pendingList = document.getElementById('pending-list');
    
    if (completedList) {
        completedList.textContent = status.completed_characters.length > 0 
            ? status.completed_characters.join(', ') 
            : 'None';
    }
    
    if (pendingList) {
        pendingList.textContent = status.pending_characters.length > 0 
            ? status.pending_characters.join(', ') 
            : 'None';
    }
    
    // If all done, hide panel after a delay
    if (status.pending_characters.length === 0 && status.completed_characters.length > 0) {
        setTimeout(() => {
            const statusPanel = document.getElementById('roll-status-panel');
            if (statusPanel) {
                statusPanel.style.display = 'none';
            }
        }, 3000);
    }
}

// Populate target dropdown when characters update
function updateTargetDropdown(chars) {
    const dropdown = document.getElementById('roll-target');
    if (!dropdown) return;
    
    // Save current selection
    const currentValue = dropdown.value;
    
    // Clear and repopulate
    dropdown.innerHTML = '<option value="all">All Players</option>';
    
    chars.filter(c => !c.is_npc).forEach(char => {
        const option = document.createElement('option');
        option.value = char.id;
        option.textContent = char.name;
        dropdown.appendChild(option);
    });
    
    // Restore selection if still valid
    if (currentValue && Array.from(dropdown.options).some(opt => opt.value === currentValue)) {
        dropdown.value = currentValue;
    }
}
