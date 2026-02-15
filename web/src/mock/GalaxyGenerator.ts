import type { StarSystem, Fleet } from './types';

const FACTIONS = ['sirius', 'earth', 'centauri', 'neutral'] as const;
const SYSTEM_NAMES = [
    'Alpha', 'Beta', 'Gamma', 'Delta', 'Epsilon', 'Zeta', 'Eta', 'Theta',
    'Iota', 'Kappa', 'Lambda', 'Mu', 'Nu', 'Xi', 'Omicron', 'Pi',
    'Rho', 'Sigma', 'Tau', 'Upsilon', 'Phi', 'Chi', 'Psi', 'Omega',
    'Sol', 'Proxima', 'Wolf', 'Luyten', 'Ross', 'Barnard', 'Sirius'
];

export class GalaxyGenerator {
    generateSystems(count: number = 50, radius: number = 100): StarSystem[] {
        return Array.from({ length: count }).map(() => {
            const theta = Math.random() * Math.PI * 2;
            const phi = Math.acos((Math.random() * 2) - 1);
            const r = Math.cbrt(Math.random()) * radius; // Uniform distribution in sphere

            const x = r * Math.sin(phi) * Math.cos(theta);
            const y = r * Math.sin(phi) * Math.sin(theta);
            const z = r * Math.cos(phi);

            const numPlanets = Math.floor(Math.random() * 5); // 0-4 planets
            const planets = Array.from({ length: numPlanets }).map((_, i) => ({
                id: crypto.randomUUID(),
                name: `Planet-${['I', 'II', 'III', 'IV', 'V'][i] || i + 1}`,
                orbit_distance: 0.5 + i * 1.5,
                angle: Math.random() * Math.PI * 2,
                size: 0.2 + Math.random() * 0.3,
                color: '#' + Math.floor(Math.random() * 16777215).toString(16),
                faction: FACTIONS[Math.floor(Math.random() * FACTIONS.length)],
                population: Math.floor(Math.random() * 1000000),
                type_name: ['Earth-like', 'Gas Giant', 'Ice World', 'Barren'][Math.floor(Math.random() * 4)],
                owner_id: null
            }));

            return {
                id: crypto.randomUUID(),
                name: `${SYSTEM_NAMES[Math.floor(Math.random() * SYSTEM_NAMES.length) % SYSTEM_NAMES.length]}-${Math.floor(Math.random() * 999)}`,
                position: [x, y, z],
                faction: FACTIONS[Math.floor(Math.random() * FACTIONS.length)],
                population: Math.floor(Math.random() * 10000000),
                planets
            };
        });
    }

    generateFleet(origin: StarSystem, destination: StarSystem): Fleet {
        return {
            id: crypto.randomUUID(),
            name: `FLEET-${Math.floor(Math.random() * 9999)}`,
            faction: origin.faction === 'neutral' ? 'earth' : origin.faction as any,
            position: [...origin.position],
            originId: origin.id,
            destinationId: destination.id,
            progress: 0,
            status: 'moving'
        };
    }
}
