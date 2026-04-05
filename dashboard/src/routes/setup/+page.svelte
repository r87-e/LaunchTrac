<script lang="ts">
	let step = $state(1);
	let gsproAddress = $state('');
	let e6Address = $state('');
	let calibrating = $state(false);
	let calibrationDone = $state(false);

	async function runCalibration() {
		calibrating = true;
		// Call the Pi's calibration endpoint
		try {
			const resp = await fetch(`http://${window.location.hostname}:8080/api/calibrate`, {
				method: 'POST'
			});
			if (resp.ok) calibrationDone = true;
		} catch {
			// Ignore for now
		}
		calibrating = false;
	}

	async function saveConfig() {
		try {
			await fetch(`http://${window.location.hostname}:8080/api/config`, {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					network: {
						gspro_address: gsproAddress,
						e6_address: e6Address
					}
				})
			});
			step = 4;
		} catch {
			// Ignore for now
		}
	}
</script>

<div class="max-w-2xl mx-auto">
	<h1 class="text-2xl font-bold mb-6">Device Setup</h1>

	<!-- Progress steps -->
	<div class="flex items-center gap-2 mb-8">
		{#each ['Connect', 'Calibrate', 'Simulators', 'Done'] as label, i}
			<div class="flex items-center gap-2">
				<div
					class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium
						{step > i + 1 ? 'bg-[var(--color-primary)] text-white' :
						 step === i + 1 ? 'bg-[var(--color-primary)] text-white' :
						 'bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]'}"
				>
					{step > i + 1 ? '&#10003;' : i + 1}
				</div>
				<span class="text-sm {step === i + 1 ? 'text-[var(--color-text)]' : 'text-[var(--color-text-muted)]'}">{label}</span>
				{#if i < 3}
					<div class="w-8 h-px bg-[var(--color-border)]"></div>
				{/if}
			</div>
		{/each}
	</div>

	<!-- Step content -->
	<div class="rounded-xl bg-[var(--color-surface)] p-8 border border-[var(--color-border)]">
		{#if step === 1}
			<h2 class="text-xl font-semibold mb-4">Connect Your LaunchTrac</h2>
			<p class="text-[var(--color-text-muted)] mb-6">
				Make sure your LaunchTrac is powered on and connected to the same WiFi network as this device.
			</p>
			<div class="space-y-3">
				<div class="flex items-center gap-3 p-4 rounded-lg bg-[var(--color-bg)]">
					<div class="w-3 h-3 rounded-full bg-emerald-400 animate-pulse"></div>
					<span>LaunchTrac detected on network</span>
				</div>
				<div class="flex items-center gap-3 p-4 rounded-lg bg-[var(--color-bg)]">
					<div class="w-3 h-3 rounded-full bg-emerald-400"></div>
					<span>Camera 1 (tee watcher) connected</span>
				</div>
				<div class="flex items-center gap-3 p-4 rounded-lg bg-[var(--color-bg)]">
					<div class="w-3 h-3 rounded-full bg-emerald-400"></div>
					<span>Camera 2 (flight capture) connected</span>
				</div>
			</div>
			<button
				onclick={() => step = 2}
				class="mt-6 px-6 py-2.5 bg-[var(--color-primary)] hover:bg-[var(--color-primary-dark)] text-white rounded-lg font-medium transition-colors"
			>
				Continue
			</button>

		{:else if step === 2}
			<h2 class="text-xl font-semibold mb-4">Calibrate Cameras</h2>
			<p class="text-[var(--color-text-muted)] mb-6">
				Place a golf ball at the marked position on the tee mat, then click calibrate.
				No checkerboard needed -- we use the known ball diameter (42.67mm) for calibration.
			</p>
			{#if !calibrationDone}
				<button
					onclick={runCalibration}
					disabled={calibrating}
					class="px-6 py-2.5 bg-[var(--color-primary)] hover:bg-[var(--color-primary-dark)] text-white rounded-lg font-medium transition-colors disabled:opacity-50"
				>
					{calibrating ? 'Calibrating...' : 'Run Auto-Calibration'}
				</button>
			{:else}
				<div class="p-4 rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 mb-4">
					Calibration complete! Cameras are aligned and ready.
				</div>
				<button
					onclick={() => step = 3}
					class="px-6 py-2.5 bg-[var(--color-primary)] hover:bg-[var(--color-primary-dark)] text-white rounded-lg font-medium transition-colors"
				>
					Continue
				</button>
			{/if}

		{:else if step === 3}
			<h2 class="text-xl font-semibold mb-4">Connect Simulators</h2>
			<p class="text-[var(--color-text-muted)] mb-6">
				Enter the IP address of your simulator PC. Leave empty to skip.
			</p>
			<div class="space-y-4">
				<div>
					<label class="block text-sm font-medium mb-1" for="gspro">GSPro Address</label>
					<input
						id="gspro"
						type="text"
						bind:value={gsproAddress}
						placeholder="192.168.1.100"
						class="w-full px-4 py-2.5 rounded-lg bg-[var(--color-bg)] border border-[var(--color-border)] focus:border-[var(--color-primary)] focus:outline-none"
					/>
					<p class="text-xs text-[var(--color-text-muted)] mt-1">Port 921 (default)</p>
				</div>
				<div>
					<label class="block text-sm font-medium mb-1" for="e6">E6 Connect / TruGolf Address</label>
					<input
						id="e6"
						type="text"
						bind:value={e6Address}
						placeholder="192.168.1.100"
						class="w-full px-4 py-2.5 rounded-lg bg-[var(--color-bg)] border border-[var(--color-border)] focus:border-[var(--color-primary)] focus:outline-none"
					/>
					<p class="text-xs text-[var(--color-text-muted)] mt-1">Port 2483 (default)</p>
				</div>
			</div>
			<button
				onclick={saveConfig}
				class="mt-6 px-6 py-2.5 bg-[var(--color-primary)] hover:bg-[var(--color-primary-dark)] text-white rounded-lg font-medium transition-colors"
			>
				Save & Finish
			</button>

		{:else}
			<div class="text-center py-8">
				<div class="w-16 h-16 mx-auto mb-4 rounded-full bg-emerald-500/20 flex items-center justify-center">
					<svg class="w-8 h-8 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
					</svg>
				</div>
				<h2 class="text-xl font-semibold mb-2">All Set!</h2>
				<p class="text-[var(--color-text-muted)]">Your LaunchTrac is ready. Go hit some balls!</p>
				<a
					href="/"
					class="inline-block mt-6 px-6 py-2.5 bg-[var(--color-primary)] hover:bg-[var(--color-primary-dark)] text-white rounded-lg font-medium transition-colors"
				>
					Go to Live View
				</a>
			</div>
		{/if}
	</div>
</div>
