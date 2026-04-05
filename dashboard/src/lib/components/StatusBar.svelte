<script lang="ts">
	import { deviceStatus } from '$lib/stores/shots';
</script>

<div class="flex items-center gap-4 px-6 py-3 bg-[var(--color-surface)] border-b border-[var(--color-border)]">
	<div class="flex items-center gap-2">
		<div class="w-2.5 h-2.5 rounded-full {$deviceStatus.connected ? 'bg-emerald-400 animate-pulse' : 'bg-red-400'}"></div>
		<span class="text-sm font-medium">
			{$deviceStatus.connected ? 'Connected' : 'Disconnected'}
		</span>
	</div>

	<div class="h-4 w-px bg-[var(--color-border)]"></div>

	<span class="text-sm text-[var(--color-text-muted)]">
		LaunchTrac v{$deviceStatus.version}
	</span>

	{#if $deviceStatus.ball_detected}
		<div class="flex items-center gap-1.5 text-sm text-emerald-400">
			<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
				<circle cx="12" cy="12" r="10" />
			</svg>
			Ball Ready
		</div>
	{/if}

	{#if $deviceStatus.simulator_connected}
		<div class="ml-auto text-sm text-[var(--color-text-muted)]">
			Sim: <span class="text-[var(--color-primary)]">{$deviceStatus.simulator_connected}</span>
		</div>
	{/if}
</div>
