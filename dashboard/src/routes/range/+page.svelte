<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { lastShot } from '$lib/stores/shots';
	import * as THREE from 'three';
	import type { ShotData } from '$lib/types';

	let container: HTMLDivElement;
	let renderer: THREE.WebGLRenderer;
	let scene: THREE.Scene;
	let camera: THREE.PerspectiveCamera;
	let animationId: number;
	let ball: THREE.Mesh | null = null;
	let trail: THREE.Line | null = null;
	let gridFloor: THREE.Group;
	let distanceMarkers: THREE.Group;
	let particles: THREE.Points;
	let impactRipple: THREE.Mesh | null = null;
	let clock = new THREE.Clock();

	// Ball flight state
	let flightPath: THREE.Vector3[] = [];
	let flightIndex = 0;
	let flightActive = false;
	let buildUpPhase = false;
	let buildUpTime = 0;
	let shotStats: ShotData | null = null;
	let showStats = false;

	// Camera modes
	type CameraMode = 'orbit' | 'behind' | 'tracking';
	let cameraMode = $state<CameraMode>('behind');
	let cameraTarget = new THREE.Vector3(0, 5, 0);
	let cameraLerp = 0.03;
	let isFullscreen = $state(false);

	function computeFlightPath(shot: ShotData): THREE.Vector3[] {
		const points: THREE.Vector3[] = [];
		const speed = shot.speed_mph * 0.44704; // mph to m/s
		const vla = (shot.vla_deg * Math.PI) / 180;
		const hla = (shot.hla_deg * Math.PI) / 180;
		const dt = 0.016; // 60fps timestep
		const gravity = 9.81;
		const drag = 0.22;
		const liftCoeff = 0.00015 * shot.backspin_rpm;
		const sideCoeff = 0.00008 * shot.sidespin_rpm;

		// Z = forward (toward skyline), X = left/right, Y = up
		let vz = speed * Math.cos(vla) * Math.cos(hla);
		let vy = speed * Math.sin(vla);
		let vx = speed * Math.cos(vla) * Math.sin(hla);
		let x = 0, y = 0.5, z = 0;

		for (let i = 0; i < 600; i++) {
			const v = Math.sqrt(vx * vx + vy * vy + vz * vz);
			const dragForce = drag * v * v * 0.001;

			vz -= (dragForce * vz / v) * dt;
			vy -= gravity * dt - liftCoeff * dt + (dragForce * vy / v) * dt * 0.3;
			vx += sideCoeff * dt - (dragForce * vx / v) * dt;

			x += vx * dt;
			y += vy * dt;
			z += vz * dt;

			points.push(new THREE.Vector3(x, Math.max(y, 0), z));

			if (y <= 0 && i > 10) break;
		}

		return points;
	}

	function createScene() {
		scene = new THREE.Scene();
		scene.background = new THREE.Color(0x06080f);
		scene.fog = new THREE.FogExp2(0x06080f, 0.0012);

		camera = new THREE.PerspectiveCamera(60, container.clientWidth / container.clientHeight, 0.1, 2000);
		camera.position.set(-8, 6, 12);
		camera.lookAt(0, 2, 0);

		renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
		renderer.setSize(container.clientWidth, container.clientHeight);
		renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
		renderer.toneMapping = THREE.ACESFilmicToneMapping;
		renderer.toneMappingExposure = 1.4;
		container.appendChild(renderer.domElement);

		// === GROUND SURFACE ===
		// Gradient ground plane — dark with green tint, fades to transparent at edges
		const groundGeo = new THREE.PlaneGeometry(600, 600, 60, 60);
		const groundPositions = groundGeo.attributes.position;
		const groundColors = new Float32Array(groundPositions.count * 3);

		for (let i = 0; i < groundPositions.count; i++) {
			const x = groundPositions.getX(i);
			const z = groundPositions.getZ(i);
			const dist = Math.sqrt(x * x + z * z);
			const fade = Math.max(0, 1.0 - dist / 300);
			const fadeSq = fade * fade;
			// Dark green-tinted ground
			groundColors[i * 3] = 0.01 * fadeSq;
			groundColors[i * 3 + 1] = 0.04 * fadeSq;
			groundColors[i * 3 + 2] = 0.02 * fadeSq;
		}
		groundGeo.setAttribute('color', new THREE.BufferAttribute(groundColors, 3));
		const groundMat = new THREE.MeshBasicMaterial({
			vertexColors: true,
			transparent: true,
			opacity: 0.8,
			depthWrite: false
		});
		const ground = new THREE.Mesh(groundGeo, groundMat);
		ground.rotation.x = -Math.PI / 2;
		ground.position.y = -0.05;
		scene.add(ground);

		// === GRID LINES ===
		gridFloor = new THREE.Group();
		const gridSize = 300;
		const gridStep = 10;

		for (let i = -gridSize; i <= gridSize; i += gridStep) {
			const dist = Math.abs(i);
			const opacity = Math.max(0.05, 0.25 * (1 - dist / gridSize));
			const mat = new THREE.LineBasicMaterial({ color: 0x22c55e, transparent: true, opacity });

			const geo1 = new THREE.BufferGeometry().setFromPoints([
				new THREE.Vector3(i, 0, -gridSize),
				new THREE.Vector3(i, 0, gridSize)
			]);
			const geo2 = new THREE.BufferGeometry().setFromPoints([
				new THREE.Vector3(-gridSize, 0, i),
				new THREE.Vector3(gridSize, 0, i)
			]);
			gridFloor.add(new THREE.Line(geo1, mat));
			gridFloor.add(new THREE.Line(geo2, mat));
		}
		scene.add(gridFloor);

		// === TEE BOX ===
		// Glowing launch pad at origin
		const teeGeo = new THREE.CircleGeometry(2, 32);
		const teeMat = new THREE.MeshBasicMaterial({
			color: 0x22c55e,
			transparent: true,
			opacity: 0.15,
			side: THREE.DoubleSide
		});
		const teePad = new THREE.Mesh(teeGeo, teeMat);
		teePad.rotation.x = -Math.PI / 2;
		teePad.position.y = 0.02;
		scene.add(teePad);

		// Tee pad outer ring
		const teeRingGeo = new THREE.RingGeometry(1.8, 2.2, 32);
		const teeRingMat = new THREE.MeshBasicMaterial({
			color: 0x4ade80,
			transparent: true,
			opacity: 0.4,
			side: THREE.DoubleSide,
			blending: THREE.AdditiveBlending
		});
		const teeRing = new THREE.Mesh(teeRingGeo, teeRingMat);
		teeRing.rotation.x = -Math.PI / 2;
		teeRing.position.y = 0.03;
		scene.add(teeRing);

		// === DISTANCE MARKERS ===
		distanceMarkers = new THREE.Group();
		const yardToMeter = 0.9144;
		[50, 100, 150, 200, 250].forEach((yards) => {
			const radius = yards * yardToMeter;

			// Ring
			const ringGeo = new THREE.RingGeometry(radius - 0.4, radius + 0.4, 128);
			const ringMat = new THREE.MeshBasicMaterial({
				color: 0x22c55e,
				transparent: true,
				opacity: 0.08 + 0.04 * (1 - yards / 250),
				side: THREE.DoubleSide,
				blending: THREE.AdditiveBlending,
				depthWrite: false
			});
			const ring = new THREE.Mesh(ringGeo, ringMat);
			ring.rotation.x = -Math.PI / 2;
			ring.position.y = 0.01;
			distanceMarkers.add(ring);

			// Distance label
			const canvas = document.createElement('canvas');
			canvas.width = 256;
			canvas.height = 64;
			const ctx = canvas.getContext('2d')!;
			ctx.fillStyle = '#4ade80';
			ctx.font = 'bold 40px -apple-system, sans-serif';
			ctx.textAlign = 'center';
			ctx.fillText(`${yards}`, 128, 42);
			const texture = new THREE.CanvasTexture(canvas);
			const spriteMat = new THREE.SpriteMaterial({ map: texture, transparent: true, opacity: 0.35 });
			const sprite = new THREE.Sprite(spriteMat);
			sprite.position.set(radius + 5, 4, 0);
			sprite.scale.set(16, 4, 1);
			distanceMarkers.add(sprite);
		});
		scene.add(distanceMarkers);

		// === CELESTIAL OBJECTS ===
		// Moon
		const moonGeo = new THREE.SphereGeometry(15, 32, 32);
		const moonMat = new THREE.MeshBasicMaterial({
			color: 0xddeeff,
			transparent: true,
			opacity: 0.12
		});
		const moon = new THREE.Mesh(moonGeo, moonMat);
		moon.position.set(200, 120, 300);
		scene.add(moon);

		// Moon glow
		const moonGlowGeo = new THREE.SphereGeometry(25, 32, 32);
		const moonGlowMat = new THREE.MeshBasicMaterial({
			color: 0x6688bb,
			transparent: true,
			opacity: 0.04,
			blending: THREE.AdditiveBlending,
			depthWrite: false
		});
		const moonGlow = new THREE.Mesh(moonGlowGeo, moonGlowMat);
		moon.add(moonGlow);

		// Planet (smaller, reddish, different position)
		const planetGeo = new THREE.SphereGeometry(8, 32, 32);
		const planetMat = new THREE.MeshBasicMaterial({
			color: 0xcc8866,
			transparent: true,
			opacity: 0.1
		});
		const planet = new THREE.Mesh(planetGeo, planetMat);
		planet.position.set(-300, 80, 200);
		scene.add(planet);

		// Nebula glow (large, subtle color wash in the sky)
		const nebulaGeo = new THREE.SphereGeometry(60, 16, 16);
		const nebulaMat = new THREE.MeshBasicMaterial({
			color: 0x4422aa,
			transparent: true,
			opacity: 0.025,
			blending: THREE.AdditiveBlending,
			depthWrite: false,
			side: THREE.DoubleSide
		});
		const nebula = new THREE.Mesh(nebulaGeo, nebulaMat);
		nebula.position.set(100, 60, 400);
		scene.add(nebula);

		// Second nebula (warmer tone)
		const nebula2 = new THREE.Mesh(
			new THREE.SphereGeometry(45, 16, 16),
			new THREE.MeshBasicMaterial({
				color: 0x226644,
				transparent: true,
				opacity: 0.02,
				blending: THREE.AdditiveBlending,
				depthWrite: false,
				side: THREE.DoubleSide
			})
		);
		nebula2.position.set(-150, 40, 350);
		scene.add(nebula2);

		// === STARS ===
		const particleCount = 3000;
		const particleGeo = new THREE.BufferGeometry();
		const positions = new Float32Array(particleCount * 3);
		const particleColors = new Float32Array(particleCount * 3);
		const sizes = new Float32Array(particleCount);

		for (let i = 0; i < particleCount; i++) {
			// Stars in a dome above
			const theta = Math.random() * Math.PI * 2;
			const phi = Math.random() * Math.PI * 0.45; // upper hemisphere
			const r = 200 + Math.random() * 400;
			positions[i * 3] = r * Math.sin(phi) * Math.cos(theta);
			positions[i * 3 + 1] = r * Math.cos(phi) + 10;
			positions[i * 3 + 2] = r * Math.sin(phi) * Math.sin(theta);

			// Varied star colors: white, blue-white, warm
			const type = Math.random();
			if (type < 0.6) {
				// White
				const b = 0.4 + Math.random() * 0.6;
				particleColors[i * 3] = b;
				particleColors[i * 3 + 1] = b;
				particleColors[i * 3 + 2] = b;
			} else if (type < 0.85) {
				// Blue-white
				const b = 0.3 + Math.random() * 0.4;
				particleColors[i * 3] = b * 0.7;
				particleColors[i * 3 + 1] = b * 0.85;
				particleColors[i * 3 + 2] = b;
			} else {
				// Warm
				const b = 0.3 + Math.random() * 0.4;
				particleColors[i * 3] = b;
				particleColors[i * 3 + 1] = b * 0.8;
				particleColors[i * 3 + 2] = b * 0.5;
			}
			sizes[i] = 0.3 + Math.random() * 1.2;
		}

		particleGeo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
		particleGeo.setAttribute('color', new THREE.BufferAttribute(particleColors, 3));
		const particleMat = new THREE.PointsMaterial({
			size: 0.8,
			vertexColors: true,
			transparent: true,
			opacity: 0.8,
			blending: THREE.AdditiveBlending,
			depthWrite: false
		});
		particles = new THREE.Points(particleGeo, particleMat);
		scene.add(particles);

		// === HORIZON GLOW ===
		const horizonGeo = new THREE.PlaneGeometry(800, 40);
		const horizonMat = new THREE.MeshBasicMaterial({
			color: 0x0a2a1a,
			transparent: true,
			opacity: 0.3,
			blending: THREE.AdditiveBlending,
			depthWrite: false,
			side: THREE.DoubleSide
		});
		const horizon = new THREE.Mesh(horizonGeo, horizonMat);
		horizon.position.set(0, 5, 350);
		scene.add(horizon);

		// === DISTANT SKYLINE ===
		const skyline = new THREE.Group();
		const buildingMat = new THREE.MeshBasicMaterial({
			color: 0x0a1520,
			transparent: true,
			opacity: 0.7
		});
		const buildingEdgeMat = new THREE.LineBasicMaterial({
			color: 0x1a3a4a,
			transparent: true,
			opacity: 0.3
		});
		const windowMat = new THREE.MeshBasicMaterial({
			color: 0x334455,
			transparent: true,
			opacity: 0.4,
			blending: THREE.AdditiveBlending,
			depthWrite: false
		});

		// Building definitions: [x, width, depth, height]
		const buildings: [number, number, number, number][] = [
			// Center cluster
			[-60, 12, 10, 45],
			[-45, 8, 8, 65],
			[-35, 14, 12, 35],
			[-18, 10, 10, 80],
			[-8, 16, 14, 55],
			[5, 8, 8, 70],
			[15, 12, 10, 40],
			[28, 6, 6, 90],
			[38, 14, 12, 50],
			[50, 10, 8, 60],
			[62, 8, 10, 35],
			// Left spread
			[-90, 10, 8, 25],
			[-78, 6, 6, 40],
			[-105, 12, 10, 20],
			[-120, 8, 8, 30],
			// Right spread
			[80, 12, 10, 30],
			[95, 8, 6, 45],
			[110, 10, 10, 20],
			[125, 6, 8, 35],
			// Far background (taller, more faded)
			[-40, 8, 8, 50],
			[20, 10, 10, 55],
			[-10, 6, 6, 95],
		];

		buildings.forEach(([bx, bw, bd, bh], idx) => {
			const z = 320 + (idx > 18 ? 40 : 0) + Math.random() * 20;
			const geo = new THREE.BoxGeometry(bw, bh, bd);
			const building = new THREE.Mesh(geo, buildingMat.clone());
			building.position.set(bx, bh / 2, z);

			// Slight opacity variation based on distance
			const distFade = 1 - (z - 320) / 80;
			(building.material as THREE.MeshBasicMaterial).opacity = 0.5 * distFade + 0.2;

			skyline.add(building);

			// Edge wireframe for definition
			const edges = new THREE.EdgesGeometry(geo);
			const edgeLine = new THREE.LineSegments(edges, buildingEdgeMat.clone());
			edgeLine.position.copy(building.position);
			(edgeLine.material as THREE.LineBasicMaterial).opacity = 0.15 * distFade;
			skyline.add(edgeLine);

			// Scattered window lights
			const windowCount = Math.floor(bh / 8) * Math.floor(bw / 4);
			for (let w = 0; w < windowCount; w++) {
				if (Math.random() > 0.3) continue; // Only ~30% of windows lit
				const wx = bx + (Math.random() - 0.5) * (bw - 2);
				const wy = 3 + Math.random() * (bh - 6);
				const wz = z - bd / 2 - 0.1;
				const winGeo = new THREE.PlaneGeometry(1.2, 1.5);
				const win = new THREE.Mesh(winGeo, windowMat.clone());
				win.position.set(wx, wy, wz);

				// Vary window color slightly
				const winColor = Math.random() > 0.7
					? new THREE.Color(0x4a6a3a) // green tint
					: Math.random() > 0.5
						? new THREE.Color(0x3a4a6a) // blue tint
						: new THREE.Color(0x5a4a3a); // warm tint
				(win.material as THREE.MeshBasicMaterial).color = winColor;
				(win.material as THREE.MeshBasicMaterial).opacity = 0.15 + Math.random() * 0.3;

				skyline.add(win);
			}
		});

		// Antenna/spire on tallest building
		const antennaGeo = new THREE.CylinderGeometry(0.15, 0.15, 20, 4);
		const antennaMat = new THREE.MeshBasicMaterial({ color: 0x1a2a3a, transparent: true, opacity: 0.5 });
		const antenna = new THREE.Mesh(antennaGeo, antennaMat);
		antenna.position.set(-10, 95 + 10, 340);
		skyline.add(antenna);

		// Blinking red light on antenna
		const redLightGeo = new THREE.SphereGeometry(0.4, 8, 8);
		const redLightMat = new THREE.MeshBasicMaterial({
			color: 0xff2200,
			transparent: true,
			opacity: 0.8,
			blending: THREE.AdditiveBlending
		});
		const redLight = new THREE.Mesh(redLightGeo, redLightMat);
		redLight.position.set(-10, 105, 340);
		redLight.name = 'antennaLight';
		skyline.add(redLight);

		// Second antenna
		const antenna2 = antenna.clone();
		antenna2.position.set(28, 90 + 8, 325);
		antenna2.scale.y = 0.8;
		skyline.add(antenna2);

		scene.add(skyline);

		// === SUN / DAWN GLOW ===
		// Large sun glow behind skyline
		const sunGlowGeo = new THREE.SphereGeometry(50, 32, 32);
		const sunGlowMat = new THREE.MeshBasicMaterial({
			color: 0xff8833,
			transparent: true,
			opacity: 0.12,
			blending: THREE.AdditiveBlending,
			depthWrite: false
		});
		const sunGlow = new THREE.Mesh(sunGlowGeo, sunGlowMat);
		sunGlow.position.set(0, 15, 380);
		scene.add(sunGlow);

		// Inner sun — brighter core
		const sunCoreGeo = new THREE.SphereGeometry(18, 32, 32);
		const sunCoreMat = new THREE.MeshBasicMaterial({
			color: 0xffaa44,
			transparent: true,
			opacity: 0.25,
			blending: THREE.AdditiveBlending,
			depthWrite: false
		});
		const sunCore = new THREE.Mesh(sunCoreGeo, sunCoreMat);
		sunCore.position.set(0, 18, 370);
		scene.add(sunCore);

		// Wide horizon wash — warm band of light at the base
		const horizonWashGeo = new THREE.PlaneGeometry(800, 60);
		const horizonWashMat = new THREE.MeshBasicMaterial({
			color: 0xff6622,
			transparent: true,
			opacity: 0.06,
			blending: THREE.AdditiveBlending,
			depthWrite: false,
			side: THREE.DoubleSide
		});
		const horizonWash = new THREE.Mesh(horizonWashGeo, horizonWashMat);
		horizonWash.position.set(0, 10, 360);
		scene.add(horizonWash);

		// === LIGHTING ===
		const ambient = new THREE.AmbientLight(0x1a1520, 1.2);
		scene.add(ambient);

		// Sunlight from behind skyline — warm directional
		const sunLight = new THREE.DirectionalLight(0xff9944, 0.6);
		sunLight.position.set(0, 30, 380);
		scene.add(sunLight);

		// Soft fill from above
		const fillLight = new THREE.HemisphereLight(0x1a1530, 0x0a1a10, 0.5);
		scene.add(fillLight);

		// Tee pad glow
		const teeLight = new THREE.PointLight(0x4ade80, 3, 25);
		teeLight.position.set(0, 1, 0);
		scene.add(teeLight);

		// Ground visibility
		const groundLight = new THREE.PointLight(0x1a3a2a, 2, 100);
		groundLight.position.set(0, -2, 50);
		scene.add(groundLight);
	}

	function launchBall(shot: ShotData) {
		shotStats = shot;
		showStats = false;

		// Clear previous
		if (ball) scene.remove(ball);
		if (trail) scene.remove(trail);
		if (impactRipple) scene.remove(impactRipple);

		// Build-up phase
		buildUpPhase = true;
		buildUpTime = 0;
		flightActive = false;

		// Compute trajectory
		flightPath = computeFlightPath(shot);
		flightIndex = 0;

		// Create ball
		const ballGeo = new THREE.SphereGeometry(0.4, 32, 32);

		// Ball color based on total spin
		const spinIntensity = Math.min(shot.total_spin_rpm / 6000, 1);
		const backspinColor = new THREE.Color(0x3b82f6); // blue
		const sidespinColor = new THREE.Color(0xef4444); // red
		const ballColor = new THREE.Color(0xffffff);
		if (Math.abs(shot.sidespin_rpm) > Math.abs(shot.backspin_rpm) * 0.5) {
			ballColor.lerp(sidespinColor, spinIntensity * 0.6);
		} else {
			ballColor.lerp(backspinColor, spinIntensity * 0.4);
		}

		const ballMat = new THREE.MeshPhysicalMaterial({
			color: ballColor,
			emissive: ballColor,
			emissiveIntensity: 0.8,
			metalness: 0.1,
			roughness: 0.3
		});
		ball = new THREE.Mesh(ballGeo, ballMat);
		ball.position.set(0, 0.5, 0);
		scene.add(ball);

		// Ball glow
		const glowGeo = new THREE.SphereGeometry(0.8, 16, 16);
		const glowMat = new THREE.MeshBasicMaterial({
			color: ballColor,
			transparent: true,
			opacity: 0.15,
			blending: THREE.AdditiveBlending,
			depthWrite: false
		});
		const glow = new THREE.Mesh(glowGeo, glowMat);
		ball.add(glow);

		// Point light follows ball
		const ballLight = new THREE.PointLight(ballColor.getHex(), 3, 40);
		ballLight.name = 'ballLight';
		ball.add(ballLight);
	}

	function startFlight() {
		buildUpPhase = false;
		flightActive = true;

		// Trail line
		const trailGeo = new THREE.BufferGeometry();
		const trailMat = new THREE.LineBasicMaterial({
			color: 0x4ade80,
			transparent: true,
			opacity: 0.6,
			blending: THREE.AdditiveBlending
		});
		trail = new THREE.Line(trailGeo, trailMat);
		scene.add(trail);
	}

	function createImpact(position: THREE.Vector3) {
		// Ripple ring
		const rippleGeo = new THREE.RingGeometry(0.1, 0.5, 32);
		const rippleMat = new THREE.MeshBasicMaterial({
			color: 0x4ade80,
			transparent: true,
			opacity: 0.8,
			side: THREE.DoubleSide,
			blending: THREE.AdditiveBlending,
			depthWrite: false
		});
		impactRipple = new THREE.Mesh(rippleGeo, rippleMat);
		impactRipple.rotation.x = -Math.PI / 2;
		impactRipple.position.copy(position);
		impactRipple.position.y = 0.1;
		scene.add(impactRipple);

		showStats = true;
	}

	function animate() {
		animationId = requestAnimationFrame(animate);
		const delta = clock.getDelta();
		const time = clock.getElapsedTime();

		// Ambient particle drift
		if (particles) {
			particles.rotation.y += delta * 0.01;
			const pos = particles.geometry.attributes.position.array as Float32Array;
			for (let i = 0; i < pos.length; i += 3) {
				pos[i + 1] += Math.sin(time + pos[i] * 0.01) * 0.005;
			}
			particles.geometry.attributes.position.needsUpdate = true;
		}

		// Blinking antenna light
		const antennaLight = scene.getObjectByName('antennaLight');
		if (antennaLight) {
			const blink = Math.sin(time * 2) > 0.7 ? 1.0 : 0.05;
			(antennaLight as THREE.Mesh).material = new THREE.MeshBasicMaterial({
				color: 0xff2200,
				transparent: true,
				opacity: blink,
				blending: THREE.AdditiveBlending
			});
		}

		// Build-up phase: particles swirl at origin
		if (buildUpPhase) {
			buildUpTime += delta;
			if (ball) {
				ball.scale.setScalar(0.5 + Math.sin(buildUpTime * 8) * 0.2);
				const mat = ball.material as THREE.MeshPhysicalMaterial;
				mat.emissiveIntensity = 0.5 + Math.sin(buildUpTime * 12) * 0.5;
			}
			if (buildUpTime > 0.8) {
				startFlight();
			}
		}

		// Ball flight animation
		if (flightActive && ball && flightPath.length > 0) {
			// Advance 2 points per frame for smooth but visible speed
			const speed = Math.max(2, Math.floor(flightPath.length / 120));
			flightIndex = Math.min(flightIndex + speed, flightPath.length - 1);

			const pos = flightPath[flightIndex];
			ball.position.copy(pos);

			// Ball rotation based on spin
			if (shotStats) {
				ball.rotation.x += (shotStats.backspin_rpm / 60000) * delta * 100;
				ball.rotation.y += (shotStats.sidespin_rpm / 60000) * delta * 100;
			}

			// Update trail
			if (trail) {
				const trailPoints = flightPath.slice(0, flightIndex + 1);
				const trailGeo = new THREE.BufferGeometry().setFromPoints(trailPoints);
				trail.geometry.dispose();
				trail.geometry = trailGeo;
			}

			// Camera follows ball (mode-dependent target set in camera section below)

			// Ball landed
			if (flightIndex >= flightPath.length - 1) {
				flightActive = false;
				createImpact(pos);
			}
		}

		// Expand impact ripple
		if (impactRipple) {
			const scale = impactRipple.scale.x + delta * 15;
			impactRipple.scale.set(scale, scale, scale);
			const mat = impactRipple.material as THREE.MeshBasicMaterial;
			mat.opacity = Math.max(0, mat.opacity - delta * 0.5);
			if (mat.opacity <= 0) {
				scene.remove(impactRipple);
				impactRipple = null;
			}
		}

		// Camera modes — Z is forward (toward skyline), X is left/right
		if (ball && (flightActive || buildUpPhase)) {
			const ballPos = ball.position;
			if (cameraMode === 'behind') {
				// Down-the-line — slightly right, behind tee, looking toward skyline
				camera.position.lerp(new THREE.Vector3(4, 3.5, -8), 0.04);
				const lookZ = flightActive ? Math.max(ballPos.z, 30) : 50;
				cameraTarget.lerp(new THREE.Vector3(0, Math.max(ballPos.y * 0.4, 2), lookZ), 0.05);
			} else if (cameraMode === 'tracking') {
				// Broadcast — side view, pans along with ball
				camera.position.lerp(new THREE.Vector3(-25, 6 + ballPos.y * 0.2, ballPos.z * 0.5), 0.03);
				cameraTarget.lerp(ballPos.clone().add(new THREE.Vector3(0, 0, 10)), 0.05);
			} else {
				// Chase — follows right behind the ball
				const lookAhead = flightIndex + 20 < flightPath.length
					? flightPath[flightIndex + 20]
					: flightPath[flightPath.length - 1];
				const dir = new THREE.Vector3().subVectors(lookAhead, ballPos).normalize();
				const camPos = new THREE.Vector3(
					ballPos.x - dir.x * 6,
					ballPos.y + 2,
					ballPos.z - dir.z * 6
				);
				camera.position.lerp(camPos, 0.05);
				cameraTarget.lerp(lookAhead, 0.05);
			}
		} else {
			// Idle — default view looking down the range toward skyline
			if (cameraMode === 'behind') {
				camera.position.lerp(new THREE.Vector3(4, 3.5, -8), 0.02);
				cameraTarget.lerp(new THREE.Vector3(0, 5, 80), 0.02);
			} else if (cameraMode === 'tracking') {
				camera.position.lerp(new THREE.Vector3(-25, 6, 20), 0.02);
				cameraTarget.lerp(new THREE.Vector3(0, 3, 60), 0.02);
			} else {
				camera.position.lerp(new THREE.Vector3(5, 5, -10), 0.02);
				cameraTarget.lerp(new THREE.Vector3(0, 3, 50), 0.02);
			}
		}
		camera.lookAt(cameraTarget);

		renderer.render(scene, camera);
	}

	function handleResize() {
		if (!container || !renderer || !camera) return;
		camera.aspect = container.clientWidth / container.clientHeight;
		camera.updateProjectionMatrix();
		renderer.setSize(container.clientWidth, container.clientHeight);
	}

	// Shot presets
	const presets = [
		{ name: 'Driver Bomb', club: 'Driver', speed: 165, vla: 11, hla: 0, back: 2500, side: 0 },
		{ name: 'Driver Draw', club: 'Driver', speed: 155, vla: 12, hla: -1, back: 2800, side: -400 },
		{ name: 'Driver Slice', club: 'Driver', speed: 140, vla: 14, hla: 4, back: 3200, side: 800 },
		{ name: 'Pure 7 Iron', club: 'Iron7', speed: 120, vla: 20, hla: 0, back: 6500, side: 0 },
		{ name: '7 Iron Push', club: 'Iron7', speed: 115, vla: 18, hla: 3, back: 5800, side: 300 },
		{ name: 'Wedge Flop', club: 'SandWedge', speed: 65, vla: 38, hla: 0, back: 9500, side: 0 },
		{ name: 'Wedge Spinner', club: 'PitchingWedge', speed: 90, vla: 28, hla: -1, back: 8500, side: -200 },
		{ name: 'Stinger', club: 'Iron3', speed: 135, vla: 6, hla: 0, back: 3500, side: 0 },
		{ name: 'Sky Ball', club: 'Driver', speed: 100, vla: 35, hla: 2, back: 5000, side: 500 },
		{ name: 'Putt', club: 'Putter', speed: 12, vla: 2, hla: 0, back: 300, side: 0 },
		{ name: 'Shank', club: 'Iron7', speed: 80, vla: 8, hla: 25, back: 1500, side: 2000 },
		{ name: 'Topped', club: 'Driver', speed: 90, vla: 1, hla: -2, back: 800, side: -100 },
	];

	// Manual shot parameters
	let manualSpeed = $state(150);
	let manualVla = $state(12);
	let manualHla = $state(0);
	let manualBackspin = $state(2800);
	let manualSidespin = $state(0);
	let panelOpen = $state(true);
	let panelTab = $state<'presets' | 'custom'>('presets');

	function buildShot(club: string, speed: number, vla: number, hla: number, back: number, side: number): ShotData {
		return {
			id: crypto.randomUUID(),
			shot_number: Math.floor(Math.random() * 100),
			speed_mph: speed,
			vla_deg: vla,
			hla_deg: hla,
			backspin_rpm: back,
			sidespin_rpm: side,
			spin_axis_deg: Math.atan2(side, back) * (180 / Math.PI),
			total_spin_rpm: Math.sqrt(back ** 2 + side ** 2),
			club,
			confidence: 0.95,
			processing_time_ms: 200,
			timestamp: new Date().toISOString()
		};
	}

	function firePreset(preset: typeof presets[0]) {
		// Add slight randomness to presets
		const shot = buildShot(
			preset.club,
			preset.speed + (Math.random() - 0.5) * 6,
			preset.vla + (Math.random() - 0.5) * 2,
			preset.hla + (Math.random() - 0.5) * 1,
			preset.back + (Math.random() - 0.5) * 400,
			preset.side + (Math.random() - 0.5) * 200
		);
		launchBall(shot);
	}

	function fireCustom() {
		const shot = buildShot('Custom', manualSpeed, manualVla, manualHla, manualBackspin, manualSidespin);
		launchBall(shot);
	}

	function fireRandom() {
		const preset = presets[Math.floor(Math.random() * presets.length)];
		firePreset(preset);
	}

	// Subscribe to real shot data
	let unsubscribe: (() => void) | undefined;

	function handleFullscreenChange() {
		isFullscreen = !!document.fullscreenElement;
		// Small delay to let the DOM update before resizing
		setTimeout(handleResize, 100);
	}

	onMount(() => {
		createScene();
		animate();
		window.addEventListener('resize', handleResize);
		document.addEventListener('fullscreenchange', handleFullscreenChange);

		unsubscribe = lastShot.subscribe((shot) => {
			if (shot) launchBall(shot);
		});
	});

	onDestroy(() => {
		if (animationId) cancelAnimationFrame(animationId);
		if (renderer) renderer.dispose();
		if (unsubscribe) unsubscribe();
		window.removeEventListener('resize', handleResize);
		document.removeEventListener('fullscreenchange', handleFullscreenChange);
	});
