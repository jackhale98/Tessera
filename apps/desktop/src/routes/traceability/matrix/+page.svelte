<script lang="ts">
	import { onMount } from 'svelte';
	import { Card, CardContent, CardHeader, CardTitle, Button, Label, Select } from '$lib/components/ui';
	import { TraceMatrix } from '$lib/components/traceability';
	import { traceability } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import { Grid3X3, RefreshCw, Download, Filter } from 'lucide-svelte';

	interface DsmData {
		entity_ids: string[];
		entity_titles: Record<string, string>;
		entity_types: Record<string, string>;
		cells: Array<{
			row_id: string;
			col_id: string;
			link_types: string[];
		}>;
	}

	let entityType = $state<string>('all');
	let dsmData = $state<DsmData | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const entityTypes = [
		{ value: 'all', label: 'All Entities' },
		{ value: 'REQ', label: 'Requirements' },
		{ value: 'RISK', label: 'Risks' },
		{ value: 'HAZ', label: 'Hazards' },
		{ value: 'TEST', label: 'Tests' },
		{ value: 'CMP', label: 'Components' },
		{ value: 'ASM', label: 'Assemblies' }
	];

	async function loadDsm() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			const typeParam = entityType === 'all' ? undefined : entityType;
			const result = await traceability.getDsm(typeParam);
			dsmData = result as DsmData;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function exportMatrix() {
		if (!dsmData) return;

		// Create CSV content
		const headers = ['', ...dsmData.entity_ids];
		const rows = dsmData.entity_ids.map((rowId) => {
			const row = [rowId];
			for (const colId of dsmData!.entity_ids) {
				const cell = dsmData!.cells.find((c) => c.row_id === rowId && c.col_id === colId);
				row.push(cell ? cell.link_types.length.toString() : '0');
			}
			return row;
		});

		const csv = [headers.join(','), ...rows.map((r) => r.join(','))].join('\n');

		// Download
		const blob = new Blob([csv], { type: 'text/csv' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `dsm-matrix-${entityType}-${new Date().toISOString().split('T')[0]}.csv`;
		a.click();
		URL.revokeObjectURL(url);
	}

	onMount(() => {
		loadDsm();
	});

	$effect(() => {
		if ($isProjectOpen && entityType) {
			loadDsm();
		}
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Design Structure Matrix</h1>
			<p class="text-muted-foreground">Visualize entity dependencies in matrix form</p>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" onclick={exportMatrix} disabled={!dsmData || loading}>
				<Download class="mr-2 h-4 w-4" />
				Export CSV
			</Button>
			<Button variant="outline" onclick={loadDsm} disabled={loading}>
				<RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
				Refresh
			</Button>
		</div>
	</div>

	<!-- Filters -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<Filter class="h-5 w-5" />
				Filter
			</CardTitle>
		</CardHeader>
		<CardContent>
			<div class="flex items-center gap-4">
				<div class="w-64">
					<Label class="mb-2 block text-sm">Entity Type</Label>
					<Select bind:value={entityType}>
						{#each entityTypes as type}
							<option value={type.value}>{type.label}</option>
						{/each}
					</Select>
				</div>
				{#if dsmData}
					<div class="text-sm text-muted-foreground">
						Showing {dsmData.entity_ids.length} entities with {dsmData.cells.length} relationships
					</div>
				{/if}
			</div>
		</CardContent>
	</Card>

	<!-- Matrix -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<Grid3X3 class="h-5 w-5" />
				Dependency Matrix
			</CardTitle>
		</CardHeader>
		<CardContent>
			{#if loading}
				<div class="flex h-64 items-center justify-center">
					<div class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
				</div>
			{:else if error}
				<div class="flex h-64 items-center justify-center text-destructive">
					{error}
				</div>
			{:else if dsmData}
				{#if dsmData.entity_ids.length > 50}
					<div class="mb-4 rounded-lg bg-yellow-500/10 p-3 text-sm text-yellow-600 dark:text-yellow-400">
						Large matrix ({dsmData.entity_ids.length}x{dsmData.entity_ids.length}). Consider filtering by entity type for better performance.
					</div>
				{/if}
				<TraceMatrix data={dsmData} />
			{:else}
				<div class="flex h-64 items-center justify-center text-muted-foreground">
					No data available. Open a project first.
				</div>
			{/if}
		</CardContent>
	</Card>

	<!-- Legend -->
	<Card>
		<CardHeader>
			<CardTitle>Understanding the Matrix</CardTitle>
		</CardHeader>
		<CardContent>
			<div class="grid gap-4 md:grid-cols-2">
				<div>
					<h4 class="mb-2 font-medium">How to Read</h4>
					<ul class="space-y-1 text-sm text-muted-foreground">
						<li>• Rows represent source entities (FROM)</li>
						<li>• Columns represent target entities (TO)</li>
						<li>• Numbers indicate the count of relationships</li>
						<li>• Click on entity IDs to navigate to their detail page</li>
					</ul>
				</div>
				<div>
					<h4 class="mb-2 font-medium">Matrix Patterns</h4>
					<ul class="space-y-1 text-sm text-muted-foreground">
						<li>• Dense clusters may indicate tightly coupled subsystems</li>
						<li>• Sparse rows/columns may indicate isolated entities</li>
						<li>• Symmetric patterns suggest bidirectional dependencies</li>
						<li>• Lower triangular density suggests good hierarchy</li>
					</ul>
				</div>
			</div>
		</CardContent>
	</Card>
</div>
