<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { isProjectOpen, projectName, entityCounts, totalEntities, refreshProject } from '$lib/stores/project.js';
	import { lots, ncrs } from '$lib/api/tauri.js';
	import type { LotSummary, NcrSummary } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { EntityCard } from '$lib/components/common/index.js';
	import { Package, ShieldAlert, AlertTriangle, Activity, ChevronRight, RefreshCw } from 'lucide-svelte';

	let activeLots = $state<LotSummary[]>([]);
	let openNcrs = $state<NcrSummary[]>([]);
	let loading = $state(true);

	const quickStats = $derived([
		{ label: 'Total Entities', value: $totalEntities, icon: Activity, color: 'var(--theme-primary)' },
		{ label: 'Active Lots', value: activeLots.length, icon: Package, color: 'var(--theme-success)' },
		{ label: 'Open NCRs', value: openNcrs.length, icon: ShieldAlert, color: 'var(--theme-error)' },
		{ label: 'Entity Types', value: $entityCounts ? Object.keys($entityCounts).filter(k => ($entityCounts as Record<string, number>)[k] > 0).length : 0, icon: AlertTriangle, color: 'var(--theme-warning)' }
	]);

	onMount(async () => {
		if (!$isProjectOpen) {
			goto('/project');
			return;
		}

		try {
			const [lotsResult, ncrsResult] = await Promise.all([
				lots.list({ lot_status: 'in_progress', limit: 5 }),
				ncrs.list({ open_only: true, limit: 5 })
			]);
			activeLots = lotsResult.items;
			openNcrs = ncrsResult.items;
		} catch {
			// ignore load errors
		} finally {
			loading = false;
		}
	});

	async function handleRefresh() {
		loading = true;
		try {
			await refreshProject();
			const [lotsResult, ncrsResult] = await Promise.all([
				lots.list({ lot_status: 'in_progress', limit: 5 }),
				ncrs.list({ open_only: true, limit: 5 })
			]);
			activeLots = lotsResult.items;
			openNcrs = ncrsResult.items;
		} finally {
			loading = false;
		}
	}
</script>

<MobileHeader title={$projectName || 'Tessera'}>
	<button class="refresh-btn" onclick={handleRefresh} disabled={loading} aria-label="Refresh">
		<RefreshCw size={18} class={loading ? 'spin' : ''} />
	</button>
</MobileHeader>

<div class="dashboard">
	<!-- Stats Grid -->
	<div class="stats-grid">
		{#each quickStats as stat}
			<div class="stat-card">
				<div class="stat-icon" style="background-color: color-mix(in oklch, {stat.color} 15%, transparent); color: {stat.color}">
					<stat.icon size={18} />
				</div>
				<span class="stat-value">{stat.value}</span>
				<span class="stat-label">{stat.label}</span>
			</div>
		{/each}
	</div>

	<!-- Active Lots Section -->
	<section class="section">
		<div class="section-header">
			<h2 class="section-title">Active Lots</h2>
			<a href="/lots" class="section-link">
				See all <ChevronRight size={14} />
			</a>
		</div>
		{#if activeLots.length === 0}
			<div class="empty-state">
				<Package size={32} strokeWidth={1.2} />
				<p>No active lots</p>
			</div>
		{:else}
			<div class="card-list">
				{#each activeLots as lot}
					<EntityCard
						id={lot.id}
						title={lot.title}
						subtitle={lot.lot_number ? `Lot #${lot.lot_number}` : undefined}
						status={lot.lot_status}
						prefix="LOT"
						href="/lots/{lot.id}"
						meta={lot.quantity ? `Qty: ${lot.quantity}` : undefined}
					/>
				{/each}
			</div>
		{/if}
	</section>

	<!-- Open NCRs Section -->
	<section class="section">
		<div class="section-header">
			<h2 class="section-title">Open NCRs</h2>
			<a href="/quality/ncrs" class="section-link">
				See all <ChevronRight size={14} />
			</a>
		</div>
		{#if openNcrs.length === 0}
			<div class="empty-state">
				<ShieldAlert size={32} strokeWidth={1.2} />
				<p>No open NCRs</p>
			</div>
		{:else}
			<div class="card-list">
				{#each openNcrs as ncr}
					<EntityCard
						id={ncr.id}
						title={ncr.title}
						subtitle={ncr.ncr_number || ncr.ncr_type}
						status={ncr.ncr_status}
						prefix="NCR"
						href="/quality/ncrs/{ncr.id}"
						meta={`Severity: ${ncr.severity}`}
					/>
				{/each}
			</div>
		{/if}
	</section>
</div>

<style>
	.dashboard {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 24px;
		padding-bottom: 32px;
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 12px;
	}

	.stat-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 20px 12px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 16px;
	}

	.stat-icon {
		width: 40px;
		height: 40px;
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 2px;
	}

	.stat-value {
		font-size: 24px;
		font-weight: 800;
		letter-spacing: -0.02em;
		line-height: 1;
	}

	.stat-label {
		font-size: 11px;
		font-weight: 500;
		color: var(--theme-muted-foreground);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.section {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 4px;
	}

	.section-title {
		font-size: 18px;
		font-weight: 700;
		letter-spacing: -0.01em;
	}

	.section-link {
		display: flex;
		align-items: center;
		gap: 2px;
		font-size: 13px;
		font-weight: 600;
		color: var(--theme-primary);
		text-decoration: none;
	}

	.card-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 32px 16px;
		color: var(--theme-muted-foreground);
		background-color: var(--theme-card);
		border: 1px dashed var(--theme-border);
		border-radius: 16px;
	}

	.empty-state p {
		font-size: 13px;
	}

	.refresh-btn {
		width: 36px;
		height: 36px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: none;
		border: none;
		color: var(--theme-foreground);
		cursor: pointer;
	}

	.refresh-btn:active {
		background-color: var(--theme-accent);
	}

	:global(.spin) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
