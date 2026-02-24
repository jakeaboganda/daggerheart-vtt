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
    
    // Setup canvas click handler for adversary spawning
    const canvas = document.getElementById('gm-canvas');
    if (canvas) {
        canvas.addEventListener('click', (e) => {
            const rect = canvas.getBoundingClientRect();
            const scaleX = canvas.width / rect.width;
            const scaleY = canvas.height / rect.height;
            const x = (e.clientX - rect.left) * scaleX;
            const y = (e.clientY - rect.top) * scaleY;
            
            if (window.handleCanvasClick) {
                window.handleCanvasClick(x, y);
            }
        });
        console.log('Canvas click handler installed');
    }
    
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
        case 'combat_started':
            handleCombatStarted(payload);
            break;
        case 'combat_ended':
            handleCombatEnded(payload);
            break;
        case 'tracker_updated':
            handleTrackerUpdated(payload);
            break;
        default:
            console.log('GM received:', type, payload);
    }
}

function handleConnected(payload) {
    const { connection_id } = payload;
    console.log('✅ GM Connected with ID:', connection_id);
    
    // Load event history
    loadEventHistory();
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

// Event Log Functions for GM
function handleGameEvent(payload) {
    addEventToLog(payload);
}

function addEventToLog(event) {
    const eventLog = document.getElementById('event-log-gm');
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
        
        const eventLog = document.getElementById('event-log-gm');
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

// ===== Adversary Management =====

let adversaries = [];
let spawnMode = false;

// Add adversary event listeners to setupEventListeners
(function() {
    const originalSetup = setupEventListeners;
    setupEventListeners = function() {
        originalSetup();
        
        // Adversary template selector
        document.getElementById('adversary-template').addEventListener('change', (e) => {
            const customPanel = document.getElementById('custom-adversary-panel');
            if (e.target.value === 'custom') {
                customPanel.style.display = 'block';
            } else {
                customPanel.style.display = 'none';
            }
        });
        
        // Spawn adversary button
        document.getElementById('spawn-adversary-btn').addEventListener('click', () => {
            spawnMode = !spawnMode;
            const btn = document.getElementById('spawn-adversary-btn');
            if (spawnMode) {
                btn.textContent = '❌ Cancel Spawn';
                btn.style.background = 'var(--fear-color)';
                console.log('🎯 Click map to spawn adversary');
            } else {
                btn.textContent = '➕ Click Map to Spawn';
                btn.style.background = '';
                console.log('Spawn mode cancelled');
            }
        });
        
        // Combat controls
        document.getElementById('start-combat-btn').addEventListener('click', startCombat);
        document.getElementById('end-combat-btn').addEventListener('click', endCombat);
        document.getElementById('add-pc-token').addEventListener('click', () => addTrackerToken('pc'));
        document.getElementById('add-adv-token').addEventListener('click', () => addTrackerToken('adversary'));
        document.getElementById('reset-tracker').addEventListener('click', resetTracker);
    };
})();

// Handle canvas click for spawning
(function() {
    const originalHandleCanvasClick = window.handleCanvasClick || function() {};
    window.handleCanvasClick = function(x, y) {
        if (spawnMode) {
            spawnAdversaryAtPosition(x, y);
            spawnMode = false;
            const btn = document.getElementById('spawn-adversary-btn');
            btn.textContent = '➕ Click Map to Spawn';
            btn.style.background = '';
        } else {
            originalHandleCanvasClick(x, y);
        }
    };
})();

function spawnAdversaryAtPosition(x, y) {
    const template = document.getElementById('adversary-template').value;
    
    if (template === 'custom') {
        // Spawn custom adversary
        const name = document.getElementById('adv-name').value;
        const hp = parseInt(document.getElementById('adv-hp').value);
        const evasion = parseInt(document.getElementById('adv-evasion').value);
        const armor = parseInt(document.getElementById('adv-armor').value);
        const attackMod = parseInt(document.getElementById('adv-attack-mod').value);
        const damage = document.getElementById('adv-damage').value;
        
        ws.send({
            type: 'spawn_custom_adversary',
            name,
            position: { x, y },
            hp,
            evasion,
            armor,
            attack_modifier: attackMod,
            damage_dice: damage
        });
        
        console.log(`🎯 Spawning custom adversary "${name}" at (${x}, ${y})`);
    } else {
        // Spawn from template
        ws.send({
            type: 'spawn_adversary',
            template,
            position: { x, y }
        });
        
        console.log(`🎯 Spawning ${template} at (${x}, ${y})`);
    }
}

function handleAdversarySpawned(payload) {
    const {
        adversary_id,
        name,
        template,
        position,
        hp,
        max_hp,
        evasion,
        armor,
        attack_modifier,
        damage_dice
    } = payload;
    
    // Add to local list
    adversaries.push({
        id: adversary_id,
        name,
        template,
        position,
        hp,
        max_hp,
        evasion,
        armor,
        attack_modifier,
        damage_dice,
        is_active: true
    });
    
    // Render on canvas
    if (mapCanvas) {
        mapCanvas.drawAdversary(adversary_id, name, position.x, position.y);
    }
    
    // Update adversary list
    renderAdversariesList();
    
    console.log(`👹 Adversary spawned: ${name} (${adversary_id})`);
}

function handleAdversaryRemoved(payload) {
    const { adversary_id, name } = payload;
    
    // Remove from local list
    adversaries = adversaries.filter(adv => adv.id !== adversary_id);
    
    // Remove from canvas
    if (mapCanvas) {
        mapCanvas.removeAdversary(adversary_id);
    }
    
    // Update list
    renderAdversariesList();
    
    console.log(`💀 Adversary removed: ${name}`);
}

function handleAdversaryUpdated(payload) {
    const { adversary_id, hp, stress, is_active } = payload;
    
    // Update local list
    const adversary = adversaries.find(adv => adv.id === adversary_id);
    if (adversary) {
        adversary.hp = hp;
        adversary.stress = stress;
        adversary.is_active = is_active;
        
        // Update display
        renderAdversariesList();
        
        // Update canvas (maybe show HP bar?)
        if (mapCanvas) {
            mapCanvas.updateAdversaryHP(adversary_id, hp, adversary.max_hp);
        }
    }
}

function renderAdversariesList() {
    const listEl = document.getElementById('adversaries-list');
    
    if (adversaries.length === 0) {
        listEl.innerHTML = '<p class="empty-state">No adversaries spawned</p>';
        return;
    }
    
    let html = '';
    adversaries.forEach(adv => {
        const hpPercent = (adv.hp / adv.max_hp) * 100;
        const statusIcon = adv.is_active ? '🗡️' : '💀';
        
        html += `
            <div class="adversary-item" data-id="${adv.id}">
                <h5>
                    ${statusIcon} ${adv.name}
                    <button onclick="removeAdversary('${adv.id}')" style="padding: 0.25rem 0.5rem; font-size: 0.8rem; background: var(--fear-color); color: white; border: none; border-radius: 4px; cursor: pointer;">🗑️</button>
                </h5>
                <div class="adversary-stats">
                    <div class="adversary-stat">HP: <strong>${adv.hp}/${adv.max_hp}</strong></div>
                    <div class="adversary-stat">Stress: <strong>${adv.stress || 0}/${adv.max_hp}</strong></div>
                    <div class="adversary-stat">Evasion: <strong>${adv.evasion}</strong></div>
                    <div class="adversary-stat">Armor: <strong>${adv.armor}</strong></div>
                </div>
                <div style="background: var(--bg-dark); height: 4px; border-radius: 2px; overflow: hidden; margin-top: 0.5rem;">
                    <div style="height: 100%; background: var(--hope-color); width: ${hpPercent}%;"></div>
                </div>
            </div>
        `;
    });
    
    listEl.innerHTML = html;
}

function removeAdversary(adversaryId) {
    if (confirm('Remove this adversary?')) {
        ws.send({
            type: 'remove_adversary',
            adversary_id: adversaryId
        });
    }
}

// ===== Combat Management =====

let combatActive = false;

function startCombat() {
    ws.send({ type: 'start_combat' });
    console.log('▶️ Starting combat...');
}

function endCombat() {
    if (confirm('End combat encounter?')) {
        ws.send({ type: 'end_combat' });
        console.log('🛑 Ending combat...');
    }
}

function handleCombatStarted(payload) {
    const { encounter_id, pc_tokens, adversary_tokens } = payload;
    
    combatActive = true;
    
    // Show combat controls
    document.getElementById('start-combat-btn').style.display = 'none';
    document.getElementById('end-combat-btn').style.display = 'block';
    document.getElementById('combat-controls').style.display = 'block';
    
    // Update tracker display
    document.getElementById('pc-tokens').textContent = pc_tokens;
    document.getElementById('adv-tokens').textContent = adversary_tokens;
    
    console.log(`⚔️ Combat started! Encounter: ${encounter_id}`);
}

function handleCombatEnded(payload) {
    const { reason } = payload;
    
    combatActive = false;
    
    // Hide combat controls
    document.getElementById('start-combat-btn').style.display = 'block';
    document.getElementById('end-combat-btn').style.display = 'none';
    document.getElementById('combat-controls').style.display = 'none';
    
    console.log(`✅ Combat ended: ${reason}`);
}

function handleTrackerUpdated(payload) {
    const { pc_tokens, adversary_tokens, next_token } = payload;
    
    document.getElementById('pc-tokens').textContent = pc_tokens;
    document.getElementById('adv-tokens').textContent = adversary_tokens;
    
    console.log(`🎲 Tracker updated: PC ${pc_tokens}, Adversary ${adversary_tokens}, Next: ${next_token}`);
}

function addTrackerToken(tokenType) {
    ws.send({
        type: 'add_tracker_token',
        token_type: tokenType
    });
}

function resetTracker() {
    // TODO: Implement reset tracker
    console.log('Reset tracker not yet implemented');
}
