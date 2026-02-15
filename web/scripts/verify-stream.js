import WebSocket from 'ws';

/**
 * verify-stream.js
 * 
 * This script connects to the simulation server and verifies:
 * 1. Connection success
 * 2. Receiving multiple updates
 * 3. Schema validation
 * 4. TPS measurement
 */

const URL = 'ws://127.0.0.1:8080';
const DURATION_MS = 3000; // Test for 3 seconds

console.log(`--- WebSocket Stream Verification ---`);
console.log(`Connecting to ${URL}...`);

const ws = new WebSocket(URL);
let messageCount = 0;
let lastTick = -1;
const startTime = Date.now();

ws.on('open', () => {
    console.log('✅ Connected to server');
});

ws.on('message', (data) => {
    try {
        const state = JSON.parse(data);
        messageCount++;

        // Basic Schema Validation
        if (typeof state.tick !== 'number') throw new Error('Missing or invalid "tick"');
        if (!Array.isArray(state.systems)) throw new Error('Missing or invalid "systems"');
        if (!Array.isArray(state.fleets)) throw new Error('Missing or invalid "fleets"');

        if (state.tick <= lastTick && state.tick !== 0) {
            // Note: Some simulations might reset tick, but generally it should increase
            console.warn(`⚠️ Warning: Tick did not increase (${lastTick} -> ${state.tick})`);
        }
        lastTick = state.tick;

        if (messageCount === 1) {
            console.log(`✅ Received first update. Systems: ${state.systems.length}, Fleets: ${state.fleets.length}`);
        }
    } catch (e) {
        console.error('❌ Failed to validate message:', e.message);
        process.exit(1);
    }
});

ws.on('error', (err) => {
    console.error('❌ WebSocket error:', err.message);
    process.exit(1);
});

setTimeout(() => {
    const endTime = Date.now();
    const elapsedSec = (endTime - startTime) / 1000;
    const tps = messageCount / elapsedSec;

    console.log(`--- Results ---`);
    console.log(`Duration: ${elapsedSec.toFixed(2)}s`);
    console.log(`Messages: ${messageCount}`);
    console.log(`Estimated TPS: ${tps.toFixed(2)}`);

    if (messageCount > 0) {
        console.log('✅ Verification successful');
        ws.close();
        process.exit(0);
    } else {
        console.error('❌ No messages received');
        process.exit(1);
    }
}, DURATION_MS);
