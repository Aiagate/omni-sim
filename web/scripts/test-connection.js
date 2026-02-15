import WebSocket from 'ws';

const ws = new WebSocket('ws://127.0.0.1:8080');

ws.on('open', () => {
    console.log('SUCCESS: Connected to mock server');
});

ws.on('message', (data) => {
    const state = JSON.parse(data);
    console.log(`SUCCESS: Received state update. Tick: ${state.tick}, Systems: ${state.systems.length}, Fleets: ${state.fleets.length}`);
    ws.close();
    process.exit(0);
});

ws.on('error', (err) => {
    console.error('FAILURE: Could not connect to mock server', err);
    process.exit(1);
});

setTimeout(() => {
    console.error('FAILURE: Timeout waiting for message');
    process.exit(1);
}, 5000);