</script>

<div class="range-container">
	<div class="range-canvas" bind:this={container}></div>

	<!-- Back button -->
	<a href="/" class="back-btn" title="Back to dashboard">
		<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
			<path d="M15 18l-6-6 6-6" />
		</svg>
		DASHBOARD
	</a>

	<!-- HUD overlay -->
	<div class="hud-top">
		<span class="hud-title">LAUNCHTRAC RANGE</span>
		<div class="hud-controls">
			<button
				class="cam-btn"
				class:active={cameraMode === 'behind'}
				on:click={() => cameraMode = 'behind'}
				title="Down the line — fixed behind tee"
			>DOWN LINE</button>
			<button
				class="cam-btn"
				class:active={cameraMode === 'tracking'}
				on:click={() => cameraMode = 'tracking'}
				title="TV broadcast angle"
			>BROADCAST</button>
			<button
				class="cam-btn"
				class:active={cameraMode === 'orbit'}
				on:click={() => cameraMode = 'orbit'}
				title="Chase camera — follows behind ball"
			>CHASE</button>
			<button
				class="cam-btn fullscreen-btn"
				on:click={() => {
					if (document.fullscreenElement) {
						document.exitFullscreen();
						isFullscreen = false;
					} else {
						container.parentElement?.requestFullscreen();
						isFullscreen = true;
					}
				}}
				title="Toggle fullscreen"
			>{isFullscreen ? 'EXIT FS' : 'FULLSCREEN'}</button>
		</div>
	</div>

	<!-- Shot stats overlay -->
	{#if showStats && shotStats}
		<div class="stats-overlay">
			<div class="stat-row">
				<span class="stat-label">SPEED</span>
				<span class="stat-value">{shotStats.speed_mph.toFixed(1)}<span class="stat-unit"> mph</span></span>
			</div>
			<div class="stat-row">
				<span class="stat-label">LAUNCH</span>
				<span class="stat-value">{shotStats.vla_deg.toFixed(1)}<span class="stat-unit"> deg</span></span>
			</div>
			<div class="stat-row">
				<span class="stat-label">SPIN</span>
				<span class="stat-value">{shotStats.total_spin_rpm.toFixed(0)}<span class="stat-unit"> rpm</span></span>
			</div>
			<div class="stat-row">
				<span class="stat-label">CARRY</span>
				<span class="stat-value carry">{Math.round(shotStats.speed_mph * 1.5 * Math.sin(2 * shotStats.vla_deg * Math.PI / 180) * 0.9)}<span class="stat-unit"> yds</span></span>
			</div>
		</div>
	{/if}

	<!-- Shot Panel -->
	<div class="panel" class:panel-collapsed={!panelOpen}>
		<button class="panel-toggle" on:click={() => panelOpen = !panelOpen}>
			{panelOpen ? '>' : '<'}
		</button>

		{#if panelOpen}
			<div class="panel-content">
				<!-- Tabs -->
				<div class="panel-tabs">
					<button
						class="panel-tab"
						class:active={panelTab === 'presets'}
						on:click={() => panelTab = 'presets'}
					>PRESETS</button>
					<button
						class="panel-tab"
						class:active={panelTab === 'custom'}
						on:click={() => panelTab = 'custom'}
					>CUSTOM</button>
				</div>

				{#if panelTab === 'presets'}
					<div class="preset-grid">
						{#each presets as preset}
							<button class="preset-btn" on:click={() => firePreset(preset)}>
								<span class="preset-name">{preset.name}</span>
								<span class="preset-detail">{preset.speed} mph / {preset.vla} deg</span>
							</button>
						{/each}
					</div>
				{:else}
					<div class="custom-panel">
						<div class="slider-group">
							<label>
								<span class="slider-label">Speed <span class="slider-val">{manualSpeed} mph</span></span>
								<input type="range" min="5" max="200" step="1" bind:value={manualSpeed} />
							</label>
							<label>
								<span class="slider-label">Launch Angle <span class="slider-val">{manualVla} deg</span></span>
								<input type="range" min="-5" max="60" step="0.5" bind:value={manualVla} />
							</label>
							<label>
								<span class="slider-label">Side Angle <span class="slider-val">{manualHla} deg</span></span>
								<input type="range" min="-30" max="30" step="0.5" bind:value={manualHla} />
							</label>
							<label>
								<span class="slider-label">Backspin <span class="slider-val">{manualBackspin} rpm</span></span>
								<input type="range" min="0" max="12000" step="100" bind:value={manualBackspin} />
							</label>
							<label>
								<span class="slider-label">Sidespin <span class="slider-val">{manualSidespin} rpm</span></span>
								<input type="range" min="-3000" max="3000" step="50" bind:value={manualSidespin} />
							</label>
						</div>
						<button class="fire-btn full" on:click={fireCustom}>LAUNCH</button>
					</div>
				{/if}

				<button class="fire-btn random" on:click={fireRandom}>RANDOM SHOT</button>
			</div>
		{/if}
	</div>
</div>

<style>
	.range-container {
		position: fixed;
		top: 0;
		right: 0;
		bottom: 0;
		left: 0;
		background: #020208;
		overflow: hidden;
		z-index: 50;
	}

	.range-canvas {
		position: absolute;
		top: 0;
		right: 0;
		bottom: 0;
		left: 0;
		width: 100%;
		height: 100%;
	}

	.hud-top {
		position: absolute;
		top: 16px;
		right: 20px;
		z-index: 10;
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 8px;
	}

	.hud-title {
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.15em;
		color: rgba(74, 222, 128, 0.5);
	}

	.hud-controls {
		display: flex;
		gap: 4px;
		margin-top: 10px;
	}

	.cam-btn {
		padding: 5px 10px;
		background: rgba(255, 255, 255, 0.03);
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 4px;
		color: rgba(255, 255, 255, 0.3);
		font-size: 0.6rem;
		font-weight: 600;
		letter-spacing: 0.1em;
		cursor: pointer;
		transition: all 0.2s;
	}

	.cam-btn:hover {
		color: rgba(255, 255, 255, 0.6);
		border-color: rgba(255, 255, 255, 0.15);
	}

	.cam-btn.active {
		background: rgba(74, 222, 128, 0.1);
		border-color: rgba(74, 222, 128, 0.3);
		color: #4ade80;
	}

	.fullscreen-btn {
		margin-left: 8px;
	}

	.back-btn {
		position: absolute;
		bottom: 20px;
		right: 20px;
		z-index: 25;
		padding: 12px 24px;
		display: flex;
		align-items: center;
		gap: 8px;
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 10px;
		color: rgba(255, 255, 255, 0.35);
		text-decoration: none;
		font-size: 0.75rem;
		font-weight: 500;
		letter-spacing: 0.08em;
		transition: all 0.2s;
	}

	.back-btn:hover {
		color: rgba(255, 255, 255, 0.8);
		border-color: rgba(255, 255, 255, 0.25);
		background: rgba(255, 255, 255, 0.08);
	}

	.stats-overlay {
		position: absolute;
		bottom: 100px;
		right: 32px;
		z-index: 10;
		display: flex;
		flex-direction: column;
		gap: 8px;
		animation: fadeIn 0.5s ease;
	}

	.stat-row {
		display: flex;
		align-items: baseline;
		gap: 12px;
		justify-content: flex-end;
	}

	.stat-label {
		font-size: 0.65rem;
		font-weight: 500;
		letter-spacing: 0.15em;
		color: rgba(255, 255, 255, 0.3);
	}

	.stat-value {
		font-size: 1.8rem;
		font-weight: 700;
		color: rgba(255, 255, 255, 0.9);
		font-variant-numeric: tabular-nums;
	}

	.stat-value.carry {
		color: #4ade80;
	}

	.stat-unit {
		font-size: 0.8rem;
		font-weight: 400;
		color: rgba(255, 255, 255, 0.4);
	}

	/* Shot panel */
	.panel {
		position: absolute;
		top: 0;
		left: 0;
		bottom: 0;
		width: 260px;
		background: rgba(2, 2, 8, 0.85);
		backdrop-filter: blur(12px);
		border-right: 1px solid rgba(74, 222, 128, 0.1);
		z-index: 20;
		display: flex;
		flex-direction: column;
		transition: width 0.3s ease;
	}

	.panel-collapsed {
		width: 32px;
	}

	.panel-toggle {
		position: absolute;
		top: 50%;
		right: -24px;
		transform: translateY(-50%);
		width: 24px;
		height: 48px;
		background: rgba(2, 2, 8, 0.9);
		border: 1px solid rgba(74, 222, 128, 0.15);
		border-left: none;
		border-radius: 0 6px 6px 0;
		color: rgba(74, 222, 128, 0.4);
		font-size: 0.7rem;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 21;
	}

	.panel-toggle:hover {
		color: #4ade80;
		background: rgba(74, 222, 128, 0.05);
	}

	.panel-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 16px;
		gap: 12px;
		overflow-y: auto;
	}

	.panel-tabs {
		display: flex;
		gap: 2px;
		background: rgba(255, 255, 255, 0.03);
		border-radius: 6px;
		padding: 2px;
	}

	.panel-tab {
		flex: 1;
		padding: 8px 0;
		background: none;
		border: none;
		color: rgba(255, 255, 255, 0.3);
		font-size: 0.65rem;
		font-weight: 600;
		letter-spacing: 0.12em;
		cursor: pointer;
		border-radius: 5px;
		transition: all 0.2s;
	}

	.panel-tab.active {
		background: rgba(74, 222, 128, 0.1);
		color: #4ade80;
	}

	.preset-grid {
		display: flex;
		flex-direction: column;
		gap: 4px;
		flex: 1;
		overflow-y: auto;
	}

	.preset-btn {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 2px;
		padding: 10px 12px;
		background: rgba(255, 255, 255, 0.02);
		border: 1px solid rgba(255, 255, 255, 0.05);
		border-radius: 6px;
		cursor: pointer;
		transition: all 0.15s;
		text-align: left;
	}

	.preset-btn:hover {
		background: rgba(74, 222, 128, 0.08);
		border-color: rgba(74, 222, 128, 0.2);
	}

	.preset-btn:active {
		transform: scale(0.98);
	}

	.preset-name {
		font-size: 0.8rem;
		font-weight: 600;
		color: rgba(255, 255, 255, 0.85);
	}

	.preset-detail {
		font-size: 0.65rem;
		color: rgba(255, 255, 255, 0.3);
	}

	.custom-panel {
		display: flex;
		flex-direction: column;
		gap: 16px;
		flex: 1;
	}

	.slider-group {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.slider-group label {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.slider-label {
		font-size: 0.7rem;
		color: rgba(255, 255, 255, 0.4);
		display: flex;
		justify-content: space-between;
	}

	.slider-val {
		color: rgba(255, 255, 255, 0.7);
		font-variant-numeric: tabular-nums;
	}

	.slider-group input[type="range"] {
		width: 100%;
		height: 4px;
		-webkit-appearance: none;
		appearance: none;
		background: rgba(255, 255, 255, 0.08);
		border-radius: 2px;
		outline: none;
	}

	.slider-group input[type="range"]::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: #4ade80;
		cursor: pointer;
		border: 2px solid #020208;
	}

	.fire-btn {
		padding: 10px 28px;
		background: rgba(74, 222, 128, 0.1);
		border: 1px solid rgba(74, 222, 128, 0.3);
		border-radius: 8px;
		color: #4ade80;
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.1em;
		cursor: pointer;
		transition: all 0.2s;
	}

	.fire-btn:hover {
		background: rgba(74, 222, 128, 0.2);
		border-color: #4ade80;
	}

	.fire-btn:active {
		transform: scale(0.97);
	}

	.fire-btn.full {
		width: 100%;
	}

	.fire-btn.random {
		width: 100%;
		background: rgba(59, 130, 246, 0.08);
		border-color: rgba(59, 130, 246, 0.25);
		color: #60a5fa;
	}

	.fire-btn.random:hover {
		background: rgba(59, 130, 246, 0.15);
		border-color: #60a5fa;
	}

	@keyframes fadeIn {
		from { opacity: 0; transform: translateY(10px); }
		to { opacity: 1; transform: translateY(0); }
	}
</style>
