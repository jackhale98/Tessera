<script lang="ts">
	import { onMount } from 'svelte';
	import { ncrs } from '$lib/api/tauri.js';
	import type { NcrSummary } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { EntityCard } from '$lib/components/common/index.js';
	import { Search, ShieldAlert, Plus } from 'lucide-svelte';

	let items = $state<NcrSummary[]>([]);
	let loading = $state(true);
	let search = $state('');
	let activeFilter = $state('all');

	const filters = [
		{ id: 'all', label: 'All' },
		{ id: 'open', label: 'Open' },
		{ id: 'critical', label: 'Critical' }
	];

	const filteredItems = $derived(
		items.filter(item => {
			if (activeFilter === 'open' && (item.ncr_status === 'closed')) return false;
			if (activeFilter === 'critical' && item.severity !== 'critical') return false;
			if (search && !item.title.toLowerCase().includes(search.toLowerCase()) &&
				!(item.ncr_number || '').toLowerCase().includes(search.toLowerCase())) return false;
			return true;
		})
	);

	onMount(async () => {
		try {
			const result = await ncrs.list({ limit: 100 });
			items = result.items;
		} finally {
			loading = false;
		}
	});
</script>

<MobileHeader title="NCRs" backHref="/quality" />

<div class="page">
	<!-- Search bar -->
	<div class="search-bar">
		<Search size={16} class="search-icon" />
		<input
			type="text"
			placeholder="Search NCRs..."
			bind:value={search}
			class="search-input"
		/>
	</div>

	<!-- Filter chips -->
	<div class="filter-chips no-scrollbar">
		{#each filters as filter}
			<button
				class="chip"
				class:active={activeFilter === filter.id}
				onclick={() => activeFilter = filter.id}
			>
				{filter.label}
			</button>
		{/each}
	</div>

	<!-- List -->
	{#if loading}
		<div class="loading-state">
			<div class="loading-spinner"></div>
			<p>Loading NCRs...</p>
		</div>
	{:else if filteredItems.length === 0}
		<div class="empty-state">
			<ShieldAlert size={40} strokeWidth={1.2} />
			<p>No NCRs found</p>
		</div>
	{:else}
		<div class="card-list">
			{#each filteredItems as ncr}
				<EntityCard
					id={ncr.id}
					title={ncr.title}
					subtitle={ncr.ncr_number ? `NCR #${ncr.ncr_number}` : ncr.ncr_type}
					status={ncr.ncr_status}
					prefix="NCR"
					href="/quality/ncrs/{ncr.id}"
					meta={ncr.severity ? `Severity: ${ncr.severity}` : undefined}
				/>
			{/each}
		</div>
	{/if}
</div>

<!-- FAB -->
<a href="/quality/ncrs/new" class="fab" aria-label="Create NCR">
	<Plus size={24} />
</a>

<style>
	.page {
		padding: 12px 16px 100px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.search-bar {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 14px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 12px;
	}

	:global(.search-icon) {
		color: var(--theme-muted-foreground);
		flex-shrink: 0;
	}

	.search-input {
		flex: 1;
		background: none;
		border: none;
		outline: none;
		color: var(--theme-foreground);
		font-size: 15px;
	}

	.search-input::placeholder {
		color: var(--theme-muted-foreground);
	}

	.filter-chips {
		display: flex;
		gap: 8px;
		overflow-x: auto;
		padding: 2px 0;
	}

	.chip {
		flex-shrink: 0;
		padding: 8px 16px;
		border-radius: 20px;
		font-size: 13px;
		font-weight: 600;
		border: 1px solid var(--theme-border);
		background-color: var(--theme-card);
		color: var(--theme-muted-foreground);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.chip.active {
		background-color: var(--theme-primary);
		color: var(--theme-primary-foreground);
		border-color: var(--theme-primary);
	}

	.chip:active {
		transform: scale(0.95);
	}

	.card-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.empty-state,
	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 48px 16px;
		color: var(--theme-muted-foreground);
	}

	.loading-spinner {
		width: 32px;
		height: 32px;
		border: 3px solid var(--theme-border);
		border-top-color: var(--theme-primary);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.fab {
		position: fixed;
		bottom: 80px;
		right: 20px;
		width: 56px;
		height: 56px;
		border-radius: 16px;
		background-color: var(--theme-primary);
		color: var(--theme-primary-foreground);
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 4px 16px color-mix(in oklch, var(--theme-primary) 35%, transparent);
		text-decoration: none;
		transition: transform 0.15s ease;
		z-index: 30;
	}

	.fab:active {
		transform: scale(0.9);
	}
</style>
