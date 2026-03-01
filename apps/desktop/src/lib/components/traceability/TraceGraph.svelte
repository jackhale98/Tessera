<script lang="ts">
	import { goto } from '$app/navigation';
	import { Badge } from '$lib/components/ui';
	import { getEntityColorSolid, getEntityRoute, truncateEntityId } from '$lib/config/entities';
	import type { TraceResult, TraceNode, TraceEdge, EntityPrefix } from '$lib/api/types';
	import { ArrowRight, ExternalLink } from 'lucide-svelte';

	interface Props {
		traceResult: TraceResult;
	}

	let { traceResult }: Props = $props();

	// Group nodes by depth into columns, sorted left (upstream) to right (downstream)
	const columns = $derived.by(() => {
		const groups = new Map<number, TraceNode[]>();
		for (const node of traceResult.nodes) {
			const existing = groups.get(node.depth) ?? [];
			existing.push(node);
			groups.set(node.depth, existing);
		}
		return [...groups.entries()].sort(([a], [b]) => a - b);
	});

	// Build edge lookup: from_id → [{to_id, link_type}]
	const edgesFrom = $derived.by(() => {
		const lookup = new Map<string, { to_id: string; link_type: string }[]>();
		for (const edge of traceResult.edges) {
			const existing = lookup.get(edge.from_id) ?? [];
			existing.push({ to_id: edge.to_id, link_type: edge.link_type });
			lookup.set(edge.from_id, existing);
		}
		return lookup;
	});

	// Build edge lookup: to_id → [{from_id, link_type}]
	const edgesTo = $derived.by(() => {
		const lookup = new Map<string, { from_id: string; link_type: string }[]>();
		for (const edge of traceResult.edges) {
			const existing = lookup.get(edge.to_id) ?? [];
			existing.push({ from_id: edge.from_id, link_type: edge.link_type });
			lookup.set(edge.to_id, existing);
		}
		return lookup;
	});

	function handleNodeClick(node: TraceNode) {
		goto(getEntityRoute(node.entity_type, node.id));
	}

	// Get edge labels for a node (what connects it)
	function getEdgeLabels(nodeId: string): string {
		const from = edgesFrom.get(nodeId) ?? [];
		const to = edgesTo.get(nodeId) ?? [];
		const parts: string[] = [];
		for (const e of to) parts.push(`← ${e.link_type}`);
		for (const e of from) parts.push(`→ ${e.link_type}`);
		return parts.join('\n');
	}

	function getColumnLabel(depth: number): string {
		if (depth === 0) return 'Selected';
		if (depth < 0) return `Upstream`;
		return `Downstream`;
	}

</script>

{#if traceResult.nodes.length === 0}
	<div class="flex h-32 items-center justify-center text-sm text-muted-foreground">
		No trace data available
	</div>
{:else}
	<div class="flex items-start justify-center gap-3 overflow-x-auto py-6 px-2">
		{#each columns as [depth, nodes], colIdx}
			<!-- Arrow between columns -->
			{#if colIdx > 0}
				<div class="flex items-center self-center px-1">
					<ArrowRight class="h-5 w-5 text-muted-foreground/50" />
				</div>
			{/if}

			<!-- Column -->
			<div class="flex flex-col gap-2 min-w-[200px] max-w-[240px]">
				<div class="text-xs text-center font-medium text-muted-foreground mb-1">
					{getColumnLabel(depth)}
				</div>

				{#each nodes as node (node.id)}
					{@const isRoot = node.id === traceResult.root_id}
					<button
						class="flex items-center gap-2 rounded-lg border bg-card p-3 text-left transition-all hover:bg-muted/50 {isRoot
							? 'border-primary border-2 shadow-md ring-1 ring-primary/20'
							: 'hover:border-muted-foreground/30'}"
						onclick={() => handleNodeClick(node)}
						title={getEdgeLabels(node.id)}
					>
						<div
							class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full {getEntityColorSolid(
								node.entity_type
							)}"
						>
							<span class="text-xs font-bold text-white">{node.entity_type}</span>
						</div>
						<div class="min-w-0 flex-1">
							<p class="truncate text-sm font-medium">{node.title}</p>
							<p class="truncate text-xs text-muted-foreground font-mono">
								{truncateEntityId(node.id)}
							</p>
						</div>
						<ExternalLink class="h-3.5 w-3.5 shrink-0 text-muted-foreground/40" />
					</button>
				{/each}
			</div>
		{/each}
	</div>
{/if}
