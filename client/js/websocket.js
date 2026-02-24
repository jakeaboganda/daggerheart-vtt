// WebSocket Client Module

class WebSocketClient {
    constructor(onMessage) {
        this.ws = null;
        this.onMessage = onMessage;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
    }

    connect() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        
        console.log('Connecting to WebSocket:', wsUrl);
        
        this.ws = new WebSocket(wsUrl);
        
        this.ws.onopen = () => {
            console.log('✅ WebSocket connected');
            this.reconnectAttempts = 0;
            
            // Send initial connect message to get connection ID (no payload)
            this.send('connect');
        };
        
        this.ws.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                console.log('📨 Received:', message);
                if (this.onMessage) {
                    this.onMessage(message);
                }
            } catch (e) {
                console.error('Failed to parse message:', e);
            }
        };
        
        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };
        
        this.ws.onclose = () => {
            console.log('WebSocket disconnected');
            this.attemptReconnect();
        };
    }

    attemptReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            console.log(`Reconnecting... (${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
            setTimeout(() => this.connect(), 2000);
        } else {
            console.error('Max reconnection attempts reached');
        }
    }

    send(type, payload) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            // For unit variants (no payload), only send type
            const message = payload === undefined || payload === null 
                ? { type } 
                : { type, payload };
            console.log('📤 Sending:', message);
            const jsonString = JSON.stringify(message);
            console.log('📤 JSON:', jsonString);
            this.ws.send(jsonString);
        } else {
            console.warn('WebSocket not connected');
        }
    }

    disconnect() {
        if (this.ws) {
            this.ws.close();
        }
    }
}
