import * as THREE from 'three';
import type { RangeTheme } from './types';

export const vapor: RangeTheme = {
	name: 'Vapor',
	description: 'Neon dream. Magenta and cyan on black.',
	ballColor: 0x00ffcc,
	trailColor: 0x00ffaa,
	glowColor: 0x00ffcc,
	impactColor: 0xff00aa,
	fogColor: 0x050008,
	fogDensity: 0.002,
	bloomStrength: 0.8,
	bloomRadius: 0.6,
	bloomThreshold: 0.25,

	setup(scene: THREE.Scene, camera: THREE.PerspectiveCamera) {
		scene.background = new THREE.Color(0x050008);
		scene.fog = new THREE.FogExp2(0x050008, 0.002);

		// Ground — dark purple tint
		const gndGeo = new THREE.PlaneGeometry(800, 800, 80, 80);
		const gndPos = gndGeo.attributes.position;
		const gndCol = new Float32Array(gndPos.count * 3);
		for (let i = 0; i < gndPos.count; i++) {
			const x = gndPos.getX(i), z = gndPos.getZ(i);
			const fade = Math.max(0, 1 - Math.sqrt(x*x + z*z) / 400);
			const f = fade * fade;
			gndCol[i*3] = 0.025*f; gndCol[i*3+1] = 0.008*f; gndCol[i*3+2] = 0.03*f;
		}
		gndGeo.setAttribute('color', new THREE.BufferAttribute(gndCol, 3));
		const gnd = new THREE.Mesh(gndGeo, new THREE.MeshBasicMaterial({ vertexColors: true }));
		gnd.rotation.x = -Math.PI / 2;
		gnd.position.y = -0.02;
		scene.add(gnd);

		// Grid — magenta and cyan alternating
		const gridGroup = new THREE.Group();
		for (let i = -300; i <= 300; i += 10) {
			const d = Math.abs(i);
			const t = 1 - d / 300;
			const op = t * t * 0.1;
			if (op < 0.005) continue;
			// Horizontal lines — magenta
			const mMag = new THREE.LineBasicMaterial({ color: 0xff0088, transparent: true, opacity: op * 0.7 });
			gridGroup.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints([
				new THREE.Vector3(-300,0,i), new THREE.Vector3(300,0,i)
			]), mMag));
			// Vertical lines — cyan
			const mCyan = new THREE.LineBasicMaterial({ color: 0x00ccff, transparent: true, opacity: op * 0.5 });
			gridGroup.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints([
				new THREE.Vector3(i,0,-300), new THREE.Vector3(i,0,300)
			]), mCyan));
		}
		scene.add(gridGroup);

		// Center line — bright cyan
		scene.add(new THREE.Line(
			new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(0,0.01,0), new THREE.Vector3(0,0.01,350)]),
			new THREE.LineBasicMaterial({ color: 0x00ffcc, transparent: true, opacity: 0.1 })
		));

		// Tee — magenta ring
		const tee = new THREE.Mesh(
			new THREE.RingGeometry(0.4, 0.6, 32),
			new THREE.MeshBasicMaterial({ color: 0xff0088, transparent: true, opacity: 0.3, side: THREE.DoubleSide })
		);
		tee.rotation.x = -Math.PI / 2;
		tee.position.y = 0.01;
		scene.add(tee);

		// Tee glow
		scene.add(new THREE.PointLight(0xff0088, 1, 8, 2).translateY(0.3));

		// Distance labels — cyan
		const yd = 0.9144;
		const distanceLabels: { sprite: THREE.Sprite; yards: number }[] = [];
		[50, 100, 150, 200, 250].forEach(yards => {
			const c = document.createElement('canvas');
			c.width = 128; c.height = 48;
			const ctx = c.getContext('2d')!;
			ctx.fillStyle = '#00ffcc';
			ctx.font = '300 32px -apple-system, Helvetica, sans-serif';
			ctx.textAlign = 'center';
			ctx.fillText(`${yards}`, 64, 34);
			const tex = new THREE.CanvasTexture(c);
			const op = 0.4 - (yards / 250) * 0.15;
			const sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: tex, transparent: true, opacity: op }));
			sp.position.set(12, 0.5, yards * yd);
			sp.scale.set(6, 2.25, 1);
			scene.add(sp);
			distanceLabels.push({ sprite: sp, yards });

			// Magenta tick
			scene.add(new THREE.Line(
				new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(-4,0.01,yards*yd), new THREE.Vector3(4,0.01,yards*yd)]),
				new THREE.LineBasicMaterial({ color: 0xff0088, transparent: true, opacity: 0.04 })
			));
		});

		// Stars — warm tint
		const starCount = 1200;
		const starGeo = new THREE.BufferGeometry();
		const starPos = new Float32Array(starCount * 3);
		const starCol = new Float32Array(starCount * 3);
		for (let i = 0; i < starCount; i++) {
			const th = Math.random() * Math.PI * 2;
			const ph = Math.random() * Math.PI * 0.35;
			const r = 400 + Math.random() * 300;
			starPos[i*3] = r*Math.sin(ph)*Math.cos(th);
			starPos[i*3+1] = r*Math.cos(ph) + 30;
			starPos[i*3+2] = r*Math.sin(ph)*Math.sin(th);
			// Mix of warm and cool star colors
			const t = Math.random();
			if (t < 0.4) { starCol[i*3]=0.8; starCol[i*3+1]=0.4; starCol[i*3+2]=0.9; } // purple
			else if (t < 0.7) { starCol[i*3]=0.3; starCol[i*3+1]=0.8; starCol[i*3+2]=0.9; } // cyan
			else { starCol[i*3]=0.9; starCol[i*3+1]=0.9; starCol[i*3+2]=0.9; } // white
		}
		starGeo.setAttribute('position', new THREE.BufferAttribute(starPos, 3));
		starGeo.setAttribute('color', new THREE.BufferAttribute(starCol, 3));
		scene.add(new THREE.Points(starGeo, new THREE.PointsMaterial({
			size: 0.5, vertexColors: true, transparent: true, opacity: 0.5, depthWrite: false
		})));

		// Horizon glow — faint magenta band
		const horizGeo = new THREE.PlaneGeometry(800, 30);
		const horiz = new THREE.Mesh(horizGeo, new THREE.MeshBasicMaterial({
			color: 0xff0066, transparent: true, opacity: 0.03,
			blending: THREE.AdditiveBlending, depthWrite: false, side: THREE.DoubleSide
		}));
		horiz.position.set(0, 5, 350);
		scene.add(horiz);

		// Lighting
		scene.add(new THREE.AmbientLight(0x0a0510, 2));
		const topLight = new THREE.DirectionalLight(0xcc88ff, 0.2);
		topLight.position.set(0, 100, 100);
		scene.add(topLight);

		const landingDots = new THREE.Group();
		scene.add(landingDots);

		return { distanceLabels, landingDots };
	}
};
