import { useEffect } from 'react';
import { Layout } from './components/Layout';
import { GlobalHeader } from './components/GlobalHeader';
import { TacticalMap } from './components/TacticalMap';
import { HUDOverlay } from './components/HUDOverlay';
import { useSimulationStore } from './store/useSimulationStore';

function App() {
  const { start, stop } = useSimulationStore();

  useEffect(() => {
    start();
    return () => stop();
  }, [start, stop]);

  return (
    <Layout>
      <GlobalHeader />
      <div className="absolute inset-0 z-0">
        <TacticalMap />
      </div>



      <HUDOverlay />
    </Layout>
  );
}

export default App;
