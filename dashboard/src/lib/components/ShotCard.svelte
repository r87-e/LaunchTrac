<script lang="ts">
	import type { ShotData } from '$lib/types';

	let { shot }: { shot: ShotData } = $props();

	function formatSpin(rpm: number): string {
		return rpm < 0 ? `${Math.abs(rpm)} L` : `${rpm} R`;
	}
</script>

<div class="rounded-xl bg-[var(--color-surface)] p-6 border border-[var(--color-border)]">
	<div class="flex items-center justify-between mb-4">
		<span class="text-sm text-[var(--color-text-muted)]">Shot #{shot.shot_number}</span>
		<span class="text-xs px-2 py-1 rounded-full {shot.confidence > 0.8 ? 'bg-emerald-500/20 text-emerald-400' : shot.confidence > 0.5 ? 'bg-amber-500/20 text-amber-400' : 'bg-red-500/20 text-red-400'}">
			{(shot.confidence * 100).toFixed(0)}%
		</span>
	</div>

	<div class="grid grid-cols-2 gap-4">
		<div>
			<p class="text-xs uppercase tracking-wider text-[var(--color-text-muted)] mb-1">Ball Speed</p>
			<p class="text-3xl font-bold text-[var(--color-primary)]">{shot.speed_mph.toFixed(1)}</p>
			<p class="text-xs text-[var(--color-text-muted)]">mph</p>
		</div>
		<div>
			<p class="text-xs uppercase tracking-wider text-[var(--color-text-muted)] mb-1">Launch Angle</p>
			<p class="text-3xl font-bold">{shot.vla_deg.toFixed(1)}&deg;</p>
			<p class="text-xs text-[var(--color-text-muted)]">vertical</p>
		</div>
		<div>
			<p class="text-xs uppercase tracking-wider text-[var(--color-text-muted)] mb-1">Backspin</p>
			<p class="text-2xl font-semibold">{shot.backspin_rpm.toLocaleString()}</p>
			<p class="text-xs text-[var(--color-text-muted)]">rpm</p>
		</div>
		<div>
			<p class="text-xs uppercase tracking-wider text-[var(--color-text-muted)] mb-1">Sidespin</p>
			<p class="text-2xl font-semibold">{formatSpin(shot.sidespin_rpm)}</p>
			<p class="text-xs text-[var(--color-text-muted)]">rpm</p>
		</div>
	</div>

	<div class="mt-4 pt-4 border-t border-[var(--color-border)] flex justify-between text-sm text-[var(--color-text-muted)]">
		<span>HLA: {shot.hla_deg.toFixed(1)}&deg;</span>
		<span>Axis: {shot.spin_axis_deg.toFixed(1)}&deg;</span>
		<span>{shot.processing_time_ms}ms</span>
	</div>
</div>
