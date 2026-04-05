<script lang="ts">
	import '../app.css';
	import StatusBar from '$lib/components/StatusBar.svelte';
	import { connectWebSocket } from '$lib/stores/shots';
	import { onMount } from 'svelte';
	import { page } from '$app/state';

	let { children } = $props();

	onMount(() => {
		const wsUrl = `ws://${window.location.hostname}:8080/ws`;
		connectWebSocket(wsUrl);
	});

	const navItems = [
		{ href: '/', label: 'Live', icon: 'M13 10V3L4 14h7v7l9-11h-7z' },
		{ href: '/history', label: 'History', icon: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z' },
		{ href: '/analytics', label: 'Analytics', icon: 'M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z' },
		{ href: '/setup', label: 'Setup', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z' }
	];
</script>

<div class="min-h-screen flex flex-col">
	<StatusBar />

	<div class="flex flex-1">
		<!-- Sidebar -->
		<nav class="w-16 bg-[var(--color-surface)] border-r border-[var(--color-border)] flex flex-col items-center py-4 gap-2">
			<div class="w-10 h-10 bg-[var(--color-primary)] rounded-lg flex items-center justify-center font-bold text-white mb-4">
				PT
			</div>
			{#each navItems as item}
				<a
					href={item.href}
					class="w-12 h-12 rounded-lg flex flex-col items-center justify-center gap-0.5 transition-colors
						{page.url.pathname === item.href
							? 'bg-[var(--color-primary)]/20 text-[var(--color-primary)]'
							: 'text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]'}"
				>
					<svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d={item.icon} />
					</svg>
					<span class="text-[9px]">{item.label}</span>
				</a>
			{/each}
		</nav>

		<!-- Main content -->
		<main class="flex-1 p-6 overflow-auto">
			{@render children()}
		</main>
	</div>
</div>
