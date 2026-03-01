<script lang="ts">
	import { onMount } from 'svelte';
	import { capas } from '$lib/api/tauri.js';
	import type { CapaSummary } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { EntityCard } from '$lib/components/common/index.js';
	import { Search, ShieldCheck } from 'lucide-svelte';

	let items = $state<CapaSummary[]>([]);
	let loading = $state(true);
	let search = $state('');
	let activeFilter = $state('all');

	const filters = [
		{ id: 'all', label: 'All' },
		{ id: 'open', label: 'Open' },
		{ id: 'overdue', label: 'Overdue' }
	];

	const filteredItems = $derived(
		items.filter(item => {
			if (activeFilter === 'open' && item.capa_status === 'closed') return false;
			if (activeFilter === 'overdue' && item.due_date) {
				const now = new Date();
				const due = new Date(item.due_date);
				if (due >= now || item.capa_status === 'closed') return false;
			} else if (activeFilter === 'overdue' && !item.due_date) {
				return false;
			}
			if (search && !item.title.toLowerCase().includes(search.toLowerCase()) &&
				!(item.capa_number || '').toLowerCase().includes(search.toLowerCase())) return false;
			return true;
		})
	);

	onMount(async () => {
		try {
			const result = await capas.list({ limit: 100 });
			items = result.items;
		} finally {
			loading = false;
		}
	});

	function getCapaMeta(capa: CapaSummary): string | undefined {
		const parts: string[] = [];
		if (capa.capa_type) parts.push(capa.capa_type.charAt(0).toUpperCase() + capa.capa_type.slice(1));
		if (capa.due_date) {
			const due = new Date(capa.due_date);
			const now = new Date();
			if (due < now && capa.capa_status !== 'closed') {
				parts.push('Overdue');
			} else {
				parts.push(`Due: ${due.toLocaleDateString()}`);
			}
		}
		return parts.length > 0 ? parts.join(' | ') : undefined;
	}
</script>

<MobileHeader title="CAPAs" backHref="/quality" />

<div class="page">
	<!-- Search bar -->
	<div class="search-bar">
		<Search size={16} class="search-icon" />
		<input
			type="text"
			placeholder="Search CAPAs..."
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
			<p>Loading CAPAs...</p>
		</div>
	{:else if filteredItems.length === 0}
		<div class="empty-state">
			<ShieldCheck size={40} strokeWidth={1.2} />
			<p>No CAPAs found</p>
		</div>
	{:else}
		<div class="card-list">
			{#each filteredItems as capa}
				<EntityCard
					id={capa.id}
					title={capa.title}
					subtitle={capa.capa_number ? `CAPA #${capa.capa_number}` : capa.capa_type}
					status={capa.capa_status}
					prefix="CAPA"
					href="/quality/capas/{capa.id}"
					meta={getCapaMeta(capa)}
				/>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 12px 16px 32px;
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
</style>
