<script lang="ts">
	import ShotCard from '$lib/components/ShotCard.svelte';
	import DispersionPlot from '$lib/components/DispersionPlot.svelte';
	import { shots, lastShot, sessionStats } from '$lib/stores/shots';
</script>

<div class="max-w-7xl mx-auto">
	<h1 class="text-2xl font-bold mb-6">Live Session</h1>

	<div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
		<!-- Last Shot -->
		<div class="lg:col-span-2">
			{#if $lastShot}
				<ShotCard shot={$lastShot} />
			{:else}
				<div class="rounded-xl bg-[var(--color-surface)] p-12 border border-[var(--color-border)] text-center">
					<div class="w-16 h-16 mx-auto mb-4 rounded-full bg-[var(--color-surface-hover)] flex items-center justify-center">
						<svg class="w-8 h-8 text-[var(--color-text-muted)]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
							<circle cx="12" cy="12" r="10" />
						</svg>
					</div>
					<p class="text-[var(--color-text-muted)] text-lg">Waiting for first shot...</p>
					<p class="text-[var(--color-text-muted)] text-sm mt-2">Place a ball on the tee and swing</p>
				</div>
			{/if}
		</div>

		<!-- Session Stats -->
		<div class="space-y-4">
			<div class="rounded-xl bg-[var(--color-surface)] p-5 border border-[var(--color-border)]">
				<h2 class="text-sm font-medium text-[var(--color-text-muted)] mb-3">Session Summary</h2>
				<div class="space-y-3">
					{#each [
						{ label: 'Shots', value: $sessionStats.shot_count.toString() },
						{ label: 'Avg Speed', value: `${$sessionStats.avg_speed.toFixed(1)} mph` },
						{ label: 'Avg Launch', value: `${$sessionStats.avg_vla.toFixed(1)} deg` },
						{ label: 'Avg Backspin', value: `${$sessionStats.avg_backspin.toLocaleString()} rpm` },
						{ label: 'Avg Carry', value: `${$sessionStats.avg_carry_estimate} yds`, highlight: true }
					] as stat}
						<div class="flex justify-between">
							<span class="text-[var(--color-text-muted)]">{stat.label}</span>
							<span class="font-semibold {stat.highlight ? 'text-[var(--color-primary)]' : ''}">{stat.value}</span>
						</div>
					{/each}
				</div>
			</div>
		</div>
	</div>

	<!-- Dispersion + Recent -->
	<div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-6">
		<div>
			<h2 class="text-lg font-semibold mb-3">Dispersion</h2>
			<DispersionPlot shots={$shots} width={500} height={500} />
		</div>

		<div>
			<h2 class="text-lg font-semibold mb-3">Recent Shots</h2>
			<div class="space-y-2 max-h-[500px] overflow-y-auto">
				{#each $shots.slice(0, 20) as shot (shot.id)}
					<div class="rounded-lg bg-[var(--color-surface)] px-4 py-3 border border-[var(--color-border)] flex items-center justify-between text-sm">
						<span class="text-[var(--color-text-muted)]">#{shot.shot_number}</span>
						<span class="font-semibold text-[var(--color-primary)]">{shot.speed_mph.toFixed(1)} mph</span>
						<span>{shot.vla_deg.toFixed(1)} deg</span>
						<span>{shot.backspin_rpm.toLocaleString()} rpm</span>
						<span class="text-[var(--color-text-muted)]">{shot.hla_deg.toFixed(1)} deg</span>
					</div>
				{:else}
					<p class="text-[var(--color-text-muted)] text-center py-8">No shots yet</p>
				{/each}
			</div>
		</div>
	</div>
</div>
