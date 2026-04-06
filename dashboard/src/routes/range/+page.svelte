<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { lastShot } from '$lib/stores/shots';
	import * as THREE from 'three';
	import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer.js';
	import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass.js';
	import { UnrealBloomPass } from 'three/examples/jsm/postprocessing/UnrealBloomPass.js';
	import { themes, type RangeTheme } from '$lib/range';
	import type { ShotData } from '$lib/types';

	let container: HTMLDivElement;
	let renderer: THREE.WebGLRenderer;
	let composer: EffectComposer;
	let scene: THREE.Scene;
	let camera: THREE.PerspectiveCamera;
	let animationId: number;
	let clock = new THREE.Clock();

	let ball: THREE.Mesh | null = null;
	let trail: THREE.Line | null = null;
	let trailParticles: THREE.Points | null = null;
	let trailPositions: Float32Array;
	let trailIdx = 0;
	const TRAIL_MAX = 150;
	let impactRipple: THREE.Mesh | null = null;
	let landingDots: THREE.Group;
	let distanceLabels: { sprite: THREE.Sprite; yards: number }[] = [];
	let circTex: THREE.Texture;

	let flightPath: THREE.Vector3[] = [];
	let flightIndex = 0;
	let flightActive = false;
	let buildUp = false;
	let buildUpTime = 0;
	let shotStats: ShotData | null = null;
	let showStats = false;

	type CamMode = 'behind' | 'tracking' | 'orbit';
	let cameraMode = $state<CamMode>('behind');
	let camTarget = new THREE.Vector3(0, 3, 40);
	let isFullscreen = $state(false);
	let currentTheme = $state<RangeTheme>(themes[0]);
	let themePickerOpen = $state(false);

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

	let manualSpeed = $state(150);
	let manualVla = $state(12);
	let manualHla = $state(0);
	let manualBackspin = $state(2800);
	let manualSidespin = $state(0);
	let panelOpen = $state(true);
	let panelTab = $state<'presets' | 'custom'>('presets');

	function computeFlight(shot: ShotData): THREE.Vector3[] {
		const pts: THREE.Vector3[] = [];
		const spd = shot.speed_mph * 0.44704;
		const vla = (shot.vla_deg * Math.PI) / 180;
		const hla = (shot.hla_deg * Math.PI) / 180;
		const dt = 0.016;
		let vz = spd * Math.cos(vla) * Math.cos(hla);
		let vy = spd * Math.sin(vla);
		let vx = spd * Math.cos(vla) * Math.sin(hla);
		let x = 0, y = 0.5, z = 0;
		for (let i = 0; i < 600; i++) {
			const v = Math.sqrt(vx*vx + vy*vy + vz*vz);
			const d = 0.22 * v * v * 0.001;
			vz -= (d * vz / v) * dt;
			vy -= 9.81 * dt - 0.00015 * shot.backspin_rpm * dt + (d * vy / v) * dt * 0.3;
			vx += 0.00008 * shot.sidespin_rpm * dt - (d * vx / v) * dt;
			x += vx * dt; y += vy * dt; z += vz * dt;
			pts.push(new THREE.Vector3(x, Math.max(y, 0), z));
			if (y <= 0 && i > 10) break;
		}
		return pts;
	}

	function makeCircleTexture(): THREE.Texture {
		const c = document.createElement('canvas');
		c.width = 64; c.height = 64;
		const ctx = c.getContext('2d')!;
		const g = ctx.createRadialGradient(32,32,0,32,32,32);
		g.addColorStop(0, 'rgba(255,255,255,1)');
		g.addColorStop(0.3, 'rgba(255,255,255,0.5)');
		g.addColorStop(1, 'rgba(255,255,255,0)');
		ctx.fillStyle = g;
		ctx.fillRect(0,0,64,64);
		return new THREE.CanvasTexture(c);
	}

	function initScene(theme: RangeTheme) {
		// Clean up existing scene
		if (renderer && container.contains(renderer.domElement)) {
			container.removeChild(renderer.domElement);
		}
		if (composer) composer.dispose();
		if (renderer) renderer.dispose();

		scene = new THREE.Scene();
		camera = new THREE.PerspectiveCamera(55, container.clientWidth / container.clientHeight, 0.1, 2000);
		camera.position.set(3, 3, -7);

		renderer = new THREE.WebGLRenderer({ antialias: true });
		renderer.setSize(container.clientWidth, container.clientHeight);
		renderer.setPixelRatio(window.devicePixelRatio);
		renderer.toneMapping = THREE.ACESFilmicToneMapping;
		renderer.toneMappingExposure = 1.2;
		container.appendChild(renderer.domElement);

		composer = new EffectComposer(renderer);
		composer.addPass(new RenderPass(scene, camera));
		composer.addPass(new UnrealBloomPass(
			new THREE.Vector2(container.clientWidth, container.clientHeight),
			theme.bloomStrength, theme.bloomRadius, theme.bloomThreshold
		));

		circTex = makeCircleTexture();

		// Let the theme build the scene
		const result = theme.setup(scene, camera);
		distanceLabels = result.distanceLabels;
		landingDots = result.landingDots;

		// Reset flight state
		ball = null; trail = null; trailParticles = null;
		impactRipple = null; flightActive = false; buildUp = false;
		showStats = false; shotStats = null;
	}

	function switchTheme(theme: RangeTheme) {
		currentTheme = theme;
		themePickerOpen = false;
		initScene(theme);
	}

	function launchBall(shot: ShotData) {
		const t = currentTheme;
		shotStats = shot; showStats = false;
		if (ball) scene.remove(ball);
		if (trail) scene.remove(trail);
		if (trailParticles) scene.remove(trailParticles);
		if (impactRipple) { scene.remove(impactRipple); impactRipple = null; }

		buildUp = true; buildUpTime = 0; flightActive = false;
		flightPath = computeFlight(shot); flightIndex = 0;

		// Trail particles
		trailPositions = new Float32Array(TRAIL_MAX * 3);
		trailIdx = 0;
		const tGeo = new THREE.BufferGeometry();
		tGeo.setAttribute('position', new THREE.BufferAttribute(trailPositions, 3));
		trailParticles = new THREE.Points(tGeo, new THREE.PointsMaterial({
			color: t.trailColor, size: 0.25, transparent: true, opacity: 0.3,
			map: circTex, blending: THREE.AdditiveBlending, depthWrite: false
		}));
		scene.add(trailParticles);

		// Ball
		ball = new THREE.Mesh(
			new THREE.SphereGeometry(0.35, 32, 32),
			new THREE.MeshBasicMaterial({ color: t.ballColor })
		);
		ball.position.set(0, 0.5, 0);
		ball.add(new THREE.Mesh(
			new THREE.SphereGeometry(0.5, 16, 16),
			new THREE.MeshBasicMaterial({ color: t.glowColor, transparent: true, opacity: 0.1, blending: THREE.AdditiveBlending, depthWrite: false })
		));
		ball.add(new THREE.PointLight(t.glowColor, 0.8, 10));
		scene.add(ball);
	}

	function startFlight() {
		buildUp = false; flightActive = true;
		trail = new THREE.Line(
			new THREE.BufferGeometry(),
			new THREE.LineBasicMaterial({ color: currentTheme.trailColor, transparent: true, opacity: 0.15 })
		);
		scene.add(trail);
	}

	function createImpact(pos: THREE.Vector3) {
		impactRipple = new THREE.Mesh(
			new THREE.RingGeometry(0.1, 0.3, 48),
			new THREE.MeshBasicMaterial({ color: currentTheme.impactColor, transparent: true, opacity: 0.3, side: THREE.DoubleSide, depthWrite: false })
		);
		impactRipple.rotation.x = -Math.PI / 2;
		impactRipple.position.set(pos.x, 0.05, pos.z);
		scene.add(impactRipple);

		const dot = new THREE.Mesh(
			new THREE.CircleGeometry(0.4, 24),
			new THREE.MeshBasicMaterial({ color: currentTheme.impactColor, transparent: true, opacity: 0.12, side: THREE.DoubleSide, depthWrite: false })
		);
		dot.rotation.x = -Math.PI / 2;
		dot.position.set(pos.x, 0.02, pos.z);
		landingDots.add(dot);
		showStats = true;
	}

	function animate() {
		animationId = requestAnimationFrame(animate);
		const dt = clock.getDelta();

		if (buildUp) {
			buildUpTime += dt;
			if (ball) ball.scale.setScalar(0.9 + Math.sin(buildUpTime * 12) * 0.08);
			if (buildUpTime > 0.25) startFlight();
		}

		if (flightActive && ball && flightPath.length > 0) {
			const spd = Math.max(2, Math.floor(flightPath.length / 140));
			flightIndex = Math.min(flightIndex + spd, flightPath.length - 1);
			const p = flightPath[flightIndex];
			ball.position.copy(p);
			ball.scale.setScalar(1);

			if (trail) {
				const g = new THREE.BufferGeometry().setFromPoints(flightPath.slice(0, flightIndex + 1));
				trail.geometry.dispose(); trail.geometry = g;
			}
			if (trailParticles && trailPositions) {
				const idx = (trailIdx % TRAIL_MAX) * 3;
				trailPositions[idx] = p.x+(Math.random()-0.5)*0.15;
				trailPositions[idx+1] = p.y+(Math.random()-0.5)*0.15;
				trailPositions[idx+2] = p.z+(Math.random()-0.5)*0.15;
				trailIdx++;
				trailParticles.geometry.attributes.position.needsUpdate = true;
			}

			const bd = Math.sqrt(p.x*p.x + p.z*p.z);
			distanceLabels.forEach(d => {
				const diff = Math.abs(bd - d.yards * 0.9144);
				(d.sprite.material as THREE.SpriteMaterial).opacity = diff < 10 ? 0.9 : 0.4;
			});

			if (flightIndex >= flightPath.length - 1) { flightActive = false; createImpact(p); }
		}

		if (impactRipple) {
			const s = impactRipple.scale.x + dt * 8;
			impactRipple.scale.set(s,s,s);
			const m = impactRipple.material as THREE.MeshBasicMaterial;
			m.opacity = Math.max(0, m.opacity - dt * 0.25);
			if (m.opacity <= 0) { scene.remove(impactRipple); impactRipple = null; }
		}

		// Camera
		if (ball && (flightActive || buildUp)) {
			const bp = ball.position;
			if (cameraMode === 'behind') {
				camera.position.lerp(new THREE.Vector3(3,3,-7), 0.04);
				camTarget.lerp(new THREE.Vector3(0, Math.max(bp.y*0.3,1.5), flightActive?Math.max(bp.z,25):40), 0.05);
			} else if (cameraMode === 'tracking') {
				camera.position.lerp(new THREE.Vector3(-20, 5+bp.y*0.15, bp.z*0.4), 0.03);
				camTarget.lerp(bp.clone().add(new THREE.Vector3(0,0,8)), 0.05);
			} else {
				const la = flightIndex+25<flightPath.length ? flightPath[flightIndex+25] : flightPath[flightPath.length-1];
				const dir = new THREE.Vector3().subVectors(la,bp).normalize();
				camera.position.lerp(new THREE.Vector3(bp.x-dir.x*5, bp.y+1.5, bp.z-dir.z*5), 0.05);
				camTarget.lerp(la, 0.05);
			}
		} else {
			const idle = cameraMode==='behind' ? {p:new THREE.Vector3(3,3,-7),t:new THREE.Vector3(0,3,60)} :
				cameraMode==='tracking' ? {p:new THREE.Vector3(-20,5,15),t:new THREE.Vector3(0,2,50)} :
				{p:new THREE.Vector3(4,4,-8),t:new THREE.Vector3(0,2,40)};
			camera.position.lerp(idle.p, 0.015);
			camTarget.lerp(idle.t, 0.015);
		}
		camera.lookAt(camTarget);
		composer.render();
	}

	function buildShot(club:string,speed:number,vla:number,hla:number,back:number,side:number): ShotData {
		return { id:crypto.randomUUID(), shot_number:Math.floor(Math.random()*100), speed_mph:speed, vla_deg:vla, hla_deg:hla, backspin_rpm:back, sidespin_rpm:side, spin_axis_deg:Math.atan2(side,back)*(180/Math.PI), total_spin_rpm:Math.sqrt(back**2+side**2), club, confidence:0.95, processing_time_ms:200, timestamp:new Date().toISOString() };
	}
	function firePreset(p:typeof presets[0]) { launchBall(buildShot(p.club,p.speed+(Math.random()-0.5)*6,p.vla+(Math.random()-0.5)*2,p.hla+(Math.random()-0.5)*1,p.back+(Math.random()-0.5)*400,p.side+(Math.random()-0.5)*200)); }
	function fireCustom() { launchBall(buildShot('Custom',manualSpeed,manualVla,manualHla,manualBackspin,manualSidespin)); }
	function fireRandom() { firePreset(presets[Math.floor(Math.random()*presets.length)]); }

	function handleResize() {
		if (!container||!renderer||!camera) return;
		camera.aspect = container.clientWidth/container.clientHeight;
		camera.updateProjectionMatrix();
		renderer.setSize(container.clientWidth,container.clientHeight);
		composer.setSize(container.clientWidth,container.clientHeight);
	}
	function handleFullscreenChange() { isFullscreen=!!document.fullscreenElement; setTimeout(handleResize,100); }

	let unsubscribe: (()=>void)|undefined;
	onMount(() => {
		initScene(currentTheme); animate();
		window.addEventListener('resize', handleResize);
		document.addEventListener('fullscreenchange', handleFullscreenChange);
		unsubscribe = lastShot.subscribe(shot => { if (shot) launchBall(shot); });
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

	<a href="/" class="back-btn">
		<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg>
		DASHBOARD
	</a>

	<div class="hud-top">
		<span class="hud-title">LAUNCHTRAC</span>
		<div class="hud-controls">
			<button class="cam-btn" class:active={cameraMode==='behind'} on:click={()=>cameraMode='behind'}>DOWN LINE</button>
			<button class="cam-btn" class:active={cameraMode==='tracking'} on:click={()=>cameraMode='tracking'}>BROADCAST</button>
			<button class="cam-btn" class:active={cameraMode==='orbit'} on:click={()=>cameraMode='orbit'}>CHASE</button>
			<button class="cam-btn" on:click={()=>{
				if(document.fullscreenElement){document.exitFullscreen();isFullscreen=false}
				else{container.parentElement?.requestFullscreen();isFullscreen=true}
			}}>{isFullscreen?'EXIT':'FULLSCREEN'}</button>
		</div>
	</div>

	<!-- Theme picker -->
	<div class="theme-picker">
		<button class="theme-toggle" on:click={()=>themePickerOpen=!themePickerOpen}>
			<span class="theme-dot" style="background:{currentTheme.name==='Midnight'?'#fff':'#00ffcc'}"></span>
			{currentTheme.name}
		</button>
		{#if themePickerOpen}
			<div class="theme-list">
				{#each themes as theme}
					<button
						class="theme-option"
						class:active={theme.name === currentTheme.name}
						on:click={()=>switchTheme(theme)}
					>
						<span class="theme-dot" style="background:{theme.name==='Midnight'?'#fff':'#00ffcc'}"></span>
						<div>
							<div class="theme-name">{theme.name}</div>
							<div class="theme-desc">{theme.description}</div>
						</div>
					</button>
				{/each}
			</div>
		{/if}
	</div>

	{#if showStats && shotStats}
		<div class="stats-overlay">
			<div class="stat-big">{shotStats.speed_mph.toFixed(0)}<span class="stat-unit">mph</span></div>
			<div class="stat-row">
				<div class="stat-item"><span class="stat-label">LAUNCH</span><span class="stat-val">{shotStats.vla_deg.toFixed(1)}&deg;</span></div>
				<div class="stat-item"><span class="stat-label">SPIN</span><span class="stat-val">{shotStats.total_spin_rpm.toFixed(0)}</span></div>
				<div class="stat-item"><span class="stat-label">CARRY</span><span class="stat-val carry">{Math.round(shotStats.speed_mph*1.5*Math.sin(2*shotStats.vla_deg*Math.PI/180)*0.9)} yds</span></div>
			</div>
		</div>
	{/if}

	<div class="panel" class:collapsed={!panelOpen}>
		<button class="panel-toggle" on:click={()=>panelOpen=!panelOpen}>{panelOpen?'\u2039':'\u203A'}</button>
		{#if panelOpen}
			<div class="panel-inner">
				<div class="tabs">
					<button class="tab" class:active={panelTab==='presets'} on:click={()=>panelTab='presets'}>PRESETS</button>
					<button class="tab" class:active={panelTab==='custom'} on:click={()=>panelTab='custom'}>CUSTOM</button>
				</div>
				{#if panelTab==='presets'}
					<div class="preset-list">
						{#each presets as p}
							<button class="preset" on:click={()=>firePreset(p)}>
								<span class="preset-name">{p.name}</span>
								<span class="preset-info">{p.speed} mph &middot; {p.vla}&deg;</span>
							</button>
						{/each}
					</div>
				{:else}
					<div class="custom-controls">
						<label><span class="lbl">Speed <span class="val">{manualSpeed}</span></span><input type="range" min="5" max="200" bind:value={manualSpeed}/></label>
						<label><span class="lbl">Launch <span class="val">{manualVla}&deg;</span></span><input type="range" min="-5" max="60" step="0.5" bind:value={manualVla}/></label>
						<label><span class="lbl">Side <span class="val">{manualHla}&deg;</span></span><input type="range" min="-30" max="30" step="0.5" bind:value={manualHla}/></label>
						<label><span class="lbl">Backspin <span class="val">{manualBackspin}</span></span><input type="range" min="0" max="12000" step="100" bind:value={manualBackspin}/></label>
						<label><span class="lbl">Sidespin <span class="val">{manualSidespin}</span></span><input type="range" min="-3000" max="3000" step="50" bind:value={manualSidespin}/></label>
						<button class="action-btn" on:click={fireCustom}>LAUNCH</button>
					</div>
				{/if}
				<button class="action-btn secondary" on:click={fireRandom}>RANDOM</button>
			</div>
		{/if}
	</div>
</div>

<style>
	.range-container { position:fixed; inset:0; background:#000; overflow:hidden; z-index:50; font-family:-apple-system,BlinkMacSystemFont,'Helvetica Neue',sans-serif; }
	.range-canvas { width:100%; height:100%; }

	.back-btn { position:absolute; bottom:20px; right:20px; z-index:25; padding:10px 20px; display:flex; align-items:center; gap:6px; background:rgba(255,255,255,0.03); border:1px solid rgba(255,255,255,0.06); border-radius:8px; color:rgba(255,255,255,0.25); text-decoration:none; font-size:0.7rem; font-weight:500; letter-spacing:0.1em; transition:all 0.2s; }
	.back-btn:hover { color:rgba(255,255,255,0.6); border-color:rgba(255,255,255,0.15); }

	.hud-top { position:absolute; top:20px; right:24px; z-index:10; display:flex; flex-direction:column; align-items:flex-end; gap:10px; }
	.hud-title { font-size:0.7rem; font-weight:300; letter-spacing:0.3em; color:rgba(255,255,255,0.2); }
	.hud-controls { display:flex; gap:3px; }
	.cam-btn { padding:6px 12px; background:transparent; border:1px solid rgba(255,255,255,0.06); border-radius:4px; color:rgba(255,255,255,0.25); font-size:0.55rem; font-weight:500; letter-spacing:0.1em; cursor:pointer; transition:all 0.2s; }
	.cam-btn:hover { color:rgba(255,255,255,0.5); border-color:rgba(255,255,255,0.12); }
	.cam-btn.active { border-color:rgba(255,255,255,0.2); color:rgba(255,255,255,0.7); }

	.theme-picker { position:absolute; top:20px; left:50%; transform:translateX(-50%); z-index:25; }
	.theme-toggle { padding:6px 16px; background:rgba(255,255,255,0.03); border:1px solid rgba(255,255,255,0.06); border-radius:20px; color:rgba(255,255,255,0.3); font-size:0.6rem; font-weight:500; letter-spacing:0.1em; cursor:pointer; transition:all 0.2s; display:flex; align-items:center; gap:8px; }
	.theme-toggle:hover { color:rgba(255,255,255,0.6); border-color:rgba(255,255,255,0.12); }
	.theme-dot { width:8px; height:8px; border-radius:50%; display:inline-block; }
	.theme-list { position:absolute; top:calc(100% + 6px); left:50%; transform:translateX(-50%); background:rgba(0,0,0,0.9); border:1px solid rgba(255,255,255,0.08); border-radius:8px; overflow:hidden; min-width:200px; backdrop-filter:blur(20px); }
	.theme-option { display:flex; align-items:center; gap:10px; padding:12px 16px; background:transparent; border:none; border-bottom:1px solid rgba(255,255,255,0.04); cursor:pointer; width:100%; text-align:left; transition:background 0.15s; }
	.theme-option:last-child { border-bottom:none; }
	.theme-option:hover { background:rgba(255,255,255,0.04); }
	.theme-option.active { background:rgba(255,255,255,0.06); }
	.theme-name { font-size:0.75rem; color:rgba(255,255,255,0.7); font-weight:500; }
	.theme-desc { font-size:0.6rem; color:rgba(255,255,255,0.25); margin-top:2px; }

	.stats-overlay { position:absolute; bottom:80px; right:28px; z-index:10; animation:fadeIn 0.6s ease; text-align:right; }
	.stat-big { font-size:3.5rem; font-weight:200; color:rgba(255,255,255,0.9); line-height:1; font-variant-numeric:tabular-nums; }
	.stat-unit { font-size:1rem; font-weight:300; color:rgba(255,255,255,0.3); margin-left:4px; }
	.stat-row { display:flex; gap:20px; justify-content:flex-end; margin-top:8px; }
	.stat-item { display:flex; flex-direction:column; align-items:flex-end; }
	.stat-label { font-size:0.55rem; font-weight:500; letter-spacing:0.15em; color:rgba(255,255,255,0.2); }
	.stat-val { font-size:1rem; font-weight:300; color:rgba(255,255,255,0.6); font-variant-numeric:tabular-nums; }
	.stat-val.carry { color:rgba(255,255,255,0.9); }

	.panel { position:absolute; top:0; left:0; bottom:0; width:220px; background:rgba(0,0,0,0.7); backdrop-filter:blur(20px); border-right:1px solid rgba(255,255,255,0.04); z-index:20; display:flex; flex-direction:column; transition:width 0.3s; }
	.panel.collapsed { width:0; overflow:hidden; }
	.panel-toggle { position:absolute; top:50%; right:-20px; transform:translateY(-50%); width:20px; height:44px; background:rgba(0,0,0,0.6); border:1px solid rgba(255,255,255,0.06); border-left:none; border-radius:0 4px 4px 0; color:rgba(255,255,255,0.2); font-size:0.8rem; cursor:pointer; display:flex; align-items:center; justify-content:center; z-index:21; }
	.panel-toggle:hover { color:rgba(255,255,255,0.5); }
	.panel-inner { flex:1; display:flex; flex-direction:column; padding:16px; gap:10px; overflow-y:auto; }

	.tabs { display:flex; gap:1px; }
	.tab { flex:1; padding:8px 0; background:transparent; border:none; border-bottom:1px solid transparent; color:rgba(255,255,255,0.2); font-size:0.6rem; font-weight:500; letter-spacing:0.12em; cursor:pointer; transition:all 0.2s; }
	.tab.active { color:rgba(255,255,255,0.6); border-bottom-color:rgba(255,255,255,0.2); }

	.preset-list { display:flex; flex-direction:column; gap:2px; flex:1; overflow-y:auto; }
	.preset { display:flex; justify-content:space-between; align-items:center; padding:10px; background:transparent; border:none; border-bottom:1px solid rgba(255,255,255,0.03); cursor:pointer; transition:all 0.15s; text-align:left; }
	.preset:hover { background:rgba(255,255,255,0.03); }
	.preset:active { background:rgba(255,255,255,0.05); }
	.preset-name { font-size:0.75rem; font-weight:400; color:rgba(255,255,255,0.6); }
	.preset-info { font-size:0.6rem; color:rgba(255,255,255,0.2); }

	.custom-controls { display:flex; flex-direction:column; gap:14px; flex:1; }
	.custom-controls label { display:flex; flex-direction:column; gap:4px; }
	.lbl { font-size:0.65rem; color:rgba(255,255,255,0.25); display:flex; justify-content:space-between; }
	.val { color:rgba(255,255,255,0.5); font-variant-numeric:tabular-nums; }
	.custom-controls input[type="range"] { width:100%; height:2px; -webkit-appearance:none; appearance:none; background:rgba(255,255,255,0.06); border-radius:1px; outline:none; }
	.custom-controls input[type="range"]::-webkit-slider-thumb { -webkit-appearance:none; width:12px; height:12px; border-radius:50%; background:rgba(255,255,255,0.6); cursor:pointer; border:none; }

	.action-btn { padding:10px; background:transparent; border:1px solid rgba(255,255,255,0.08); border-radius:6px; color:rgba(255,255,255,0.4); font-size:0.65rem; font-weight:500; letter-spacing:0.12em; cursor:pointer; transition:all 0.2s; width:100%; }
	.action-btn:hover { border-color:rgba(255,255,255,0.15); color:rgba(255,255,255,0.7); }
	.action-btn:active { background:rgba(255,255,255,0.03); }
	.action-btn.secondary { color:rgba(255,255,255,0.2); border-color:rgba(255,255,255,0.04); }
	.action-btn.secondary:hover { color:rgba(255,255,255,0.4); border-color:rgba(255,255,255,0.08); }

	@keyframes fadeIn { from{opacity:0;transform:translateY(8px)} to{opacity:1;transform:translateY(0)} }
</style>
