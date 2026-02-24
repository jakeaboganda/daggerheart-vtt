// Daggerheart VTT Client
// Phase 5A: Character-Centric Architecture

console.log('🎲 Daggerheart VTT Client - Phase 5A');

let ws = null;
let currentConnectionId = null;
let currentCharacterId = null;
let currentCharacter = null;
let mapCanvas = null;
let characterCreator = null;
let allCharacters = []; // Store all characters for canvas repopulation

// LocalStorage keys
const STORAGE_KEYS = {
    CHARACTER_ID: 'dh_vtt_character_id',
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
    
    // Check if we have a saved character
    const savedCharId = localStorage.getItem(STORAGE_KEYS.CHARACTER_ID);
    const sessionActive = localStorage.getItem(STORAGE_KEYS.SESSION_ACTIVE) === 'true';
    
    if (savedCharId && sessionActive) {
        console.log('Found saved character, auto-reconnecting:', savedCharId);
        
        // Auto-reconnect
        setTimeout(() => {
            autoReconnect(savedCharId);
        }, 500);
    }
    
    if (joinButton) {
        console.log('✅ Join button found, adding click handler');
        joinButton.addEventListener('click', () => {
            console.log('🖱️ Join button clicked!');
            const name = playerNameInput.value.trim();
            console.log('📝 Player name entered:', name);
            if (name) {
                // Just connect first, we'll create character after
                connectToGame(name);
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
    
    // Mobile tap-to-move
    setupMobileCanvas('mini-canvas');
}

function setupMobileCanvas(canvasId) {
    const canvas = document.getElementById(canvasId);
    if (!canvas) return;
    
    canvas.addEventListener('click', (e) => {
        if (currentCharacterId && ws) {
            const rect = canvas.getBoundingClientRect();
            const x = (e.clientX - rect.left) * (canvas.width / rect.width);
            const y = (e.clientY - rect.top) * (canvas.height / rect.height);
            console.log('Tap to move:', {x, y});
            ws.send('move_character', { x, y });
        }
    });
    
    canvas.addEventListener('touchend', (e) => {
        e.preventDefault();
        if (currentCharacterId && ws && e.changedTouches.length > 0) {
            const touch = e.changedTouches[0];
            const rect = canvas.getBoundingClientRect();
            const x = (touch.clientX - rect.left) * (canvas.width / rect.width);
            const y = (touch.clientY - rect.top) * (canvas.height / rect.height);
            console.log('Touch to move:', {x, y});
            ws.send('move_character', { x, y });
        }
    });
}

function autoReconnect(characterId) {
    console.log('Auto-reconnecting and selecting character:', characterId);
    
    // Set flag to indicate we're auto-reconnecting (for error handling)
    window.isAutoReconnecting = true;
    
    // Connect to WebSocket
    ws = new WebSocketClient(handleServerMessage);
    window.ws = ws; // Update global reference
    ws.connect();
    
    // After connection, try to select the character
    setTimeout(() => {
        if (ws) {
            ws.send('select_character', { character_id: characterId });
            
            // Clear auto-reconnect flag after a delay
            setTimeout(() => {
                window.isAutoReconnecting = false;
            }, 2000);
        }
    }, 500);
}

function connectToGame(playerName) {
    console.log('🔌 connectToGame() called with name:', playerName);
    
    // Connect to WebSocket
    ws = new WebSocketClient(handleServerMessage);
    window.ws = ws; // Update global reference
    
    console.log('📡 Connecting to WebSocket...');
    ws.connect();
    
    // After connection is established, we'll show character creation
    // (The 'connected' message handler will trigger this)
    window.pendingPlayerName = playerName; // Store for character creation
    console.log('✅ Stored pendingPlayerName:', window.pendingPlayerName);
}

function leaveGame() {
    console.log('Leaving game');
    
    // Clear localStorage
    localStorage.removeItem(STORAGE_KEYS.CHARACTER_ID);
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
    
    currentConnectionId = null;
    currentCharacterId = null;
    currentCharacter = null;
}

function showPanel(panelId) {
    // Hide all panels
    const panels = ['join-panel', 'player-info', 'char-creation-panel', 'char-sheet-panel', 'char-select-panel'];
    panels.forEach(id => {
        const el = document.getElementById(id);
        if (el) el.style.display = 'none';
    });
    
    // Show requested panel
    const panel = document.getElementById(panelId);
    if (panel) panel.style.display = 'block';
}

function showCharacterCreation() {
    console.log('🎨 showCharacterCreation() called');
    console.log('📦 characterCreator:', characterCreator);
    
    showPanel('char-creation-panel');
    
    const container = document.getElementById('char-creation-container');
    console.log('📦 char-creation-container:', container);
    
    if (container && characterCreator) {
        characterCreator.init(container);
        console.log('✅ Character creator initialized');
    } else {
        console.error('❌ Failed to initialize character creator - container:', container, 'creator:', characterCreator);
    }
}

function showCharacterSelection(characters) {
    showPanel('char-select-panel');
    const list = document.getElementById('char-select-list');
    if (!list) return;
    
    list.innerHTML = '';
    
    // Filter for player characters that aren't controlled by others
    const availableChars = characters.filter(c => !c.is_npc && !c.controlled_by_other);
    
    if (availableChars.length === 0) {
        list.innerHTML = '<p>No available characters. Create a new one!</p>';
        return;
    }
    
    availableChars.forEach(char => {
        const card = document.createElement('div');
        card.className = 'character-card';
        card.innerHTML = `
            <h3>${char.name}</h3>
            <p>${char.class} • ${char.ancestry}</p>
            <button onclick="selectCharacter('${char.id}')">Select</button>
        `;
        list.appendChild(card);
    });
}

function selectCharacter(characterId) {
    console.log('Selecting character:', characterId);
    if (ws) {
        ws.send('select_character', { character_id: characterId });
    }
}

function showCharacterSheet(character) {
    showPanel('char-sheet-panel');
    updateCharacterSheet(character);
    
    // Initialize/reinitialize mini canvas for character sheet
    console.log('📊 Initializing character sheet canvas...');
    mapCanvas = new MapCanvas('mini-canvas');
    
    // Repopulate canvas with all characters
    console.log(`🎮 Repopulating canvas with ${allCharacters.length} characters:`);
    allCharacters.forEach(char => {
        console.log(`   - ${char.id.substring(0, 8)}: "${char.name}"`);
        mapCanvas.addPlayer(char.id, char.name, char.position, char.color);
    });
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
        case 'connected':
            handleConnected(payload);
            break;
        case 'characters_list':
            handleCharactersList(payload);
            break;
        case 'character_selected':
            handleCharacterSelected(payload);
            break;
        case 'character_spawned':
            handleCharacterSpawned(payload);
            break;
        case 'character_moved':
            handleCharacterMoved(payload);
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
        case 'roll_requested':
            handleRollRequested(payload);
            break;
        case 'detailed_roll_result':
            handleDetailedRollResult(payload);
            break;
        case 'roll_request_status':
            handleRollRequestStatus(payload);
            break;
        case 'game_event':
            handleGameEvent(payload);
            break;
        case 'adversary_spawned':
            handleAdversarySpawned(payload);
            break;
        case 'adversary_removed':
            handleAdversaryRemoved(payload);
            break;
        case 'adversary_updated':
            handleAdversaryUpdated(payload);
            break;
        case 'error':
            handleError(payload);
            break;
        default:
            console.warn('Unknown message type:', type);
    }
}

function handleConnected(payload) {
    const { connection_id } = payload;
    console.log('✅ Connected with ID:', connection_id);
    console.log('📍 Current pathname:', window.location.pathname);
    console.log('📝 pendingPlayerName:', window.pendingPlayerName);
    console.log('🎯 Is mobile?', window.location.pathname.includes('mobile'));
    console.log('🎯 Has pendingPlayerName?', !!window.pendingPlayerName);
    
    currentConnectionId = connection_id;
    
    // If we're on mobile and just joined, show character creation
    if (window.location.pathname.includes('mobile') && window.pendingPlayerName) {
        const playerName = window.pendingPlayerName;
        delete window.pendingPlayerName;
        
        console.log('🎨 Showing character creation for:', playerName);
        
        // Show character creation immediately (no separate join panel needed)
        showCharacterCreation();
        
        // Pre-fill the name if the creator supports it
        if (characterCreator && characterCreator.setName) {
            characterCreator.setName(playerName);
            console.log('✏️ Pre-filled character name:', playerName);
        } else {
            console.warn('⚠️ characterCreator not available or missing setName()');
        }
    } else {
        console.log('⚠️ Not showing character creation - mobile:', window.location.pathname.includes('mobile'), 'pendingPlayerName:', !!window.pendingPlayerName);
    }
    
    // Load event history (for TV and GM views)
    if (!window.location.pathname.includes('mobile')) {
        loadEventHistory();
    }
}

function handleCharactersList(payload) {
    const { characters } = payload;
    console.log('Characters list:', characters);
    
    // Store characters for later use
    allCharacters = characters;
    
    // Clear and re-add all characters to canvas
    if (mapCanvas) {
        mapCanvas.clearPlayers();
        characters.forEach(char => {
            mapCanvas.addPlayer(char.id, char.name, char.position, char.color);
        });
    }
    
    // Update characters list on desktop
    if (!window.location.pathname.includes('mobile')) {
        updateCharactersList(characters);
    }
}

function handleCharacterSelected(payload) {
    const { character_id, character } = payload;
    console.log('✅ Character selected:', character);
    
    currentCharacterId = character_id;
    
    // Save to localStorage
    localStorage.setItem(STORAGE_KEYS.CHARACTER_ID, character_id);
    localStorage.setItem(STORAGE_KEYS.SESSION_ACTIVE, 'true');
    
    // Show character sheet
    showCharacterSheet(character);
}

function handleCharacterSpawned(payload) {
    const { character_id, name, position, color, is_npc } = payload;
    console.log(`Character spawned: ${name} (${character_id}) at (${position.x}, ${position.y})`);
    
    // Add to all characters list
    allCharacters.push({
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
    
    // Add to characters list if we're on desktop
    if (!window.location.pathname.includes('mobile')) {
        addCharacterToList(character_id, name, color, is_npc);
    }
}

function handleCharacterMoved(payload) {
    const { character_id, position } = payload;
    console.log(`Character ${character_id} moved to (${position.x}, ${position.y})`);
    
    // Update in allCharacters
    const char = allCharacters.find(c => c.id === character_id);
    if (char) {
        char.position = position;
    }
    
    // Update canvas
    if (mapCanvas) {
        mapCanvas.updatePlayerPosition(character_id, position);
    }
}

function handleCharacterCreated(payload) {
    const { character_id, character } = payload;
    console.log('Character created:', character);
    
    // This character is automatically selected for us
    currentCharacterId = character_id;
    
    // Save to localStorage
    localStorage.setItem(STORAGE_KEYS.CHARACTER_ID, character_id);
    localStorage.setItem(STORAGE_KEYS.SESSION_ACTIVE, 'true');
    
    // Show character sheet
    showCharacterSheet(character);
}

function handleCharacterUpdated(payload) {
    const { character_id, character } = payload;
    console.log('Character updated:', character);
    
    // If it's our character, update sheet
    if (character_id === currentCharacterId) {
        updateCharacterSheet(character);
    }
}

function handleRollResult(payload) {
    const { character_id, character_name, roll } = payload;
    console.log(`${character_name} rolled:`, roll);
    
    // Show roll result on TV
    if (!window.location.pathname.includes('mobile')) {
        showRollResultOnTV(character_name, roll);
    }
    
    // If on mobile and it's our roll, could show feedback
    if (character_id === currentCharacterId) {
        console.log('Your roll result:', roll);
        // Could add mobile-specific feedback here
    }
}

// Handle GM roll request
let currentRollRequest = null;

function handleRollRequested(payload) {
    console.log('Roll requested:', payload);
    
    // Store the current request
    currentRollRequest = payload;
    
    // Only show on mobile if this character is targeted
    if (!window.location.pathname.includes('mobile')) {
        return;
    }
    
    const rollPanel = document.getElementById('roll-request-panel');
    const rollBtn = document.getElementById('roll-btn');
    
    if (!rollPanel) return;
    
    // Populate the panel
    document.getElementById('roll-context').textContent = payload.context || 'Roll requested';
    
    const attrName = payload.attribute ? payload.attribute.charAt(0).toUpperCase() + payload.attribute.slice(1) : 'None';
    document.getElementById('roll-attribute').textContent = attrName;
    document.getElementById('roll-difficulty').textContent = payload.difficulty;
    document.getElementById('roll-modifier').textContent = `+${payload.total_modifier}`;
    
    // Show/hide advantage
    const advantageInfo = document.getElementById('advantage-info');
    if (payload.has_advantage) {
        advantageInfo.style.display = 'flex';
    } else {
        advantageInfo.style.display = 'none';
    }
    
    // Show/hide Hope spending option
    const hopeLabel = document.getElementById('spend-hope-label');
    if (payload.can_spend_hope) {
        hopeLabel.style.display = 'flex';
    } else {
        hopeLabel.style.display = 'none';
    }
    
    // Show panel, hide normal roll button
    rollPanel.style.display = 'block';
    if (rollBtn) rollBtn.style.display = 'none';
    
    // Set up execute button
    const executeBtn = document.getElementById('execute-roll-btn');
    if (executeBtn) {
        executeBtn.onclick = () => executeRollRequest();
    }
}

function executeRollRequest() {
    if (!currentRollRequest) {
        console.error('No roll request to execute');
        return;
    }
    
    const spendHopeCheckbox = document.getElementById('spend-hope-checkbox');
    const spendHope = spendHopeCheckbox ? spendHopeCheckbox.checked : false;
    
    console.log('Executing roll request:', currentRollRequest.request_id, 'spend hope:', spendHope);
    
    ws.send('execute_roll', {
        request_id: currentRollRequest.request_id,
        spend_hope_for_bonus: spendHope,
        chosen_experience: null, // TODO: Add experience selection UI
    });
    
    // Hide the panel
    const rollPanel = document.getElementById('roll-request-panel');
    if (rollPanel) rollPanel.style.display = 'none';
    
    // Show normal roll button again
    const rollBtn = document.getElementById('roll-btn');
    if (rollBtn) rollBtn.style.display = 'block';
    
    // Clear the request
    currentRollRequest = null;
}

function handleDetailedRollResult(payload) {
    console.log('Detailed roll result:', payload);
    
    // Show on TV if not mobile
    if (!window.location.pathname.includes('mobile')) {
        showDetailedRollResultOnTV(payload);
    }
    
    // If this is our roll, show feedback on mobile
    if (payload.character_id === currentCharacterId) {
        // Could show toast notification or result panel
        console.log(`Your roll: ${payload.outcome_description}`);
    }
}

function handleRollRequestStatus(payload) {
    console.log('Roll request status:', payload);
    
    // Update TV display to show who has/hasn't rolled
    if (!window.location.pathname.includes('mobile')) {
        updateRollStatusOnTV(payload);
    }
}

function handleError(payload) {
    const { message } = payload;
    console.error('Server error:', message);
    
    // If this is an auto-reconnect failure, handle it gracefully
    if (window.isAutoReconnecting && (
        message.includes('Character not found') || 
        message.includes('Connection not found') ||
        message.includes('select character')
    )) {
        console.warn('Auto-reconnect failed:', message, '- Clearing saved session.');
        
        // Clear saved character
        localStorage.removeItem(STORAGE_KEYS.CHARACTER_ID);
        localStorage.setItem(STORAGE_KEYS.SESSION_ACTIVE, 'false');
        window.isAutoReconnecting = false;
        
        // Show join screen instead of error popup
        const joinPanel = document.getElementById('join-panel');
        const charSheet = document.getElementById('char-sheet-panel');
        const charCreation = document.getElementById('char-creation-panel');
        
        if (joinPanel) joinPanel.style.display = 'block';
        if (charSheet) charSheet.style.display = 'none';
        if (charCreation) charCreation.style.display = 'none';
        
        console.log('✅ Returned to join screen. Please join the game again.');
        return;
    }
    
    // For other errors, show alert as normal
    alert(`Error: ${message}`);
}

function showRollResultOnTV(characterName, roll) {
    const overlay = document.getElementById('roll-overlay');
    if (!overlay) return;
    
    // Update content
    document.getElementById('roll-player').textContent = characterName;
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
    if (roll.is_success) {
        successBadge.textContent = 'SUCCESS';
        successBadge.className = 'success-badge success';
    } else {
        successBadge.textContent = 'FAILURE';
        successBadge.className = 'success-badge failure';
    }
    
    // Show critical badge if applicable
    const criticalBadge = document.getElementById('critical-badge');
    if (roll.is_critical) {
        criticalBadge.style.display = 'inline-block';
    } else {
        criticalBadge.style.display = 'none';
    }
    
    // Show overlay
    overlay.style.display = 'flex';
    
    // Hide after 5 seconds
    setTimeout(() => {
        overlay.style.display = 'none';
    }, 5000);
}

function addCharacterToList(characterId, name, color, isNpc) {
    const playersList = document.getElementById('players-list');
    if (!playersList) return;
    
    // Remove empty state if present
    const emptyState = playersList.querySelector('.empty-state');
    if (emptyState) {
        emptyState.remove();
    }
    
    // Create character card
    const card = document.createElement('div');
    card.className = 'player-card';
    card.id = `character-${characterId}`;
    card.style.borderLeftColor = color;
    card.innerHTML = `
        <h3><span style="display:inline-block;width:12px;height:12px;border-radius:50%;background:${color};margin-right:8px;"></span>${name}</h3>
        <p class="status">${isNpc ? 'NPC' : 'Player'}</p>
    `;
    
    playersList.appendChild(card);
}

function updateCharactersList(characters) {
    const playersList = document.getElementById('players-list');
    if (!playersList) return;
    
    // Clear list
    playersList.innerHTML = '';
    
    if (characters.length === 0) {
        playersList.innerHTML = '<p class="empty-state">No characters in game yet...</p>';
        return;
    }
    
    // Add each character
    characters.forEach(char => {
        addCharacterToList(char.id, char.name, char.color, char.is_npc);
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
// (see initDesktopView, autoReconnect, connectToGame functions)

// Global helper for character selection (called from HTML onclick)
window.selectCharacter = selectCharacter;

function showDetailedRollResultOnTV(result) {
    const overlay = document.getElementById('roll-overlay');
    if (!overlay) return;
    
    const roll = result.roll_details;
    
    // Update content
    document.getElementById('roll-player').textContent = result.character_name;
    document.getElementById('hope-value').textContent = roll.hope_die;
    document.getElementById('fear-value').textContent = roll.fear_die;
    document.getElementById('total-value').textContent = roll.total;
    
    // Update controlling die badge
    const controllingBadge = document.getElementById('controlling-die');
    controllingBadge.className = 'controlling-badge';
    
    if (roll.controlling_die === 'hope') {
        controllingBadge.classList.add('hope');
        controllingBadge.textContent = 'With Hope';
    } else if (roll.controlling_die === 'fear') {
        controllingBadge.classList.add('fear');
        controllingBadge.textContent = 'With Fear';
    } else {
        controllingBadge.classList.add('tied');
        controllingBadge.textContent = 'Tied';
    }
    
    // Update success badge
    const successBadge = document.getElementById('success-badge');
    const successType = roll.success_type;
    
    if (successType === 'critical_success') {
        successBadge.textContent = 'CRITICAL SUCCESS!';
        successBadge.className = 'success-badge success critical';
    } else if (successType === 'success_with_hope' || successType === 'success_with_fear') {
        successBadge.textContent = 'SUCCESS';
        successBadge.className = 'success-badge success';
    } else {
        successBadge.textContent = 'FAILURE';
        successBadge.className = 'success-badge failure';
    }
    
    // Show critical badge if applicable
    const criticalBadge = document.getElementById('critical-badge');
    if (roll.is_critical) {
        criticalBadge.style.display = 'inline-block';
    } else {
        criticalBadge.style.display = 'none';
    }
    
    // Show overlay
    overlay.style.display = 'flex';
    
    // Hide after 6 seconds
    setTimeout(() => {
        overlay.style.display = 'none';
    }, 6000);
}

function updateRollStatusOnTV(status) {
    // TODO: Add a roll status panel to the TV view
    console.log('Roll status update:', status);
    console.log('Completed:', status.completed_characters);
    console.log('Pending:', status.pending_characters);
    
    // For now, just log it - we can add a visual panel later
}

// Event Log Functions
function handleGameEvent(payload) {
    addEventToLog(payload);
}

// Adversary Handlers
function handleAdversarySpawned(payload) {
    const { adversary_id, name, position } = payload;
    console.log(`👹 Adversary spawned: ${name} at (${position.x}, ${position.y})`);
    
    if (mapCanvas) {
        mapCanvas.drawAdversary(adversary_id, name, position.x, position.y);
    }
}

function handleAdversaryRemoved(payload) {
    const { adversary_id, name } = payload;
    console.log(`💀 Adversary removed: ${name}`);
    
    if (mapCanvas) {
        mapCanvas.removeAdversary(adversary_id);
    }
}

function handleAdversaryUpdated(payload) {
    const { adversary_id, hp } = payload;
    console.log(`🩸 Adversary ${adversary_id} updated: HP ${hp}`);
    
    if (mapCanvas && payload.hp !== undefined) {
        // Assuming max_hp is sent or we track it
        mapCanvas.updateAdversaryHP(adversary_id, hp, payload.max_hp || hp);
    }
}

function addEventToLog(event) {
    const eventLog = document.getElementById('event-log');
    if (!eventLog) return;
    
    // Remove empty state message
    const emptyState = eventLog.querySelector('.empty-state');
    if (emptyState) {
        emptyState.remove();
    }
    
    // Create event item
    const item = document.createElement('div');
    item.className = `event-item event-type-${event.event_type.toLowerCase().replace(/_/g, '-')}`;
    
    let html = `
        <div>
            <span class="event-timestamp">${event.timestamp}</span>
            ${event.character_name ? `<span class="event-character">${event.character_name}:</span>` : ''}
            <span class="event-message">${event.message}</span>
        </div>
    `;
    
    if (event.details) {
        html += `<div class="event-details">${event.details}</div>`;
    }
    
    item.innerHTML = html;
    
    // Add to log (newest at bottom)
    eventLog.appendChild(item);
    
    // Keep only last 50 events in DOM
    while (eventLog.children.length > 50) {
        eventLog.removeChild(eventLog.firstChild);
    }
    
    // Auto-scroll to bottom
    eventLog.scrollTop = eventLog.scrollHeight;
}

// Load event history on connection
async function loadEventHistory() {
    try {
        const response = await fetch('/api/events');
        const data = await response.json();
        
        const eventLog = document.getElementById('event-log');
        if (!eventLog) return;
        
        // Clear existing
        eventLog.innerHTML = '';
        
        if (data.events && data.events.length > 0) {
            // Show last 30 events
            const recentEvents = data.events.slice(-30);
            recentEvents.forEach(event => addEventToLog(event));
        } else {
            eventLog.innerHTML = '<p class="empty-state">No events yet...</p>';
        }
    } catch (error) {
        console.error('Failed to load event history:', error);
    }
}
