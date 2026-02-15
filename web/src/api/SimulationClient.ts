import type { StarSystem, Fleet, SimEvent } from '../mock/types';

export interface GameStateDto {
    tick: number;
    systems: StarSystemDto[];
    fleets: FleetDto[];
    events: EventRefDto[];
}

interface StarSystemDto {
    id: String;
    name: String;
    position: [number, number, number];
    faction: string;
    population: number;
    planets: PlanetDto[];
}

interface PlanetDto {
    id: string;
    name: string;
    orbit_distance: number;
    angle: number;
    size: number;
    color: string;
    faction: string;
    population: number;
    type_name: string;
    owner_id: string | null;
}

interface FleetDto {
    id: string;
    name: string;
    faction: string;
    position: [number, number, number];
    destination_id: string;
    origin_id: string;
    progress: number;
    status: string;
}

interface EventRefDto {
    id: string;
    timestamp: number;
    type: string;
    message: string;
    severity: string;
}

type MessageHandler = (state: GameStateDto) => void;

export class SimulationClient {
    private ws: WebSocket | null = null;
    private url: string;
    private onMessage: MessageHandler;

    constructor(onMessage: MessageHandler) {
        this.onMessage = onMessage;
        this.url = `ws://${window.location.hostname}:8080`;
    }

    connect() {
        if (this.ws) {
            this.ws.close();
        }

        console.log(`Connecting to ${this.url}...`);
        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
            console.log('Connected to Simulation Server');
        };

        this.ws.onmessage = (event) => {
            try {
                const state: GameStateDto = JSON.parse(event.data);
                this.onMessage(state);
            } catch (e) {
                console.error('Failed to parse game state:', e);
            }
        };

        this.ws.onclose = () => {
            console.log('Disconnected. Retrying in 3s...');
            setTimeout(() => this.connect(), 3000);
        };

        this.ws.onerror = (err) => {
            console.error('WebSocket error:', err);
            this.ws?.close();
        };
    }

    disconnect() {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
    }
}

// Data Mapper Utility
export const mapDtoToState = (dto: GameStateDto): { systems: StarSystem[], fleets: Fleet[], events: SimEvent[], tick: number } => {
    return {
        tick: dto.tick,
        systems: dto.systems.map(s => ({
            id: s.id as string,
            name: s.name as string,
            position: s.position,
            faction: s.faction as any, // Simple cast for now
            population: s.population,
            planets: s.planets.map(p => ({
                id: p.id,
                name: p.name,
                orbit_distance: p.orbit_distance,
                angle: p.angle,
                size: p.size,
                color: p.color,
                faction: p.faction,
                population: p.population,
                type_name: p.type_name,
                owner_id: p.owner_id
            }))
        })),
        fleets: dto.fleets.map(f => ({
            id: f.id,
            name: f.name,
            faction: f.faction as any,
            position: f.position,
            originId: f.origin_id,
            destinationId: f.destination_id,
            progress: f.progress,
            status: f.status as any
        })),
        events: dto.events.map(e => ({
            id: e.id,
            timestamp: e.timestamp,
            type: e.type as any,
            message: e.message,
            severity: e.severity as any
        }))
    };
};
