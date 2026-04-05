import { writable, derived } from 'svelte/store';
import type { ShotData, DeviceStatus } from '$lib/types';

export const shots = writable<ShotData[]>([]);
export const lastShot = writable<ShotData | null>(null);
export const deviceStatus = writable<DeviceStatus>({
	connected: false,
	version: '0.1.0',
	ball_detected: false,
	simulator_connected: null,
	uptime_seconds: 0
});

export const sessionStats = derived(shots, ($shots) => {
	if ($shots.length === 0) {
		return { shot_count: 0, avg_speed: 0, avg_vla: 0, avg_backspin: 0, avg_carry_estimate: 0 };
	}

	const count = $shots.length;
	const avg = (arr: number[]) => arr.reduce((a, b) => a + b, 0) / count;

	return {
		shot_count: count,
		avg_speed: avg($shots.map((s) => s.speed_mph)),
		avg_vla: avg($shots.map((s) => s.vla_deg)),
		avg_backspin: avg($shots.map((s) => s.backspin_rpm)),
		avg_carry_estimate: avg($shots.map((s) => estimateCarry(s)))
	};
});

function estimateCarry(shot: ShotData): number {
	// Simplified carry distance estimation (yards)
	// Real formula would use full trajectory simulation
	const speedFactor = shot.speed_mph * 1.5;
	const angleFactor = Math.sin((2 * shot.vla_deg * Math.PI) / 180);
	return Math.round(speedFactor * angleFactor * 0.9);
}

// WebSocket connection management
let ws: WebSocket | null = null;

export function connectWebSocket(url: string) {
	if (ws) ws.close();

	ws = new WebSocket(url);

	ws.onopen = () => {
		deviceStatus.update((s) => ({ ...s, connected: true }));
		console.log('WebSocket connected');
	};

	ws.onmessage = (event) => {
		try {
			const shot: ShotData = JSON.parse(event.data);
			lastShot.set(shot);
			shots.update((s) => [shot, ...s].slice(0, 500)); // Keep last 500 shots
		} catch (e) {
			console.error('Failed to parse shot data:', e);
		}
	};

	ws.onclose = () => {
		deviceStatus.update((s) => ({ ...s, connected: false }));
		console.log('WebSocket disconnected, reconnecting in 3s...');
		setTimeout(() => connectWebSocket(url), 3000);
	};

	ws.onerror = (e) => {
		console.error('WebSocket error:', e);
	};
}

export function disconnectWebSocket() {
	if (ws) {
		ws.close();
		ws = null;
	}
}
