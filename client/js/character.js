// Character Creation Module
// Phase 3: Daggerheart Integration

const CLASSES = [
    { id: "Bard", name: "Bard", description: "Support and utility specialist" },
    { id: "Druid", name: "Druid", description: "Nature magic and shapeshifting" },
    { id: "Guardian", name: "Guardian", description: "Defensive tank and protector" },
    { id: "Ranger", name: "Ranger", description: "Wilderness expert and tracker" },
    { id: "Rogue", name: "Rogue", description: "Stealth and precision striker" },
    { id: "Seraph", name: "Seraph", description: "Divine magic and healing" },
    { id: "Sorcerer", name: "Sorcerer", description: "Raw magical power" },
    { id: "Warrior", name: "Warrior", description: "Martial combat expert" },
    { id: "Wizard", name: "Wizard", description: "Learned arcane magic" },
];

const ANCESTRIES = [
    "Clank", "Daemon", "Drakona", "Dwarf", "Faerie", "Faun",
    "Fungril", "Galapa", "Giant", "Goblin", "Halfling", "Human",
    "Inferis", "Katari", "Orc", "Ribbet", "Simiah"
];

const ATTRIBUTES = [
    { id: "agility", name: "Agility", description: "Speed and reflexes" },
    { id: "strength", name: "Strength", description: "Physical power" },
    { id: "finesse", name: "Finesse", description: "Precision and grace" },
    { id: "instinct", name: "Instinct", description: "Intuition and awareness" },
    { id: "presence", name: "Presence", description: "Charisma and force of will" },
    { id: "knowledge", name: "Knowledge", description: "Learning and memory" },
];

// Standard attribute distribution: [+2, +1, +1, 0, 0, -1]
const STANDARD_DISTRIBUTION = [2, 1, 1, 0, 0, -1];

class CharacterCreator {
    constructor() {
        this.step = 1;
        this.data = {
            name: "",
            class: null,
            ancestry: null,
            attributes: [0, 0, 0, 0, 0, 0], // agility, strength, finesse, instinct, presence, knowledge
        };
    }
    
    init(containerElement) {
        this.container = containerElement;
        this.render();
    }
    
    render() {
        if (!this.container) return;
        
        switch (this.step) {
            case 1:
                this.renderNameStep();
                break;
            case 2:
                this.renderClassStep();
                break;
            case 3:
                this.renderAncestryStep();
                break;
            case 4:
                this.renderAttributesStep();
                break;
            case 5:
                this.renderConfirmStep();
                break;
        }
    }
    
    renderNameStep() {
        this.container.innerHTML = `
            <div class="character-creation">
                <h2>Create Your Character</h2>
                <p class="step-indicator">Step 1 of 4</p>
                
                <div class="form-group">
                    <label>Character Name</label>
                    <input type="text" id="char-name" placeholder="Enter name" value="${this.data.name}" />
                </div>
                
                <button id="next-btn" class="btn-primary">Next</button>
            </div>
        `;
        
        document.getElementById('next-btn').addEventListener('click', () => {
            const name = document.getElementById('char-name').value.trim();
            if (name) {
                this.data.name = name;
                this.step = 2;
                this.render();
            } else {
                alert('Please enter a character name');
            }
        });
    }
    
    renderClassStep() {
        this.container.innerHTML = `
            <div class="character-creation">
                <h2>Choose Your Class</h2>
                <p class="step-indicator">Step 2 of 4</p>
                
                <div class="class-grid" id="class-grid">
                    ${CLASSES.map(cls => `
                        <div class="class-card" data-class="${cls.id}">
                            <h3>${cls.name}</h3>
                            <p>${cls.description}</p>
                        </div>
                    `).join('')}
                </div>
                
                <div class="button-group">
                    <button id="back-btn" class="btn-secondary">Back</button>
                </div>
            </div>
        `;
        
        document.querySelectorAll('.class-card').forEach(card => {
            card.addEventListener('click', () => {
                this.data.class = card.dataset.class;
                this.step = 3;
                this.render();
            });
        });
        
        document.getElementById('back-btn').addEventListener('click', () => {
            this.step = 1;
            this.render();
        });
    }
    
