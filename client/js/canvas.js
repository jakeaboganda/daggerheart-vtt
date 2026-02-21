// Canvas Rendering Module
// Phase 5A: Character-Centric Architecture

const MAP_WIDTH = 800;
const MAP_HEIGHT = 600;
const PLAYER_RADIUS = 20;

class MapCanvas {
    constructor(canvasId) {
        this.canvas = document.getElementById(canvasId);
        if (!this.canvas) {
            console.error('Canvas element not found:', canvasId);
            return;
        }
        
        this.ctx = this.canvas.getContext('2d');
        this.players = new Map(); // character_id -> character data (keeping "players" var name for compatibility)
        this.animating = new Map(); // character_id -> animation state
        
        // Set canvas size
        this.canvas.width = MAP_WIDTH;
        this.canvas.height = MAP_HEIGHT;
        
        // Start render loop
        this.startRenderLoop();
        
        console.log('MapCanvas initialized:', MAP_WIDTH, 'x', MAP_HEIGHT);
    }
    
    addPlayer(playerId, name, position, color) {
        this.players.set(playerId, {
            id: playerId,
            name,
            position: { ...position },
            targetPosition: { ...position },
            color
        });
        console.log('Added player to canvas:', name, position);
    }
    
    removePlayer(playerId) {
        this.players.delete(playerId);
        this.animating.delete(playerId);
    }
    
    updatePlayerName(playerId, newName) {
        const player = this.players.get(playerId);
        if (player) {
            console.log(`MapCanvas: Updating player ${playerId} name: "${player.name}" → "${newName}"`);
            player.name = newName;
        } else {
            console.warn(`MapCanvas: Player ${playerId} not found in canvas!`);
        }
    }
    
    updatePlayerPosition(playerId, newPosition) {
        const player = this.players.get(playerId);
        if (player) {
            player.targetPosition = { ...newPosition };
            
            // Start animation
            this.animating.set(playerId, {
                startTime: Date.now(),
                duration: 500, // ms
                startPos: { ...player.position }
            });
        }
    }
    
    clearPlayers() {
        this.players.clear();
        this.animating.clear();
    }
    
    startRenderLoop() {
        const render = () => {
            this.render();
            requestAnimationFrame(render);
        };
        requestAnimationFrame(render);
    }
    
    render() {
        if (!this.ctx) return;
        
        // Clear canvas
        this.ctx.fillStyle = '#1a1a1a';
        this.ctx.fillRect(0, 0, MAP_WIDTH, MAP_HEIGHT);
        
        // Draw grid (subtle)
        this.drawGrid();
        
        // Update animations and render players
        const now = Date.now();
        for (const [playerId, player] of this.players) {
            // Check if animating
            const anim = this.animating.get(playerId);
            if (anim) {
                const elapsed = now - anim.startTime;
                const progress = Math.min(elapsed / anim.duration, 1.0);
                
                // Ease out cubic
                const eased = 1 - Math.pow(1 - progress, 3);
                
                // Interpolate position
                player.position.x = anim.startPos.x + (player.targetPosition.x - anim.startPos.x) * eased;
                player.position.y = anim.startPos.y + (player.targetPosition.y - anim.startPos.y) * eased;
                
                // Remove animation when done
                if (progress >= 1.0) {
                    this.animating.delete(playerId);
                }
            }
            
            this.drawPlayer(player);
        }
    }
    
    drawGrid() {
        const gridSize = 50;
        this.ctx.strokeStyle = '#2a2a2a';
        this.ctx.lineWidth = 1;
        
        // Vertical lines
        for (let x = 0; x <= MAP_WIDTH; x += gridSize) {
            this.ctx.beginPath();
            this.ctx.moveTo(x, 0);
            this.ctx.lineTo(x, MAP_HEIGHT);
            this.ctx.stroke();
        }
        
        // Horizontal lines
        for (let y = 0; y <= MAP_HEIGHT; y += gridSize) {
            this.ctx.beginPath();
            this.ctx.moveTo(0, y);
            this.ctx.lineTo(MAP_WIDTH, y);
            this.ctx.stroke();
        }
    }
    
    drawPlayer(player) {
        const { position, color, name } = player;
        
        // Draw glow
        this.ctx.shadowBlur = 15;
        this.ctx.shadowColor = color;
        
        // Draw circle
        this.ctx.fillStyle = color;
        this.ctx.beginPath();
        this.ctx.arc(position.x, position.y, PLAYER_RADIUS, 0, Math.PI * 2);
        this.ctx.fill();
        
        // Draw border
        this.ctx.strokeStyle = '#ffffff';
        this.ctx.lineWidth = 2;
        this.ctx.stroke();
        
        // Reset shadow
        this.ctx.shadowBlur = 0;
        
        // Draw name label
        this.ctx.fillStyle = '#ffffff';
        this.ctx.font = 'bold 14px sans-serif';
        this.ctx.textAlign = 'center';
        this.ctx.textBaseline = 'bottom';
        
        // Text shadow for readability
        this.ctx.shadowColor = '#000000';
        this.ctx.shadowBlur = 4;
        
        this.ctx.fillText(name, position.x, position.y - PLAYER_RADIUS - 5);
        
        // Reset shadow
        this.ctx.shadowBlur = 0;
    }
    
    // Get canvas click position (for mobile tap)
    getClickPosition(event) {
        const rect = this.canvas.getBoundingClientRect();
        const scaleX = this.canvas.width / rect.width;
        const scaleY = this.canvas.height / rect.height;
        
        return {
            x: (event.clientX - rect.left) * scaleX,
            y: (event.clientY - rect.top) * scaleY
        };
    }
}
