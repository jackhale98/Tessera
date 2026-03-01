<script lang="ts">
	import { onMount } from 'svelte';
	import { deviations } from '$lib/api/tauri.js';
	import type { DeviationSummary } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { EntityCard } from '$lib/components/common/index.js';
	import { Search, GitBranch, Plus } from 'lucide-svelte';

	let items = $state<DeviationSummary[]>([]);
	let loading = $state(true);
	let search = $state('');
	let activeFilter = $state('all');

	const filters = [
		{ id: 'all', label: 'All' },
		{ id: 'active', label: 'Active' },
		{ id: 'pending', label: 'Pending' }
	];

	const filteredItems = $derived(
		items.filter(item => {
			if (activeFilter === 'active' && item.dev_status !== 'active') return false;
			if (activeFilter === 'pending' && item.dev_status !== 'pending') return false;
			if (search && !item.title.toLowerCase().includes(search.toLowerCase()) &&
				!item.id.toLowerCase().includes(search.toLowerCase())) return false;
			return true;
		})
	);

	onMount(async () => {
		try {
			const result = await deviations.list({ limit: 100 });
			items = result.items;
		} finally {
			loading = false;
		}
	});
</script>

<MobileHeader title="Deviations" backHref="/more" />

<div class="page">
	<!-- Search bar -->
	<div class="search-bar">
		<Search size={16} class="search-icon" />
		<input
			type="text"
			placeholder="Search deviations..."
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
			<p>Loading deviations...</p>
		</div>
	{:else if filteredItems.length === 0}
		<div class="empty-state">
			<GitBranch size={40} strokeWidth={1.2} />
			<p>No deviations found</p>
		</div>
	{:else}
		<div class="card-list">
			{#each filteredItems as dev}
				<EntityCard
					id={dev.id}
					title={dev.title}
					subtitle={dev.deviation_type ? `${dev.deviation_type} - ${dev.category}` : undefined}
					status={dev.dev_status}
					prefix="DEV"
					href="/more/deviations/{dev.id}"
					meta={dev.risk_level ? `Risk: ${dev.risk_level}` : undefined}
				/>
			{/each}
		</div>
	{/if}
</div>

<!-- FAB -->
<a href="/more/deviations/new" class="fab" aria-label="New deviation">
	<Plus size={24} />
</a>

<style>
	.page {
		padding: 12px 16px 96px;
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

	.fab {
		position: fixed;
		bottom: 88px;
		right: 20px;
		width: 56px;
		height: 56px;
		border-radius: 18px;
		background-color: var(--theme-primary);
		color: var(--theme-primary-foreground);
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 4px 16px color-mix(in oklch, var(--theme-primary) 40%, transparent);
		text-decoration: none;
		transition: transform 0.15s ease;
		z-index: 30;
	}

	.fab:active {
		transform: scale(0.92);
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
