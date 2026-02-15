import React, { useEffect, useState } from 'react';
import { useSimulationStore } from '../store/useSimulationStore';

export const GlobalHeader: React.FC = () => {
    const { tick } = useSimulationStore();
    const [timeStr, setTimeStr] = useState('');

    useEffect(() => {
        const updateTime = () => {
            const now = new Date();
            setTimeStr(now.toISOString().split('T')[1].replace('Z', ''));
        };
        const timer = setInterval(updateTime, 57); // Fast update for milliseconds
        return () => clearInterval(timer);
    }, []);

    return (
        <header className="absolute top-0 left-0 w-full p-4 flex justify-between items-center z-50 pointer-events-none select-none">
            <div className="flex items-center gap-4">
                <h1 className="text-2xl font-bold tracking-widest text-[#FF8C00] drop-shadow-[0_0_8px_rgba(255,140,0,0.8)]">
                    OMNI-SIM // TACTICAL VIEW
                </h1>
                <div className="h-6 w-[2px] bg-[#4A4A6A]" />
                <span className="text-sm text-[#4A4A6A]">SECTOR: 001-A</span>
            </div>

            <div className="flex flex-col items-end">
                <div className="flex items-baseline gap-2">
                    <span className="text-xs text-[#4A4A6A]">OP. TIME</span>
                    <span className="text-xl font-medium tabular-nums">{timeStr}</span>
                </div>
                <div className="flex items-baseline gap-2">
                    <span className="text-xs text-[#4A4A6A]">SIM TICK</span>
                    <span className="text-lg text-[#00F0FF] tabular-nums">{tick.toString().padStart(8, '0')}</span>
                </div>
            </div>
        </header>
    );
};
