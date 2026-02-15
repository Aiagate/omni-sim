import React, { useEffect, useRef } from 'react';
import { useSimulationStore } from '../store/useSimulationStore';
import { AnimatePresence, motion } from 'framer-motion';

export const HUDOverlay: React.FC = () => {
    const { events, systems } = useSimulationStore();
    const logContainerRef = useRef<HTMLDivElement>(null);

    // Auto-scroll log
    useEffect(() => {
        if (logContainerRef.current) {
            logContainerRef.current.scrollTop = 0;
        }
    }, [events]);

    return (
        <div className="absolute inset-0 pointer-events-none z-20 flex flex-col justify-between p-6">
            <div className="flex h-full gap-4">
                {/* Left Side: System & Planet List */}
                <div className="w-[300px] h-full bg-[#020205]/80 border-r-2 border-[#FF8C00] p-4 backdrop-blur-sm pointer-events-auto overflow-y-auto scrollbar-hide flex flex-col gap-4">
                    <h3 className="text-xs text-[#4A4A6A] font-bold tracking-wider mb-2 sticky top-0 bg-[#020205]/90 py-2 border-b border-[#4A4A6A]">STAR SYSTEMS</h3>
                    {systems.map(system => (
                        <div key={system.id} className="flex flex-col gap-1 border border-[#4A4A6A]/30 p-2 rounded hover:bg-[#4A4A6A]/10 transition-colors">
                            <div className="flex justify-between items-center">
                                <span className="text-[#00F0FF] font-bold">{system.name}</span>
                                <span className="text-[10px] text-[#4A4A6A]">{system.faction}</span>
                            </div>
                            <div className="text-[10px] text-[#4A4A6A]">
                                Pop: {(system.population / 1000000).toFixed(1)}M
                            </div>

                            {/* Planet List */}
                            {system.planets && system.planets.length > 0 && (
                                <div className="mt-2 pl-2 border-l border-[#4A4A6A]/30 flex flex-col gap-1">
                                    {system.planets.map(planet => (
                                        <div key={planet.id} className="flex justify-between items-center text-[10px]">
                                            <span className="text-[#FF8C00]">{planet.name}</span>
                                            <span className="text-[#4A4A6A]">{planet.type_name}</span>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </div>
                    ))}
                </div>

                <div className="flex-1 flex flex-col justify-end">
                    <div className="flex justify-between items-end w-full">

                        {/* Left Bottom: Status / Decoration */}
                        <div className="bg-[#020205]/80 border-l-2 border-[#FF8C00] p-4 backdrop-blur-sm pointer-events-auto">
                            <h3 className="text-xs text-[#4A4A6A] mb-1">SYSTEM DIAGNOSTICS</h3>
                            <div className="flex gap-2 text-sm">
                                <div className="flex flex-col">
                                    <span className="text-[#00F0FF]">CPU: OPTIMAL</span>
                                    <span className="text-[#00F0FF]">MEM: STABLE</span>
                                </div>
                                <div className="w-[100px] h-full border border-[#4A4A6A] relative overflow-hidden">
                                    <div className="absolute inset-0 bg-[#FF8C00] opacity-20 animate-pulse w-[70%]" />
                                </div>
                            </div>
                        </div>

                        {/* Right Bottom: Log Console */}
                        <div className="w-[400px] h-[200px] bg-[#020205]/90 border border-[#4A4A6A] border-t-4 border-t-[#FF8C00] backdrop-blur-md pointer-events-auto flex flex-col">
                            <div className="bg-[#4A4A6A]/20 p-1 px-2 flex justify-between items-center border-b border-[#4A4A6A]">
                                <span className="text-xs font-bold tracking-wider">EVENT LOG</span>
                                <div className="flex gap-1">
                                    <div className="w-2 h-2 bg-[#FF8C00] rounded-full animate-pulse" />
                                </div>
                            </div>

                            <div ref={logContainerRef} className="flex-1 overflow-y-auto p-2 font-[Fira_Code] text-xs flex flex-col gap-1 scrollbar-hide">
                                <AnimatePresence initial={false}>
                                    {events.slice(0, 20).map(event => (
                                        <motion.div
                                            key={event.id}
                                            initial={{ opacity: 0, x: 20 }}
                                            animate={{ opacity: 1, x: 0 }}
                                            exit={{ opacity: 0 }}
                                            className={`flex gap-2 ${event.severity === 'critical' ? 'text-[#FF2A2A]' : 'text-[#FF8C00]'}`}
                                        >
                                            <span className="opacity-50">[{new Date(event.timestamp).toLocaleTimeString().split(' ')[0]}]</span>
                                            <span>{event.message}</span>
                                        </motion.div>
                                    ))}
                                </AnimatePresence>
                                {events.length === 0 && <span className="text-[#4A4A6A] italic">... NO EVENTS ...</span>}
                            </div>
                        </div>

                    </div>
                </div>
            </div>
        </div>
    );
};
