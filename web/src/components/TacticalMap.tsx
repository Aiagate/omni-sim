import React, { useMemo } from 'react';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, Stars, PerspectiveCamera } from '@react-three/drei';
import { EffectComposer, Bloom, Noise, Vignette, ChromaticAberration, Scanline } from '@react-three/postprocessing';
import { BlendFunction } from 'postprocessing';
import * as THREE from 'three';
import { useSimulationStore } from '../store/useSimulationStore';
import type { StarSystem, Fleet, Vector3 } from '../mock/types';

// Constants for styling
const FACTION_COLORS: Record<string, string> = {
    sirius: '#FF0000',
    earth: '#00F0FF',
    centauri: '#00FF00',
    neutral: '#444444'
};

const StarNode: React.FC<{ system: StarSystem }> = ({ system }) => {
    const color = FACTION_COLORS[system.faction] || '#FFFFFF';

    return (
        <group position={system.position}>
            {/* Core glow */}
            <mesh>
                <sphereGeometry args={[1.5, 16, 16]} />
                <meshBasicMaterial color={color} toneMapped={false} />
            </mesh>
            {/* Outer ring */}
            <mesh>
                <ringGeometry args={[2.5, 2.6, 32]} />
                <meshBasicMaterial color={color} opacity={0.3} transparent side={THREE.DoubleSide} toneMapped={false} />
            </mesh>

            {/* Planets */}
            {system.planets?.map(planet => (
                <group key={planet.id} rotation={[0, planet.angle, 0]}>
                    {/* Orbit Ring */}
                    <mesh rotation={[Math.PI / 2, 0, 0]}>
                        <ringGeometry args={[planet.orbit_distance - 0.2, planet.orbit_distance + 0.2, 64]} />
                        <meshBasicMaterial color="#444" opacity={0.3} transparent side={THREE.DoubleSide} />
                    </mesh>

                    {/* Planet Body */}
                    <mesh position={[planet.orbit_distance, 0, 0]}>
                        <sphereGeometry args={[planet.size * 5, 16, 16]} />
                        <meshStandardMaterial color={planet.color} />
                    </mesh>
                </group>
            ))}
        </group>
    );
};

const FleetMarker: React.FC<{ fleet: Fleet; systems: StarSystem[] }> = ({ fleet, systems }) => {
    const startSys = systems.find(s => s.id === fleet.originId);
    const endSys = systems.find(s => s.id === fleet.destinationId);

    // Linear interpolation for position
    const position = useMemo(() => {
        if (!startSys || !endSys) return [0, 0, 0] as Vector3;
        const [x1, y1, z1] = startSys.position;
        const [x2, y2, z2] = endSys.position;
        return [
            x1 + (x2 - x1) * fleet.progress,
            y1 + (y2 - y1) * fleet.progress,
            z1 + (z2 - z1) * fleet.progress
        ] as Vector3;
    }, [fleet.progress, startSys, endSys]);

    if (!startSys || !endSys) return null;

    const points = useMemo(() => {
        return new Float32Array([...startSys.position, ...endSys.position]);
    }, [startSys, endSys]);

    return (
        <group position={position}>
            {/* Fleet geometry (Tetrahedron) */}
            <mesh rotation={[0, 0, Math.PI / 2]}> {/* TODO: Orient to velocity */}
                <tetrahedronGeometry args={[1.2]} />
                <meshBasicMaterial color="#FFDD00" toneMapped={false} />
            </mesh>

            {/* Trajectory Line - simplified */}
            <line>
                <bufferGeometry>
                    <bufferAttribute attach="attributes-position" count={2} args={[points, 3]} />
                </bufferGeometry>
                <lineBasicMaterial color="#333333" transparent opacity={0.5} />
            </line>
        </group>
    );
};

// Scene Content to access store within Canvas
const SimulationScene: React.FC = () => {
    const { systems, fleets } = useSimulationStore();

    return (
        <>
            <ambientLight intensity={0.1} />
            <Stars radius={300} depth={50} count={5000} factor={4} saturation={0} fade speed={1} />

            <group>
                {systems.map(sys => <StarNode key={sys.id} system={sys} />)}
                {fleets.map(f => <FleetMarker key={f.id} fleet={f} systems={systems} />)}
            </group>
        </>
    );
};

export const TacticalMap: React.FC = () => {
    return (
        <div className="absolute inset-0 w-full h-full z-0">
            <Canvas gl={{ antialias: false, toneMapping: THREE.NoToneMapping }}>
                <PerspectiveCamera makeDefault position={[0, 100, 200]} fov={60} />
                <OrbitControls enablePan={true} enableZoom={true} enableRotate={true} />

                <SimulationScene />

                <EffectComposer autoClear={false} enabled={true}>
                    <Bloom luminanceThreshold={0} mipmapBlur intensity={1.5} radius={0.5} />
                    <Noise opacity={0.1} blendFunction={BlendFunction.OVERLAY} />
                    <ChromaticAberration offset={new THREE.Vector2(0.002, 0.002)} radialModulation={false} modulationOffset={0} />
                    <Scanline density={1.5} opacity={0.1} />
                    <Vignette eskil={false} offset={0.1} darkness={0.5} />
                </EffectComposer>
            </Canvas>
        </div>
    );
};
