import * as THREE from 'three';
import type { RangeTheme } from './types';

export const midnight: RangeTheme = {
	name: 'Midnight',
	description: 'Pure black. Minimal grid. The ball is the star.',
	ballColor: 0xffffff,
	trailColor: 0xffffff,
	glowColor: 0xffffff,
	impactColor: 0xffffff,
	fogColor: 0x000000,
	fogDensity: 0.0025,
	bloomStrength: 0.6,
	bloomRadius: 0.8,
	bloomThreshold: 0.35,

	setup(scene: THREE.Scene, camera: THREE.PerspectiveCamera) {
		scene.background = new THREE.Color(0x000000);
		scene.fog = new THREE.FogExp2(0x000000, 0.0025);

		// Ground
		const gndGeo = new THREE.PlaneGeometry(800, 800, 80, 80);
		const gndPos = gndGeo.attributes.position;
		const gndCol = new Float32Array(gndPos.count * 3);
		for (let i = 0; i < gndPos.count; i++) {
			const x = gndPos.getX(i), z = gndPos.getZ(i);
			const fade = Math.max(0, 1 - Math.sqrt(x*x + z*z) / 400);
			const f = fade * fade;
			gndCol[i*3] = 0.018*f; gndCol[i*3+1] = 0.020*f; gndCol[i*3+2] = 0.025*f;
		}
		gndGeo.setAttribute('color', new THREE.BufferAttribute(gndCol, 3));
		const gnd = new THREE.Mesh(gndGeo, new THREE.MeshBasicMaterial({ vertexColors: true }));
		gnd.rotation.x = -Math.PI / 2;
		gnd.position.y = -0.02;
		scene.add(gnd);

		// Grid
		const gridGroup = new THREE.Group();
		for (let i = -300; i <= 300; i += 10) {
			const d = Math.abs(i);
			const t = 1 - d / 300;
			const op = t * t * 0.08;
			if (op < 0.005) continue;
			const col = new THREE.Color().setHSL(0.55, 0.1, 0.4);
			const m = new THREE.LineBasicMaterial({ color: col, transparent: true, opacity: op });
			gridGroup.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(i,0,-300), new THREE.Vector3(i,0,300)]), m));
			gridGroup.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(-300,0,i), new THREE.Vector3(300,0,i)]), m));
		}
		scene.add(gridGroup);

		// Center line
		scene.add(new THREE.Line(
			new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(0,0.01,0), new THREE.Vector3(0,0.01,350)]),
			new THREE.LineBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.06 })
		));

		// Tee
		const tee = new THREE.Mesh(
			new THREE.CircleGeometry(0.3, 32),
			new THREE.MeshBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.3 })
		);
		tee.rotation.x = -Math.PI / 2;
		tee.position.y = 0.01;
		scene.add(tee);

		// Distance labels
		const yd = 0.9144;
		const distanceLabels: { sprite: THREE.Sprite; yards: number }[] = [];
		[50, 100, 150, 200, 250].forEach(yards => {
			const c = document.createElement('canvas');
			c.width = 128; c.height = 48;
			const ctx = c.getContext('2d')!;
			ctx.fillStyle = 'rgba(255,255,255,0.6)';
			ctx.font = '300 32px -apple-system, Helvetica, sans-serif';
			ctx.textAlign = 'center';
			ctx.fillText(`${yards}`, 64, 34);
			const tex = new THREE.CanvasTexture(c);
			const op = 0.5 - (yards / 250) * 0.2;
			const sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: tex, transparent: true, opacity: op }));
			sp.position.set(12, 0.5, yards * yd);
			sp.scale.set(6, 2.25, 1);
			scene.add(sp);
			distanceLabels.push({ sprite: sp, yards });

			scene.add(new THREE.Line(
				new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(-4,0.01,yards*yd), new THREE.Vector3(4,0.01,yards*yd)]),
				new THREE.LineBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.03 })
			));
		});

		// Stars
		const starCount = 1500;
		const starGeo = new THREE.BufferGeometry();
		const starPos = new Float32Array(starCount * 3);
		for (let i = 0; i < starCount; i++) {
			const th = Math.random() * Math.PI * 2;
			const ph = Math.random() * Math.PI * 0.35;
			const r = 500 + Math.random() * 200;
			starPos[i*3] = r*Math.sin(ph)*Math.cos(th);
			starPos[i*3+1] = r*Math.cos(ph) + 30;
			starPos[i*3+2] = r*Math.sin(ph)*Math.sin(th);
		}
		starGeo.setAttribute('position', new THREE.BufferAttribute(starPos, 3));
		scene.add(new THREE.Points(starGeo, new THREE.PointsMaterial({
			color: 0xffffff, size: 0.4, transparent: true, opacity: 0.4, depthWrite: false
		})));

		// Lighting
		scene.add(new THREE.AmbientLight(0x111115, 2));
		const topLight = new THREE.DirectionalLight(0xffffff, 0.15);
		topLight.position.set(0, 100, 100);
		scene.add(topLight);
		scene.add(new THREE.PointLight(0xffffff, 1.5, 15, 2).translateY(0.5));

		const landingDots = new THREE.Group();
		scene.add(landingDots);

		return { distanceLabels, landingDots };
	}
};
