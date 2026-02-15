import { WebSocketServer } from 'ws';
import { randomUUID } from 'crypto';

const PORT = 8080;
const TPS = 10;

// Types and Constants (Ported from GalaxyGenerator.ts)
const FACTIONS = ['sirius', 'earth', 'centauri', 'neutral'];
const SYSTEM_NAMES = [
    'Alpha', 'Beta', 'Gamma', 'Delta', 'Epsilon', 'Zeta', 'Eta', 'Theta',
    'Iota', 'Kappa', 'Lambda', 'Mu', 'Nu', 'Xi', 'Omicron', 'Pi',
    'Rho', 'Sigma', 'Tau', 'Upsilon', 'Phi', 'Chi', 'Psi', 'Omega',
    'Sol', 'Proxima', 'Wolf', 'Luyten', 'Ross', 'Barnard', 'Sirius'
];

// --- Generator Logic ---

function generateSystems(count = 50, radius = 100) {
    return Array.from({ length: count }).map(() => {
        const theta = Math.random() * Math.PI * 2;
        const phi = Math.acos((Math.random() * 2) - 1);
        const r = Math.cbrt(Math.random()) * radius;

        const x = r * Math.sin(phi) * Math.cos(theta);
        const y = r * Math.sin(phi) * Math.sin(theta);
        const z = r * Math.cos(phi);

        return {
            id: randomUUID(),
            name: `${SYSTEM_NAMES[Math.floor(Math.random() * SYSTEM_NAMES.length)]}-${Math.floor(Math.random() * 999)}`,
            position: [x, y, z],
            faction: FACTIONS[Math.floor(Math.random() * FACTIONS.length)],
            population: Math.floor(Math.random() * 10000000)
        };
    });
}

function generateFleet(origin, destination) {
    return {
        id: randomUUID(),
        name: `FLEET-${Math.floor(Math.random() * 9999)}`,
        faction: origin.faction === 'neutral' ? 'earth' : origin.faction,
        position: [...origin.position],
        start_position: [...origin.position], // Helper for interpolation
        origin_id: origin.id,
        destination_id: destination.id,
        progress: 0,
        status: 'moving'
    };
}

// --- Simulation State ---

class SimulationService {
    constructor() {
        this.tick = 0;
        this.systems = generateSystems();
        this.fleets = [];
        this.events = [];
    }

    update() {
        this.tick++;

        // 1. Generate Fleets randomly
        if (Math.random() < 0.05) { // 5% chance per tick
            const origin = this.systems[Math.floor(Math.random() * this.systems.length)];
            const destination = this.systems[Math.floor(Math.random() * this.systems.length)];
            if (origin.id !== destination.id) {
                const fleet = generateFleet(origin, destination);
                this.fleets.push(fleet);
                this.logEvent('info', `Fleet ${fleet.name} launched from ${origin.name} to ${destination.name}`);
            }
        }

        // 2. Move Fleets
        // Simple lerp for visual effect
        const speed = 0.01;
        this.fleets.forEach(fleet => {
            if (fleet.status === 'moving') {
                fleet.progress += speed;

                // Calculate position based on progress
                const originSys = this.systems.find(s => s.id === fleet.origin_id);
                const destSys = this.systems.find(s => s.id === fleet.destination_id);

                if (originSys && destSys) {
                    // We don't have the original start pos easily available unless we store it, 
                    // but for a mock, lerping between the looked-up systems is fine.
                    // Actually, let's use the helper 'start_position' I added.
                    const start = fleet.start_position || originSys.position;
                    const end = destSys.position;

                    fleet.position[0] = start[0] + (end[0] - start[0]) * fleet.progress;
                    fleet.position[1] = start[1] + (end[1] - start[1]) * fleet.progress;
                    fleet.position[2] = start[2] + (end[2] - start[2]) * fleet.progress;
                }

                if (fleet.progress >= 1) {
                    fleet.status = 'arrived';
                    this.logEvent('success', `Fleet ${fleet.name} arrived at destination`);
                }
            }
        });

        // 3. Cleanup Arrived Fleets
        this.fleets = this.fleets.filter(f => f.status !== 'arrived');
    }

    logEvent(severity, message) {
        this.events.unshift({
            id: randomUUID(),
            timestamp: Date.now(),
            type: 'general',
            message,
            severity
        });
        if (this.events.length > 50) this.events.pop();
    }

    getState() {
        return {
            tick: this.tick,
            systems: this.systems,
            fleets: this.fleets,
            events: this.events
        };
    }
}

// --- Server Setup ---

const wss = new WebSocketServer({ port: PORT, host: '0.0.0.0' });
const sim = new SimulationService();

console.log(`Mock Simulation Server running on ws://localhost:${PORT}`);

// Simulation Loop
setInterval(() => {
    sim.update();
    const state = sim.getState();
    const message = JSON.stringify(state);

    wss.clients.forEach(client => {
        if (client.readyState === 1) { // WebSocket.OPEN is 1
            client.send(message);
        }
    });
}, 1000 / TPS);

wss.on('connection', (ws) => {
    console.log('Client connected');
    ws.send(JSON.stringify(sim.getState())); // Initial state

    ws.on('close', () => {
        console.log('Client disconnected');
    });
});
