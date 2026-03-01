<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { entities } from '$lib/api/tauri.js';
	import type { EntityData } from '$lib/api/types.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { EntityCard } from '$lib/components/common/index.js';
	import { Search, Inbox } from 'lucide-svelte';

	const TYPE_LABELS: Record<string, string> = {
		req: 'Requirements',
		risk: 'Risks',
		test: 'Tests',
		rslt: 'Results',
		cmp: 'Components',
		asm: 'Assemblies',
		proc: 'Processes',
		ctrl: 'Controls',
		work: 'Work Instructions',
		lot: 'Lots',
		dev: 'Deviations',
		ncr: 'NCRs',
		capa: 'CAPAs'
	};

	let items = $state<EntityData[]>([]);
	let loading = $state(true);
	let search = $state('');

	let typeParam = $derived($page.params.type);
	let entityPrefix = $derived(typeParam.toUpperCase());
	let typeLabel = $derived(TYPE_LABELS[typeParam] ?? typeParam.toUpperCase());

	const filteredItems = $derived(
		items.filter(item => {
			if (search && !item.title.toLowerCase().includes(search.toLowerCase()) &&
				!item.id.toLowerCase().includes(search.toLowerCase())) return false;
			return true;
		})
	);

	onMount(async () => {
		try {
			const result = await entities.list(entityPrefix);
			items = result.items;
		} finally {
			loading = false;
		}
	});
</script>

<MobileHeader title={typeLabel} backHref="/browse" />

<div class="page">
	<!-- Search bar -->
	<div class="search-bar">
		<Search size={16} class="search-icon" />
		<input
			type="text"
			placeholder="Search {typeLabel.toLowerCase()}..."
			bind:value={search}
			class="search-input"
		/>
	</div>

	<!-- List -->
	{#if loading}
		<div class="loading-state">
			<div class="loading-spinner"></div>
			<p>Loading {typeLabel.toLowerCase()}...</p>
		</div>
	{:else if filteredItems.length === 0}
		<div class="empty-state">
			<Inbox size={40} strokeWidth={1.2} />
			<p>No {typeLabel.toLowerCase()} found</p>
		</div>
	{:else}
		<div class="card-list">
			{#each filteredItems as entity}
				<EntityCard
					id={entity.id}
					title={entity.title}
					status={entity.status}
					prefix={entityPrefix}
					href="/browse/entity/{entity.id}"
					meta={entity.author ? `By ${entity.author}` : undefined}
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
