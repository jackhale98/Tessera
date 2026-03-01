<script lang="ts">
	import { traceability } from '$lib/api/tauri.js';
	import type { TraceResult, TraceNode, TraceEdge } from '$lib/api/types.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { truncateEntityId, getEntityColorSolid } from '$lib/config/entities.js';
	import { Search, Network, ChevronRight } from 'lucide-svelte';

	let searchId = $state('');
	let traceResult = $state<TraceResult | null>(null);
	let loading = $state(false);
	let error = $state('');

	// Build a flat list of nodes with link-type annotations from edges
	let traceList = $derived.by(() => {
		if (!traceResult) return [];

		const edgeMap = new Map<string, string>();
		for (const edge of traceResult.edges) {
			edgeMap.set(edge.to_id, edge.link_type);
		}

		return traceResult.nodes.map(node => ({
			...node,
			linkType: edgeMap.get(node.id) ?? (node.id === traceResult!.root_id ? 'root' : ''),
			prefix: node.entity_type ?? node.id.split('-')[0]
		}));
	});

	async function handleTrace() {
		const id = searchId.trim();
		if (!id) {
			error = 'Please enter an entity ID.';
			return;
		}

		loading = true;
		error = '';
		traceResult = null;

		try {
			traceResult = await traceability.traceFrom({ id });
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}
</script>

<MobileHeader title="Traceability" backHref="/more" />

<div class="trace-page">
	<!-- Search Section -->
	<div class="search-section">
		<div class="search-bar">
			<Search size={16} class="search-icon" />
			<input
				type="text"
				placeholder="Enter entity ID (e.g. REQ-01HQ3K...)"
				bind:value={searchId}
				class="search-input"
				onkeydown={e => { if (e.key === 'Enter') handleTrace(); }}
			/>
		</div>
		<button class="trace-btn" onclick={handleTrace} disabled={loading}>
			{#if loading}
				<div class="btn-spinner"></div>
			{:else}
				<Network size={16} />
			{/if}
			<span>Trace</span>
		</button>
	</div>

	{#if error}
		<div class="error-card">
			<p>{error}</p>
		</div>
	{/if}

	<!-- Results -->
	{#if traceResult}
		<section class="section">
			<h3 class="section-title">
				Trace Results ({traceResult.nodes.length} nodes)
			</h3>
			<div class="trace-list">
				{#each traceList as node}
					<a href="/browse/entity/{node.id}" class="trace-node">
						<div class="node-depth" style="margin-left: {node.depth * 16}px">
							<div class="node-indicator" style="background-color: {getEntityColorSolid(node.prefix)}"></div>
							<div class="node-info">
								<div class="node-top">
									<span class="node-prefix">{node.prefix}</span>
									{#if node.linkType && node.linkType !== 'root'}
										<span class="node-link-type">{node.linkType.replace(/_/g, ' ')}</span>
									{/if}
								</div>
								<span class="node-id">{truncateEntityId(node.id)}</span>
								<span class="node-title">{node.title}</span>
							</div>
						</div>
						<ChevronRight size={16} class="node-chevron" />
					</a>
				{/each}
			</div>
		</section>
	{:else if !loading && !error}
		<div class="empty-state">
			<Network size={48} strokeWidth={1} />
			<p class="empty-title">Trace Entity Links</p>
			<p class="empty-desc">Enter an entity ID above to trace its dependencies and relationships through the project.</p>
		</div>
	{/if}
</div>

<style>
	.trace-page {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding-bottom: 32px;
	}

	.search-section {
		display: flex;
		gap: 10px;
	}

	.search-bar {
		flex: 1;
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
		min-width: 0;
	}

	.search-input::placeholder {
		color: var(--theme-muted-foreground);
	}

	.trace-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 18px;
		border-radius: 12px;
		font-size: 14px;
		font-weight: 600;
		border: none;
		background-color: var(--theme-primary);
		color: var(--theme-primary-foreground);
		cursor: pointer;
		white-space: nowrap;
		transition: transform 0.1s ease;
		-webkit-tap-highlight-color: transparent;
		flex-shrink: 0;
	}

	.trace-btn:active { transform: scale(0.95); }
	.trace-btn:disabled { opacity: 0.5; }

	.btn-spinner {
		width: 16px;
		height: 16px;
		border: 2px solid rgba(255,255,255,0.3);
		border-top-color: white;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	.error-card {
		padding: 12px 16px;
		background-color: color-mix(in oklch, var(--theme-error) 15%, transparent);
		border: 1px solid var(--theme-error);
		border-radius: 12px;
		color: var(--theme-error);
		font-size: 13px;
	}

	.section { display: flex; flex-direction: column; gap: 10px; }
	.section-title { font-size: 15px; font-weight: 700; padding: 0 4px; }

	.trace-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.trace-node {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 14px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 14px;
		text-decoration: none;
		color: inherit;
		transition: all 0.15s ease;
		-webkit-tap-highlight-color: transparent;
	}

	.trace-node:active {
		transform: scale(0.98);
		background-color: var(--theme-accent);
	}

	.node-depth {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
	}

	.node-indicator {
		width: 6px;
		height: 36px;
		border-radius: 3px;
		flex-shrink: 0;
	}

	.node-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.node-top {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.node-prefix {
		font-size: 10px;
		font-weight: 700;
		color: var(--theme-muted-foreground);
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}

	.node-link-type {
		font-size: 10px;
		font-weight: 600;
		color: var(--theme-primary);
		text-transform: capitalize;
		padding: 1px 6px;
		background-color: color-mix(in oklch, var(--theme-primary) 10%, transparent);
		border-radius: 4px;
	}

	.node-id {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--theme-muted-foreground);
		letter-spacing: 0.03em;
	}

	.node-title {
		font-size: 14px;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	:global(.node-chevron) {
		color: var(--theme-muted-foreground);
		flex-shrink: 0;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 48px 24px;
		color: var(--theme-muted-foreground);
		text-align: center;
	}

	.empty-title {
		font-size: 16px;
		font-weight: 700;
		color: var(--theme-foreground);
	}

	.empty-desc {
		font-size: 14px;
		line-height: 1.5;
		max-width: 280px;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
