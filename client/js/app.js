// Daggerheart VTT Client
// Phase 1: Foundation & Connection

console.log('🎲 Daggerheart VTT Client - Phase 1');

let ws = null;
let currentPlayerId = null;
let currentPlayerName = null;

// LocalStorage keys
const STORAGE_KEYS = {
    PLAYER_NAME: 'dh_vtt_player_name',
    SESSION_ACTIVE: 'dh_vtt_session_active'
};

document.addEventListener('DOMContentLoaded', () => {
    console.log('DOM loaded, initializing client...');
    
    // Detect if we're on mobile or desktop
    const isMobile = window.location.pathname.includes('mobile');
    
    if (isMobile) {
        initMobileView();
    } else {
        initDesktopView();
    }
});

function initDesktopView() {
    console.log('Initializing desktop/TV view');
    
    // Load QR code
    loadQRCode();
    
    // Connect to WebSocket
    ws = new WebSocketClient(handleServerMessage);
    ws.connect();
}

function initMobileView() {
    console.log('Initializing mobile view');
    
    const joinButton = document.getElementById('join-button');
    const playerNameInput = document.getElementById('player-name');
    const leaveButton = document.getElementById('leave-button');
    
    // Check if we have a saved session
    const savedName = localStorage.getItem(STORAGE_KEYS.PLAYER_NAME);
    const sessionActive = localStorage.getItem(STORAGE_KEYS.SESSION_ACTIVE) === 'true';
    
    if (savedName && sessionActive) {
        console.log('Found saved session, auto-rejoining as:', savedName);
        currentPlayerName = savedName;
        
        // Auto-rejoin
        setTimeout(() => {
            autoRejoin(savedName);
        }, 500);
    }
    
    if (joinButton) {
        joinButton.addEventListener('click', () => {
            const name = playerNameInput.value.trim();
            if (name) {
                joinGame(name);
            } else {
                alert('Please enter your name');
            }
        });
        
        // Allow Enter key to join
        playerNameInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                joinButton.click();
            }
        });
    }
    
    if (leaveButton) {
        leaveButton.addEventListener('click', () => {
            leaveGame();
        });
    }
}

function autoRejoin(playerName) {
    console.log('Auto-rejoining as:', playerName);
    
    // Update UI immediately
    document.querySelector('.join-panel').style.display = 'none';
    document.getElementById('player-info').style.display = 'block';
    document.getElementById('player-name-display').textContent = playerName;
    
    // Connect to WebSocket
    ws = new WebSocketClient(handleServerMessage);
    ws.connect();
    
    // Send join message
    setTimeout(() => {
        ws.send('player_join', { name: playerName });
    }, 500);
}

function joinGame(playerName) {
    console.log('Joining game as:', playerName);
    
    currentPlayerName = playerName;
    
    // Save to localStorage
    localStorage.setItem(STORAGE_KEYS.PLAYER_NAME, playerName);
    localStorage.setItem(STORAGE_KEYS.SESSION_ACTIVE, 'true');
    
    // Connect to WebSocket
    ws = new WebSocketClient(handleServerMessage);
    ws.connect();
    
    // Wait a bit for connection, then send join message
    setTimeout(() => {
        ws.send('player_join', { name: playerName });
        
        // Update UI
        document.querySelector('.join-panel').style.display = 'none';
        document.getElementById('player-info').style.display = 'block';
        document.getElementById('player-name-display').textContent = playerName;
    }, 500);
}

function leaveGame() {
    console.log('Leaving game');
    
    // Clear localStorage
    localStorage.removeItem(STORAGE_KEYS.PLAYER_NAME);
    localStorage.setItem(STORAGE_KEYS.SESSION_ACTIVE, 'false');
    
    // Disconnect WebSocket
    if (ws) {
        ws.disconnect();
    }
    
    // Reset UI
    document.querySelector('.join-panel').style.display = 'block';
    document.getElementById('player-info').style.display = 'none';
    document.getElementById('player-name').value = '';
    
    currentPlayerName = null;
    currentPlayerId = null;
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
        case 'error':
            handleError(payload);
            break;
        default:
            console.warn('Unknown message type:', type);
    }
}

function handlePlayerJoined(payload) {
    const { player_id, name } = payload;
    console.log(`Player joined: ${name} (${player_id})`);
    
    // Track our own player ID
    if (name === currentPlayerName) {
        currentPlayerId = player_id;
        console.log('Set our player ID:', currentPlayerId);
    }
    
    // Add to players list if we're on desktop
    if (!window.location.pathname.includes('mobile')) {
        addPlayerToList(player_id, name);
    }
}

function handlePlayerLeft(payload) {
    const { player_id, name } = payload;
    console.log(`Player left: ${name} (${player_id})`);
    
    // Remove from players list if we're on desktop
    if (!window.location.pathname.includes('mobile')) {
        removePlayerFromList(player_id);
    }
}

function handlePlayersList(payload) {
    const { players } = payload;
    console.log('Players list:', players);
    
    // Update players list on desktop
    if (!window.location.pathname.includes('mobile')) {
        updatePlayersList(players);
    }
}

function handleError(payload) {
    const { message } = payload;
    console.error('Server error:', message);
    alert(`Error: ${message}`);
}

function addPlayerToList(playerId, name) {
    const playersList = document.getElementById('players-list');
    if (!playersList) return;
    
    // Remove empty state if present
    const emptyState = playersList.querySelector('.empty-state');
    if (emptyState) {
        emptyState.remove();
    }
    
    // Create player card
    const card = document.createElement('div');
    card.className = 'player-card';
    card.id = `player-${playerId}`;
    card.innerHTML = `
        <h3>${name}</h3>
        <p class="status">Connected</p>
    `;
    
    playersList.appendChild(card);
}

function removePlayerFromList(playerId) {
    const card = document.getElementById(`player-${playerId}`);
    if (card) {
        card.remove();
    }
    
    // Show empty state if no players
    const playersList = document.getElementById('players-list');
    if (playersList && playersList.children.length === 0) {
        playersList.innerHTML = '<p class="empty-state">No players connected yet...</p>';
    }
}

function updatePlayersList(players) {
    const playersList = document.getElementById('players-list');
    if (!playersList) return;
    
    // Clear list
    playersList.innerHTML = '';
    
    if (players.length === 0) {
        playersList.innerHTML = '<p class="empty-state">No players connected yet...</p>';
        return;
    }
    
    // Add each player
    players.forEach(player => {
        addPlayerToList(player.player_id, player.name);
    });
}

async function loadQRCode() {
    try {
        const response = await fetch('/api/qr-code');
        const data = await response.json();
        
        const qrContainer = document.getElementById('qr-code-container');
        if (qrContainer) {
            qrContainer.innerHTML = `
                <img src="${data.qr_code}" alt="QR Code" style="width: 200px; height: 200px;" />
            `;
        }
        
        const urlElement = document.getElementById('connection-url');
        if (urlElement) {
            urlElement.textContent = data.url;
        }
    } catch (error) {
        console.error('Failed to load QR code:', error);
    }
}
