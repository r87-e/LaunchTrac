import type * as THREE from 'three';

export interface RangeTheme {
	name: string;
	description: string;
	setup: (scene: THREE.Scene, camera: THREE.PerspectiveCamera) => {
		distanceLabels: { sprite: THREE.Sprite; yards: number }[];
		landingDots: THREE.Group;
	};
	ballColor: number;
	trailColor: number;
	glowColor: number;
	impactColor: number;
	fogColor: number;
	fogDensity: number;
	bloomStrength: number;
	bloomRadius: number;
	bloomThreshold: number;
}