    renderAncestryStep() {
        this.container.innerHTML = `
            <div class="character-creation">
                <h2>Choose Your Ancestry</h2>
                <p class="step-indicator">Step 3 of 4</p>
                
                <div class="ancestry-grid" id="ancestry-grid">
                    ${ANCESTRIES.map(ancestry => `
                        <div class="ancestry-card" data-ancestry="${ancestry}">
                            <h3>${ancestry}</h3>
                        </div>
                    `).join('')}
                </div>
                
                <div class="button-group">
                    <button id="back-btn" class="btn-secondary">Back</button>
                </div>
            </div>
        `;
        
        document.querySelectorAll('.ancestry-card').forEach(card => {
            card.addEventListener('click', () => {
                this.data.ancestry = card.dataset.ancestry;
                this.step = 4;
                this.render();
            });
        });
        
        document.getElementById('back-btn').addEventListener('click', () => {
            this.step = 2;
            this.render();
        });
    }
    
    renderAttributesStep() {
        // Pre-fill with standard distribution if empty
        if (this.data.attributes.every(v => v === 0)) {
            this.data.attributes = [...STANDARD_DISTRIBUTION];
        }
        
        this.container.innerHTML = `
            <div class="character-creation">
                <h2>Assign Attributes</h2>
                <p class="step-indicator">Step 4 of 4</p>
                <p class="hint">Assign: +2, +1, +1, 0, 0, -1</p>
                
                <div class="attributes-list" id="attributes-list">
                    ${ATTRIBUTES.map((attr, idx) => `
                        <div class="attribute-row">
                            <span class="attr-name">${attr.name}</span>
                            <select class="attr-select" data-index="${idx}">
                                <option value="2" ${this.data.attributes[idx] === 2 ? 'selected' : ''}>+2</option>
                                <option value="1" ${this.data.attributes[idx] === 1 ? 'selected' : ''}>+1</option>
                                <option value="0" ${this.data.attributes[idx] === 0 ? 'selected' : ''}>0</option>
                                <option value="-1" ${this.data.attributes[idx] === -1 ? 'selected' : ''}>-1</option>
                            </select>
                        </div>
                    `).join('')}
                </div>
                
                <div class="button-group">
                    <button id="back-btn" class="btn-secondary">Back</button>
                    <button id="create-btn" class="btn-primary">Create Character</button>
                </div>
            </div>
        `;
        
        document.querySelectorAll('.attr-select').forEach(select => {
            select.addEventListener('change', (e) => {
                const idx = parseInt(e.target.dataset.index);
                this.data.attributes[idx] = parseInt(e.target.value);
            });
        });
        
        document.getElementById('back-btn').addEventListener('click', () => {
            this.step = 3;
            this.render();
        });
        
        document.getElementById('create-btn').addEventListener('click', () => {
            if (this.validateAttributes()) {
                this.step = 5;
                this.render();
            } else {
                alert('Invalid attribute distribution! Must be exactly: +2, +1, +1, 0, 0, -1');
            }
        });
    }
    
    validateAttributes() {
        const sorted = [...this.data.attributes].sort((a, b) => b - a);
        const expected = [...STANDARD_DISTRIBUTION].sort((a, b) => b - a);
        return sorted.every((val, idx) => val === expected[idx]);
    }
    
    renderConfirmStep() {
        this.container.innerHTML = `
            <div class="character-creation">
                <h2>Confirm Character</h2>
                
                <div class="character-summary">
                    <p><strong>Name:</strong> ${this.data.name}</p>
                    <p><strong>Class:</strong> ${this.data.class}</p>
                    <p><strong>Ancestry:</strong> ${this.data.ancestry}</p>
                    <h3>Attributes:</h3>
                    <ul>
                        ${ATTRIBUTES.map((attr, idx) => `
                            <li>${attr.name}: ${this.data.attributes[idx] >= 0 ? '+' : ''}${this.data.attributes[idx]}</li>
                        `).join('')}
                    </ul>
                </div>
                
                <div class="button-group">
                    <button id="back-btn" class="btn-secondary">Back</button>
                    <button id="confirm-btn" class="btn-primary">Confirm & Create</button>
                </div>
            </div>
        `;
        
        document.getElementById('back-btn').addEventListener('click', () => {
            this.step = 4;
            this.render();
        });
        
        document.getElementById('confirm-btn').addEventListener('click', () => {
            this.createCharacter();
        });
    }
    
    createCharacter() {
        // Send to server
        console.log('Creating character:', this.data);
        
        if (window.ws) {
            console.log('Sending create_character message...');
            window.ws.send('create_character', {
                name: this.data.name,
                class: this.data.class,
                ancestry: this.data.ancestry,
                attributes: this.data.attributes,
            });
        } else {
            console.error('WebSocket not connected!');
            alert('Error: Not connected to server. Please refresh and try again.');
        }
    }
}
