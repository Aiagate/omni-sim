import { create } from 'zustand';
import { SimulationClient, mapDtoToState } from '../api/SimulationClient';
import type { StarSystem, Fleet, SimEvent } from '../mock/types';

interface SimulationState {
    systems: StarSystem[];
    fleets: Fleet[];
    events: SimEvent[];
    tick: number;
    isRunning: boolean;

    start: () => void;
    stop: () => void;

    // Internal
    client: SimulationClient | null;
}

export const useSimulationStore = create<SimulationState>((set, get) => {
    // Initialize Client Handler
    const handleServerMessage = (dto: any) => {
        const state = mapDtoToState(dto);
        set({
            systems: state.systems,
            fleets: state.fleets,
            events: state.events,
            tick: state.tick
        });
    };

    const client = new SimulationClient(handleServerMessage);

    return {
        systems: [],
        fleets: [],
        events: [],
        tick: 0,
        isRunning: false,
        client,

        start: () => {
            const { isRunning, client } = get();
            if (isRunning) return;

            console.log('Starting LIVE connection');
            client?.connect();
            set({ isRunning: true });
        },

        stop: () => {
            const { client } = get();
            client?.disconnect();
            set({ isRunning: false });
        }
    };
});
