<script lang="ts">
	import { shots } from '$lib/stores/shots';

	let clubFilter = $state('all');
	let sortBy = $state<'timestamp' | 'speed_mph' | 'vla_deg' | 'backspin_rpm'>('timestamp');
	let sortDir = $state<'asc' | 'desc'>('desc');

	let filteredShots = $derived(
		$shots
			.filter((s) => clubFilter === 'all' || s.club === clubFilter)
			.sort((a, b) => {
				const va = a[sortBy] as number;
				const vb = b[sortBy] as number;
				return sortDir === 'asc' ? va - vb : vb - va;
			})
	);

	function toggleSort(col: typeof sortBy) {
		if (sortBy === col) {
			sortDir = sortDir === 'asc' ? 'desc' : 'asc';
		} else {
			sortBy = col;
			sortDir = 'desc';
		}
	}
</script>

<div class="max-w-7xl mx-auto">
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-2xl font-bold">Shot History</h1>

		<select
			bind:value={clubFilter}
			class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm"
		>
			<option value="all">All Clubs</option>
			<option value="Driver">Driver</option>
			<option value="Iron7">7 Iron</option>
			<option value="PitchingWedge">PW</option>
			<option value="Putter">Putter</option>
		</select>
	</div>

	<div class="rounded-xl bg-[var(--color-surface)] border border-[var(--color-border)] overflow-hidden">
		<table class="w-full text-sm">
			<thead>
				<tr class="border-b border-[var(--color-border)]">
					<th class="px-4 py-3 text-left text-[var(--color-text-muted)] font-medium">#</th>
					<th
						class="px-4 py-3 text-left text-[var(--color-text-muted)] font-medium cursor-pointer hover:text-[var(--color-text)]"
						onclick={() => toggleSort('speed_mph')}
					>
						Speed {sortBy === 'speed_mph' ? (sortDir === 'asc' ? '&#9650;' : '&#9660;') : ''}
					</th>
					<th
						class="px-4 py-3 text-left text-[var(--color-text-muted)] font-medium cursor-pointer hover:text-[var(--color-text)]"
						onclick={() => toggleSort('vla_deg')}
					>
						VLA {sortBy === 'vla_deg' ? (sortDir === 'asc' ? '&#9650;' : '&#9660;') : ''}
					</th>
					<th class="px-4 py-3 text-left text-[var(--color-text-muted)] font-medium">HLA</th>
					<th
						class="px-4 py-3 text-left text-[var(--color-text-muted)] font-medium cursor-pointer hover:text-[var(--color-text)]"
						onclick={() => toggleSort('backspin_rpm')}
					>
						Backspin {sortBy === 'backspin_rpm' ? (sortDir === 'asc' ? '&#9650;' : '&#9660;') : ''}
					</th>
					<th class="px-4 py-3 text-left text-[var(--color-text-muted)] font-medium">Sidespin</th>
					<th class="px-4 py-3 text-left text-[var(--color-text-muted)] font-medium">Club</th>
					<th class="px-4 py-3 text-left text-[var(--color-text-muted)] font-medium">Conf</th>
				</tr>
			</thead>
			<tbody>
				{#each filteredShots as shot (shot.id)}
					<tr class="border-b border-[var(--color-border)] hover:bg-[var(--color-surface-hover)] transition-colors">
						<td class="px-4 py-3 text-[var(--color-text-muted)]">{shot.shot_number}</td>
						<td class="px-4 py-3 font-semibold text-[var(--color-primary)]">{shot.speed_mph.toFixed(1)} mph</td>
						<td class="px-4 py-3">{shot.vla_deg.toFixed(1)}&deg;</td>
						<td class="px-4 py-3">{shot.hla_deg.toFixed(1)}&deg;</td>
						<td class="px-4 py-3">{shot.backspin_rpm.toLocaleString()}</td>
						<td class="px-4 py-3">{shot.sidespin_rpm}</td>
						<td class="px-4 py-3">{shot.club}</td>
						<td class="px-4 py-3">
							<span class="text-xs px-2 py-0.5 rounded-full {shot.confidence > 0.8 ? 'bg-emerald-500/20 text-emerald-400' : 'bg-amber-500/20 text-amber-400'}">
								{(shot.confidence * 100).toFixed(0)}%
							</span>
						</td>
					</tr>
				{:else}
					<tr>
						<td colspan="8" class="px-4 py-12 text-center text-[var(--color-text-muted)]">
							No shots recorded yet
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>
