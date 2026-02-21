// Daggerheart VTT Client
// Phase 3: Daggerheart Integration

console.log('🎲 Daggerheart VTT Client - Phase 3');

let ws = null;
let currentPlayerId = null;
let currentPlayerName = null;
let currentCharacter = null;
let mapCanvas = null;
let characterCreator = null;

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
    
    // Initialize canvas
    mapCanvas = new MapCanvas('game-canvas');
    
    // Load QR code
    loadQRCode();
    
    // Connect to WebSocket
    ws = new WebSocketClient(handleServerMessage);
    window.ws = ws; // Update global reference
    ws.connect();
}

function initMobileView() {
    console.log('Initializing mobile view');
    
    const joinButton = document.getElementById('join-button');
    const playerNameInput = document.getElementById('player-name');
    const leaveButton = document.getElementById('leave-button');
    const leaveButtonBasic = document.getElementById('leave-button-basic');
    const createCharBtn = document.getElementById('create-char-btn');
    const rollBtn = document.getElementById('roll-btn');
    
    // Initialize character creator
    characterCreator = new CharacterCreator();
    
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
    
    if (leaveButtonBasic) {
        leaveButtonBasic.addEventListener('click', () => {
            leaveGame();
        });
    }
    
    if (createCharBtn) {
        createCharBtn.addEventListener('click', () => {
            showCharacterCreation();
        });
    }
    
    if (rollBtn) {
        rollBtn.addEventListener('click', () => {
            rollDuality();
        });
    }
    
    // Mobile tap-to-move (only the character sheet canvas)
    setupMobileCanvas('mini-canvas');
}

function setupMobileCanvas(canvasId) {
    const canvas = document.getElementById(canvasId);
    if (!canvas) return;
    
    canvas.addEventListener('click', (e) => {
        if (currentPlayerId && ws && mapCanvas) {
            const pos = mapCanvas.getClickPosition(e);
            console.log('Tap to move:', pos);
            ws.send('player_move', { x: pos.x, y: pos.y });
        }
    });
    
    canvas.addEventListener('touchend', (e) => {
        e.preventDefault();
        if (currentPlayerId && ws && mapCanvas && e.changedTouches.length > 0) {
            const touch = e.changedTouches[0];
            const pos = mapCanvas.getClickPosition(touch);
            console.log('Touch to move:', pos);
            ws.send('player_move', { x: pos.x, y: pos.y });
        }
    });
}

