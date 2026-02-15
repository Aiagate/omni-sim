import React from 'react';

interface LayoutProps {
    children: React.ReactNode;
}

export const Layout: React.FC<LayoutProps> = ({ children }) => {
    return (
        <div className="relative w-screen h-screen bg-[#020205] overflow-hidden text-[#FF8C00] font-[Share_Tech_Mono]">
            {/* Scanlines Effect */}
            <div className="absolute inset-0 pointer-events-none z-50 mix-blend-overlay opacity-30"
                style={{
                    backgroundImage: 'linear-gradient(rgba(18, 16, 16, 0) 50%, rgba(0, 0, 0, 0.25) 50%), linear-gradient(90deg, rgba(255, 0, 0, 0.06), rgba(0, 255, 0, 0.02), rgba(0, 0, 255, 0.06))',
                    backgroundSize: '100% 2px, 2px 100%'
                }}
            />

            {/* Vignette */}
            <div className="absolute inset-0 pointer-events-none z-40 bg-[radial-gradient(circle_at_center,transparent_0%,rgba(0,0,0,0.4)_100%)]" />

            {/* Grid Overlay */}
            <div className="absolute inset-0 pointer-events-none z-30 opacity-10"
                style={{
                    backgroundImage: 'linear-gradient(#4d4d4d 1px, transparent 1px), linear-gradient(90deg, #4d4d4d 1px, transparent 1px)',
                    backgroundSize: '40px 40px'
                }}
            />

            {/* Corner Brackets */}
            <div className="absolute inset-4 pointer-events-none z-50 border-2 border-transparent">
                <div className="absolute top-0 left-0 w-8 h-8 border-t-2 border-l-2 border-[#FF8C00] opacity-80" />
                <div className="absolute top-0 right-0 w-8 h-8 border-t-2 border-r-2 border-[#FF8C00] opacity-80" />
                <div className="absolute bottom-0 left-0 w-8 h-8 border-b-2 border-l-2 border-[#FF8C00] opacity-80" />
                <div className="absolute bottom-0 right-0 w-8 h-8 border-b-2 border-r-2 border-[#FF8C00] opacity-80" />
            </div>

            <div className="relative z-10 w-full h-full">
                {children}
            </div>
        </div>
    );
};
