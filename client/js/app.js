// Daggerheart VTT Client
// Phase 1: Foundation & Connection

console.log('🎲 Daggerheart VTT Client loaded');

// Phase 1: Basic placeholder
// WebSocket and real functionality will be added in implementation

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
    
    // Display connection URL
    const urlElement = document.getElementById('connection-url');
    if (urlElement) {
        const baseUrl = window.location.origin;
        urlElement.textContent = `${baseUrl}/mobile`;
    }
    
    // TODO: Generate QR code
    // TODO: Connect to WebSocket
    // TODO: Display connected players
}

function initMobileView() {
    console.log('Initializing mobile view');
    
    const joinButton = document.getElementById('join-button');
    const playerNameInput = document.getElementById('player-name');
    
    if (joinButton) {
        joinButton.addEventListener('click', () => {
            const name = playerNameInput.value.trim();
            if (name) {
                joinGame(name);
            } else {
                alert('Please enter your name');
            }
        });
    }
    
    // TODO: Connect to WebSocket
}

function joinGame(playerName) {
    console.log('Joining game as:', playerName);
    
    // Hide join panel, show player info
    document.querySelector('.join-panel').style.display = 'none';
    document.getElementById('player-info').style.display = 'block';
    document.getElementById('player-name-display').textContent = playerName;
    
    // TODO: Send join message via WebSocket
}