function autoRejoin(playerName) {
    console.log('Auto-rejoining as:', playerName);
    
    // Connect to WebSocket
    ws = new WebSocketClient(handleServerMessage);
    window.ws = ws; // Update global reference
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
    window.ws = ws; // Update global reference
    ws.connect();
    
    // Wait a bit for connection, then send join message
    setTimeout(() => {
        ws.send('player_join', { name: playerName });
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
    
    // Clear canvas
    if (mapCanvas) {
        mapCanvas.clearPlayers();
    }
    
    // Reset UI
    showPanel('join-panel');
    document.getElementById('player-name').value = '';
    
    currentPlayerName = null;
    currentPlayerId = null;
    currentCharacter = null;
}

function showPanel(panelId) {
    // Hide all panels
    const panels = ['join-panel', 'player-info', 'char-creation-panel', 'char-sheet-panel'];
    panels.forEach(id => {
        const el = document.getElementById(id);
        if (el) el.style.display = 'none';
    });
    
    // Show requested panel
    const panel = document.getElementById(panelId);
    if (panel) panel.style.display = 'block';
}

function showCharacterCreation() {
    showPanel('char-creation-panel');
    const container = document.getElementById('char-creation-container');
    characterCreator.init(container);
}

function showCharacterSheet(character) {
    showPanel('char-sheet-panel');
    updateCharacterSheet(character);
    
    // Initialize/reinitialize mini canvas for character sheet
    // (Always create fresh to ensure correct canvas element)
    console.log('Initializing character sheet canvas...');
    mapCanvas = new MapCanvas('mini-canvas');
}

function updateCharacterSheet(character) {
    currentCharacter = character;
    
    // Update header
    document.getElementById('char-name').textContent = character.name;
    document.getElementById('char-details').textContent = `${character.class} • ${character.ancestry}`;
    
    // Update resources
    document.getElementById('hp-current').textContent = character.hp.current;
    document.getElementById('hp-max').textContent = character.hp.maximum;
    const hpPercent = (character.hp.current / character.hp.maximum) * 100;
    document.getElementById('hp-bar').style.width = `${hpPercent}%`;
    
    document.getElementById('stress-value').textContent = character.stress;
    
    document.getElementById('hope-current').textContent = character.hope.current;
    document.getElementById('hope-max').textContent = character.hope.maximum;
    const hopePercent = (character.hope.current / character.hope.maximum) * 100;
    document.getElementById('hope-bar').style.width = `${hopePercent}%`;
    
    document.getElementById('evasion-value').textContent = character.evasion;
    
    // Update attributes
    const attrs = character.attributes;
    document.getElementById('attr-agility').textContent = formatModifier(attrs.agility);
    document.getElementById('attr-strength').textContent = formatModifier(attrs.strength);
    document.getElementById('attr-finesse').textContent = formatModifier(attrs.finesse);
    document.getElementById('attr-instinct').textContent = formatModifier(attrs.instinct);
    document.getElementById('attr-presence').textContent = formatModifier(attrs.presence);
    document.getElementById('attr-knowledge').textContent = formatModifier(attrs.knowledge);
}

function formatModifier(value) {
    return value >= 0 ? `+${value}` : `${value}`;
}

function rollDuality() {
    if (!ws) return;
    
    // TODO: Add modifier selection UI
    const modifier = 0;
    const withAdvantage = false;
    
    ws.send('roll_duality', {
        modifier,
        with_advantage: withAdvantage,
    });
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
            handleCharacterCreated(payload);
            break;
        case 'character_updated':
            handleCharacterUpdated(payload);
            break;
        case 'roll_result':
            handleRollResult(payload);
            break;
        case 'error':
            handleError(payload);
            break;
        default:
            console.warn('Unknown message type:', type);
    }
}

function handlePlayerJoined(payload) {
    const { player_id, name, position, color } = payload;
    console.log(`Player joined: ${name} (${player_id}) at (${position.x}, ${position.y})`);
    
    // Track our own player ID
    if (name === currentPlayerName) {
        currentPlayerId = player_id;
        console.log('Set our player ID:', currentPlayerId);
        
        // Show player info panel (no character yet)
        showPanel('player-info');
        document.getElementById('player-name-display').textContent = name;
        
        // Add color dot
        const colorDot = document.createElement('span');
        colorDot.style.display = 'inline-block';
        colorDot.style.width = '20px';
        colorDot.style.height = '20px';
        colorDot.style.borderRadius = '50%';
        colorDot.style.backgroundColor = color;
        colorDot.style.marginLeft = '10px';
        colorDot.style.verticalAlign = 'middle';
        const nameDisplay = document.getElementById('player-name-display');
        if (nameDisplay && !nameDisplay.querySelector('span')) {
            nameDisplay.appendChild(colorDot);
        }
        
        // NOTE: Canvas initialization happens AFTER character creation
        // See showCharacterSheet() function
    }
    
    // Add to canvas
    if (mapCanvas) {
        mapCanvas.addPlayer(player_id, name, position, color);
    }
    
    // Add to players list if we're on desktop
    if (!window.location.pathname.includes('mobile')) {
        addPlayerToList(player_id, name, color);
    }
}

function handlePlayerLeft(payload) {
    const { player_id, name } = payload;
    console.log(`Player left: ${name} (${player_id})`);
    
    // Remove from canvas
    if (mapCanvas) {
        mapCanvas.removePlayer(player_id);
    }
    
    // Remove from players list if we're on desktop
    if (!window.location.pathname.includes('mobile')) {
        removePlayerFromList(player_id);
    }
}

function handlePlayersList(payload) {
    const { players } = payload;
    console.log('Players list:', players);
    
    // Clear and re-add all players to canvas
    if (mapCanvas) {
        mapCanvas.clearPlayers();
        players.forEach(player => {
            mapCanvas.addPlayer(player.player_id, player.name, player.position, player.color);
        });
    }
    
    // Update players list on desktop
    if (!window.location.pathname.includes('mobile')) {
        updatePlayersList(players);
    }
}

function handlePlayerMoved(payload) {
    const { player_id, position } = payload;
    console.log(`Player ${player_id} moved to (${position.x}, ${position.y})`);
    
    // Update canvas
    if (mapCanvas) {
        mapCanvas.updatePlayerPosition(player_id, position);
    }
}

function handleCharacterCreated(payload) {
    const { player_id, character } = payload;
    console.log('Character created:', character);
    
    // If it's our character, show character sheet
    if (player_id === currentPlayerId) {
        showCharacterSheet(character);
    }
}

function handleCharacterUpdated(payload) {
    const { player_id, character } = payload;
    console.log('Character updated:', character);
    
    // If it's our character, update sheet
    if (player_id === currentPlayerId) {
        updateCharacterSheet(character);
    }
}

function handleRollResult(payload) {
    const { player_id, player_name, roll } = payload;
    console.log(`${player_name} rolled:`, roll);
    
    // Show roll result on TV
    if (!window.location.pathname.includes('mobile')) {
        showRollResultOnTV(player_name, roll);
    }
    
    // If on mobile and it's our roll, could show feedback
    if (player_id === currentPlayerId) {
        console.log('Your roll result:', roll);
        // Could add mobile-specific feedback here
    }
}

function showRollResultOnTV(playerName, roll) {
    const overlay = document.getElementById('roll-overlay');
    if (!overlay) return;
    
    // Update content
    document.getElementById('roll-player').textContent = playerName;
    document.getElementById('hope-value').textContent = roll.hope;
    document.getElementById('fear-value').textContent = roll.fear;
    document.getElementById('total-value').textContent = roll.total;
    
    // Update controlling die badge
    const controllingBadge = document.getElementById('controlling-die');
    controllingBadge.className = 'controlling-badge';
    if (roll.controlling_die === 'Hope') {
        controllingBadge.classList.add('hope');
        controllingBadge.textContent = 'With Hope';
    } else if (roll.controlling_die === 'Fear') {
        controllingBadge.classList.add('fear');
        controllingBadge.textContent = 'With Fear';
    } else {
        controllingBadge.classList.add('tied');
        controllingBadge.textContent = 'Tied';
    }
    
    // Update success badge
    const successBadge = document.getElementById('success-badge');
    successBadge.className = 'success-badge';
    if (roll.is_success) {
        successBadge.classList.add('success');
        successBadge.textContent = 'SUCCESS';
    } else {
        successBadge.classList.add('failure');
        successBadge.textContent = 'FAILURE';
    }
    
    // Show/hide critical badge
    const criticalBadge = document.getElementById('critical-badge');
    if (roll.is_critical) {
        criticalBadge.style.display = 'block';
    } else {
        criticalBadge.style.display = 'none';
    }
    
    // Show overlay
    overlay.style.display = 'block';
    
    // Hide after 4 seconds
    setTimeout(() => {
        overlay.style.display = 'none';
    }, 4000);
}

function handleError(payload) {
    const { message } = payload;
    console.error('Server error:', message);
    alert(`Error: ${message}`);
}

function addPlayerToList(playerId, name, color) {
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
    card.style.borderLeftColor = color;
    card.innerHTML = `
        <h3><span style="display:inline-block;width:12px;height:12px;border-radius:50%;background:${color};margin-right:8px;"></span>${name}</h3>
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
        addPlayerToList(player.player_id, player.name, player.color);
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

// Note: window.ws is updated dynamically when WebSocket connections are created
// (see initDesktopView, autoRejoin, joinGame functions)
