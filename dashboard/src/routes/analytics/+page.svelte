<script lang="ts">
	import DispersionPlot from '$lib/components/DispersionPlot.svelte';
	import { shots, sessionStats } from '$lib/stores/shots';

	// Compute distributions
	let speedBuckets = $derived(bucketize($shots.map((s) => s.speed_mph), 5, 80, 200));
	let vlaBuckets = $derived(bucketize($shots.map((s) => s.vla_deg), 2, 0, 40));
	let spinBuckets = $derived(bucketize($shots.map((s) => s.backspin_rpm), 500, 0, 10000));

	function bucketize(values: number[], step: number, min: number, max: number) {
		const buckets: { label: string; count: number; pct: number }[] = [];
		for (let v = min; v < max; v += step) {
			const count = values.filter((x) => x >= v && x < v + step).length;
			buckets.push({
				label: `${v}`,
				count,
				pct: values.length > 0 ? (count / values.length) * 100 : 0
			});
		}
		return buckets;
	}
</script>

<div class="max-w-7xl mx-auto">
	<h1 class="text-2xl font-bold mb-6">Analytics</h1>

	{#if $shots.length === 0}
		<div class="rounded-xl bg-[var(--color-surface)] p-12 border border-[var(--color-border)] text-center">
			<p class="text-[var(--color-text-muted)] text-lg">Hit some shots to see analytics</p>
		</div>
	{:else}
		<!-- Key metrics row -->
		<div class="grid grid-cols-2 md:grid-cols-5 gap-4 mb-6">
			{#each [
				{ label: 'Shots', value: $sessionStats.shot_count.toString(), unit: '' },
				{ label: 'Avg Speed', value: $sessionStats.avg_speed.toFixed(1), unit: 'mph' },
				{ label: 'Avg Launch', value: $sessionStats.avg_vla.toFixed(1), unit: 'deg' },
				{ label: 'Avg Spin', value: $sessionStats.avg_backspin.toLocaleString(), unit: 'rpm' },
				{ label: 'Avg Carry', value: $sessionStats.avg_carry_estimate.toString(), unit: 'yds' }
			] as metric}
				<div class="rounded-xl bg-[var(--color-surface)] p-4 border border-[var(--color-border)]">
					<p class="text-xs text-[var(--color-text-muted)] uppercase tracking-wider">{metric.label}</p>
					<p class="text-2xl font-bold mt-1">{metric.value} <span class="text-sm font-normal text-[var(--color-text-muted)]">{metric.unit}</span></p>
				</div>
			{/each}
		</div>

		<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
			<!-- Dispersion -->
			<div>
				<h2 class="text-lg font-semibold mb-3">Shot Dispersion</h2>
				<DispersionPlot shots={$shots} width={500} height={500} />
			</div>

			<!-- Distributions -->
			<div class="space-y-6">
				<div>
					<h2 class="text-lg font-semibold mb-3">Ball Speed Distribution</h2>
					<div class="rounded-xl bg-[var(--color-surface)] p-4 border border-[var(--color-border)]">
						<div class="flex items-end gap-1 h-32">
							{#each speedBuckets as bucket}
								<div class="flex-1 flex flex-col items-center gap-1">
									<div
										class="w-full bg-[var(--color-primary)] rounded-t transition-all"
										style="height: {bucket.pct * 1.2}%"
									></div>
								</div>
							{/each}
						</div>
						<div class="flex gap-1 mt-1">
							{#each speedBuckets.filter((_, i) => i % 4 === 0) as bucket}
								<span class="flex-[4] text-[9px] text-[var(--color-text-muted)]">{bucket.label}</span>
							{/each}
						</div>
					</div>
				</div>

				<div>
					<h2 class="text-lg font-semibold mb-3">Launch Angle Distribution</h2>
					<div class="rounded-xl bg-[var(--color-surface)] p-4 border border-[var(--color-border)]">
						<div class="flex items-end gap-1 h-32">
							{#each vlaBuckets as bucket}
								<div class="flex-1 flex flex-col items-center gap-1">
									<div
										class="w-full bg-blue-400 rounded-t transition-all"
										style="height: {bucket.pct * 1.2}%"
									></div>
								</div>
							{/each}
						</div>
					</div>
				</div>

				<div>
					<h2 class="text-lg font-semibold mb-3">Backspin Distribution</h2>
					<div class="rounded-xl bg-[var(--color-surface)] p-4 border border-[var(--color-border)]">
						<div class="flex items-end gap-1 h-32">
							{#each spinBuckets as bucket}
								<div class="flex-1 flex flex-col items-center gap-1">
									<div
										class="w-full bg-amber-400 rounded-t transition-all"
										style="height: {bucket.pct * 1.2}%"
									></div>
								</div>
							{/each}
						</div>
					</div>
				</div>
			</div>
		</div>
	{/if}
</div>
