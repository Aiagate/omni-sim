export type Vector3 = [number, number, number];

export interface StarSystem {
    id: string;
    name: string;
    position: Vector3;
    faction: 'sirius' | 'earth' | 'centauri' | 'neutral';
    population: number;
    planets: Planet[];
}

export interface Planet {
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

export interface Fleet {
    id: string;
    name: string;
    faction: 'sirius' | 'earth' | 'centauri';
    position: Vector3;
    destinationId: string;
    originId: string;
    progress: number; // 0 to 1
    status: 'idle' | 'moving' | 'combat';
}

export interface SimEvent {
    id: string;
    timestamp: number;
    type: 'BATTLE' | 'TRADE' | 'DIPLOMACY' | 'SYSTEM';
    message: string;
    severity: 'info' | 'warning' | 'critical';
}
