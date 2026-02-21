// Daggerheart VTT - GM View
// Phase 4: Save/Load & GM Controls

console.log('🎮 GM View Initialized');

let ws = null;
let mapCanvas = null;
let players = [];
let saves = [];

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
    if (!confirm('Clear stress for all players?')) {
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
        case 'player_joined':
            handlePlayerJoined(payload);
            break;
        case 'player_left':
            handlePlayerLeft(payload);
            break;
        case 'players_list':
            handlePlayersList(payload);
            break;
        case 'player_moved':
            handlePlayerMoved(payload);
            break;
        case 'character_created':
        case 'character_updated':
        case 'player_name_updated':
            // Refresh player list
            requestPlayersList();
            break;
        default:
            console.log('GM received:', type, payload);
    }
}

function handlePlayerJoined(payload) {
    const { player_id, name, position, color } = payload;
    console.log(`Player joined: ${name}`);
    
    // Add to canvas
    if (mapCanvas) {
        mapCanvas.addPlayer(player_id, name, position, color);
    }
    
    // Request full player list to update sidebar
    requestPlayersList();
}

function handlePlayerLeft(payload) {
    const { player_id, name } = payload;
    console.log(`Player left: ${name}`);
    
    // Remove from canvas
    if (mapCanvas) {
        mapCanvas.removePlayer(player_id);
    }
    
    // Update player list
    requestPlayersList();
}

function handlePlayersList(payload) {
    players = payload.players;
    console.log('Players list:', players);
    
    // Update canvas
    if (mapCanvas) {
        mapCanvas.clearPlayers();
        players.forEach(player => {
            const displayName = player.character_name || player.name;
            mapCanvas.addPlayer(player.player_id, displayName, player.position, player.color);
        });
    }
    
    // Update sidebar
    renderPlayersList();
    updateSessionInfo();
}

function handlePlayerMoved(payload) {
    const { player_id, position } = payload;
    
    // Update canvas
    if (mapCanvas) {
        mapCanvas.updatePlayerPosition(player_id, position);
    }
}

function renderPlayersList() {
    const container = document.getElementById('players-list-gm');
    
    if (players.length === 0) {
        container.innerHTML = '<p class="empty-state">No players connected</p>';
        return;
    }
    
    container.innerHTML = players.map(player => {
        const displayName = player.character_name || player.name;
        const statusClass = player.connected ? 'status-online' : 'status-offline';
        const statusText = player.connected ? 'Online' : 'Offline';
        
        let characterInfo = '';
        if (player.has_character && player.character_name) {
            characterInfo = `
                <div class="player-stats">
                    <div class="stat">Char: ${player.character_name}</div>
                    <div class="stat">Status: Active</div>
                </div>
            `;
        } else {
            characterInfo = '<p style="font-size: 0.85rem; color: var(--text-dim); margin-top: 0.5rem;">No character yet</p>';
        }
        
        return `
            <div class="player-item-gm">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <strong>
                        <span class="status-indicator ${statusClass}"></span>
                        ${displayName}
                    </strong>
                    <span style="font-size: 0.75rem; color: var(--text-dim);">${statusText}</span>
                </div>
                ${characterInfo}
            </div>
        `;
    }).join('');
}

function updateSessionInfo() {
    const playerCount = players.length;
    const characterCount = players.filter(p => p.has_character).length;
    
    document.getElementById('player-count').textContent = playerCount;
    document.getElementById('character-count').textContent = characterCount;
}

function requestPlayersList() {
    // The server will send players_list when we join
    // For now, we rely on the automatic broadcast
}
