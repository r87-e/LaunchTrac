<script lang="ts">
	import type { ShotData } from '$lib/types';

	let { shots, width = 400, height = 400 }: { shots: ShotData[]; width?: number; height?: number } = $props();

	const cx = width / 2;
	const cy = height * 0.85; // Origin near bottom center
	const scale = 2.0; // Pixels per yard

	function shotToPoint(shot: ShotData) {
		// Estimate carry and lateral displacement
		const carry = shot.speed_mph * 1.5 * Math.sin((2 * shot.vla_deg * Math.PI) / 180) * 0.9;
		const lateral = carry * Math.tan((shot.hla_deg * Math.PI) / 180);

		return {
			x: cx + lateral * scale,
			y: cy - carry * scale,
			carry: Math.round(carry),
			shot
		};
	}

	let points = $derived(shots.map(shotToPoint));
	let avgCarry = $derived(
		points.length > 0 ? Math.round(points.reduce((s, p) => s + p.carry, 0) / points.length) : 0
	);
</script>

<svg {width} {height} class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)]">
	<!-- Grid lines -->
	{#each [50, 100, 150, 200, 250] as dist}
		<line
			x1="0" y1={cy - dist * scale}
			x2={width} y2={cy - dist * scale}
			stroke="var(--color-border)" stroke-width="0.5" stroke-dasharray="4,4"
		/>
		<text x="4" y={cy - dist * scale - 4} fill="var(--color-text-muted)" font-size="10">{dist}y</text>
	{/each}

	<!-- Center line -->
	<line x1={cx} y1="0" x2={cx} y2={height} stroke="var(--color-border)" stroke-width="0.5" />

	<!-- Target -->
	<circle cx={cx} cy={cy - avgCarry * scale} r="20" fill="none" stroke="var(--color-primary)" stroke-width="1" opacity="0.3" />

	<!-- Shot dots -->
	{#each points as point, i}
		<circle
			cx={point.x}
			cy={point.y}
			r="5"
			fill={i === 0 ? 'var(--color-primary)' : 'var(--color-primary)'}
			opacity={i === 0 ? 1 : 0.5}
		/>
	{/each}

	<!-- Origin marker -->
	<rect x={cx - 8} y={cy - 2} width="16" height="4" rx="2" fill="var(--color-text-muted)" />

	<!-- Legend -->
	<text x={width / 2} y={height - 8} text-anchor="middle" fill="var(--color-text-muted)" font-size="11">
		Avg Carry: {avgCarry}y | {points.length} shots
	</text>
</svg>
