import { GalaxyGenerator } from './GalaxyGenerator';
import type { StarSystem, Fleet, SimEvent } from './types';

export class SimulationService {
    systems: StarSystem[] = [];
    fleets: Fleet[] = [];
    events: SimEvent[] = [];
    tick: number = 0;
    private generator = new GalaxyGenerator();

    constructor() {
        this.systems = this.generator.generateSystems(100, 200);
    }

    update() {
        this.tick++;
        this.updateFleets();
        this.spawnEvents();
    }

    private updateFleets() {
        // Example: Move fleets, spawn new ones periodically
        if (this.tick % 60 === 0 && this.fleets.length < 20) {
            this.spawnFleet();
        }

        this.fleets = this.fleets.map((fleet): Fleet => {
            if (fleet.status !== 'moving') return fleet;

            const newProgress = fleet.progress + 0.01;
            if (newProgress >= 1) {
                return { ...fleet, progress: 1, status: 'idle' } as Fleet;
            }
            return { ...fleet, progress: newProgress } as Fleet;
        }).filter(f => f.status !== 'idle'); // Remove arrived fleets for now
    }

    private spawnFleet() {
        const origin = this.systems[Math.floor(Math.random() * this.systems.length)];
        const destination = this.systems[Math.floor(Math.random() * this.systems.length)];
        if (origin.id === destination.id) return;

        this.fleets.push(this.generator.generateFleet(origin, destination));
    }

    private spawnEvents() {
        if (Math.random() < 0.05) {
            const type = ['BATTLE', 'TRADE', 'DIPLOMACY', 'SYSTEM'][Math.floor(Math.random() * 4)] as any;
            this.events.unshift({
                id: crypto.randomUUID(),
                timestamp: Date.now(),
                type,
                message: `Event ${type} occurred at sector ${Math.floor(Math.random() * 999)}`,
                severity: type === 'BATTLE' ? 'critical' : 'info'
            });
            if (this.events.length > 50) this.events.pop();
        }
    }
}
