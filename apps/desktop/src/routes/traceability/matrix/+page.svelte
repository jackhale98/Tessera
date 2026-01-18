<script lang="ts">
	import { onMount } from 'svelte';
	import { Card, CardContent, CardHeader, CardTitle, Button, Label, Select } from '$lib/components/ui';
	import { TraceMatrix } from '$lib/components/traceability';
	import { traceability, type DmmResult } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import { Grid3X3, RefreshCw, Download, Filter, ArrowRight } from 'lucide-svelte';

	// Matrix mode
	type MatrixMode = 'dmm';

	// Frontend-friendly matrix structure for TraceMatrix component
	interface MatrixData {
		entity_ids: string[]; // For backwards compatibility
		row_entity_ids: string[]; // Row entity IDs for rectangular matrix
		col_entity_ids: string[]; // Column entity IDs for rectangular matrix
		entity_titles: Record<string, string>;
		entity_types: Record<string, string>;
		cells: Array<{
			row_id: string;
			col_id: string;
			link_types: string[];
		}>;
	}

	let rowEntityType = $state<string>('REQ');
	let colEntityType = $state<string>('TEST');
	let matrixData = $state<MatrixData | null>(null);
	let dmmResult = $state<DmmResult | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// Entity types for DMM (different entity types comparison)
	const entityTypes = [
		{ value: 'REQ', label: 'Requirements' },
		{ value: 'RISK', label: 'Risks' },
		{ value: 'HAZ', label: 'Hazards' },
		{ value: 'TEST', label: 'Tests' },
		{ value: 'CMP', label: 'Components' },
		{ value: 'ASM', label: 'Assemblies' },
		{ value: 'PROC', label: 'Processes' },
		{ value: 'CTRL', label: 'Controls' }
	];

	// Convert DMM result to frontend matrix format
	function convertDmmToMatrix(dmm: DmmResult): MatrixData {
		const entityTitles: Record<string, string> = {};
		const entityTypesMap: Record<string, string> = {};
		const cells: MatrixData['cells'] = [];

		// Separate row and column entity IDs
		const rowEntityIds = dmm.row_entities.map(e => e.id);
		const colEntityIds = dmm.col_entities.map(e => e.id);

		// All entity IDs for backwards compatibility
		const allIds = [...rowEntityIds, ...colEntityIds];

		// Build maps
		for (const entity of dmm.row_entities) {
			entityTitles[entity.id] = entity.title;
			entityTypesMap[entity.id] = dmm.row_type;
		}
		for (const entity of dmm.col_entities) {
			entityTitles[entity.id] = entity.title;
			entityTypesMap[entity.id] = dmm.col_type;
		}

		// Convert links to cells
		for (const link of dmm.links) {
			cells.push({
				row_id: link.row_id,
				col_id: link.col_id,
				link_types: ['link'] // Generic link type for DMM
			});
		}

		return {
			entity_ids: allIds,
			row_entity_ids: rowEntityIds,
			col_entity_ids: colEntityIds,
			entity_titles: entityTitles,
			entity_types: entityTypesMap,
			cells
		};
	}

	async function loadMatrix() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			// DMM mode - compare two entity types
			if (rowEntityType === colEntityType) {
				error = 'Row and column types must be different for DMM analysis';
				matrixData = null;
				dmmResult = null;
				return;
			}
			const result = await traceability.getDmm(rowEntityType, colEntityType);
			dmmResult = result;
			matrixData = convertDmmToMatrix(result);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function exportMatrix() {
		if (!dmmResult) return;

		// Create CSV content for DMM
		const headers = ['', ...dmmResult.col_entities.map(e => e.id)];
		const rows = dmmResult.row_entities.map((rowEntity) => {
			const row = [rowEntity.id];
			for (const colEntity of dmmResult!.col_entities) {
				const hasLink = dmmResult!.links.some(
					l => l.row_id === rowEntity.id && l.col_id === colEntity.id
				);
				row.push(hasLink ? 'X' : '');
			}
			return row;
		});

		const csv = [headers.join(','), ...rows.map((r) => r.join(','))].join('\n');

		// Download
		const blob = new Blob([csv], { type: 'text/csv' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `dmm-${rowEntityType}-${colEntityType}-${new Date().toISOString().split('T')[0]}.csv`;
		a.click();
		URL.revokeObjectURL(url);
	}

	onMount(() => {
		loadMatrix();
	});

	$effect(() => {
		if ($isProjectOpen && rowEntityType && colEntityType) {
			loadMatrix();
		}
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Domain Mapping Matrix</h1>
			<p class="text-muted-foreground">Visualize relationships between different entity types</p>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" onclick={exportMatrix} disabled={!dmmResult || loading}>
				<Download class="mr-2 h-4 w-4" />
				Export CSV
			</Button>
			<Button variant="outline" onclick={loadMatrix} disabled={loading}>
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
			<div class="flex items-center gap-4 flex-wrap">
				<div class="w-48">
					<Label class="mb-2 block text-sm">Row Entity Type</Label>
					<Select bind:value={rowEntityType}>
						{#each entityTypes as type}
							<option value={type.value}>{type.label}</option>
						{/each}
					</Select>
				</div>
				<div class="flex items-center justify-center pt-6">
					<ArrowRight class="h-5 w-5 text-muted-foreground" />
				</div>
				<div class="w-48">
					<Label class="mb-2 block text-sm">Column Entity Type</Label>
					<Select bind:value={colEntityType}>
						{#each entityTypes as type}
							<option value={type.value}>{type.label}</option>
						{/each}
					</Select>
				</div>
				{#if dmmResult}
					<div class="text-sm text-muted-foreground ml-4">
						Showing {dmmResult.row_entities.length} {entityTypes.find(t => t.value === rowEntityType)?.label ?? rowEntityType}
						&times; {dmmResult.col_entities.length} {entityTypes.find(t => t.value === colEntityType)?.label ?? colEntityType}
						with {dmmResult.coverage.total_links} relationships
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
				Relationship Matrix
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
			{:else if matrixData && dmmResult}
				{#if dmmResult.row_entities.length === 0 || dmmResult.col_entities.length === 0}
					<div class="flex h-64 items-center justify-center text-muted-foreground">
						No entities found for the selected types.
					</div>
				{:else if dmmResult.row_entities.length > 50 || dmmResult.col_entities.length > 50}
					<div class="mb-4 rounded-lg bg-yellow-500/10 p-3 text-sm text-yellow-600 dark:text-yellow-400">
						Large matrix ({dmmResult.row_entities.length}&times;{dmmResult.col_entities.length}). Consider filtering by entity type for better performance.
					</div>
					<TraceMatrix data={matrixData} />
				{:else}
					<TraceMatrix data={matrixData} />
				{/if}
			{:else}
				<div class="flex h-64 items-center justify-center text-muted-foreground">
					No data available. Open a project first.
				</div>
			{/if}
		</CardContent>
	</Card>

	<!-- Coverage Stats -->
	{#if dmmResult}
		<Card>
			<CardHeader>
				<CardTitle>Coverage Statistics</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="grid gap-4 md:grid-cols-2">
					<div class="rounded-lg border p-4">
						<div class="text-sm text-muted-foreground mb-1">
							{entityTypes.find(t => t.value === rowEntityType)?.label ?? rowEntityType} Coverage
						</div>
						<div class="text-2xl font-bold">
							{dmmResult.coverage.row_coverage_pct.toFixed(1)}%
						</div>
						<div class="text-xs text-muted-foreground">
							{dmmResult.coverage.rows_with_links} of {dmmResult.coverage.total_rows} have links
						</div>
					</div>
					<div class="rounded-lg border p-4">
						<div class="text-sm text-muted-foreground mb-1">
							{entityTypes.find(t => t.value === colEntityType)?.label ?? colEntityType} Coverage
						</div>
						<div class="text-2xl font-bold">
							{dmmResult.coverage.col_coverage_pct.toFixed(1)}%
						</div>
						<div class="text-xs text-muted-foreground">
							{dmmResult.coverage.cols_with_links} of {dmmResult.coverage.total_cols} have links
						</div>
					</div>
				</div>
			</CardContent>
		</Card>
	{/if}

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
						<li>Rows represent {entityTypes.find(t => t.value === rowEntityType)?.label ?? 'source'} entities</li>
						<li>Columns represent {entityTypes.find(t => t.value === colEntityType)?.label ?? 'target'} entities</li>
						<li>An 'X' indicates a link exists between the entities</li>
						<li>Click on entity IDs to navigate to their detail page</li>
					</ul>
				</div>
				<div>
					<h4 class="mb-2 font-medium">Common Use Cases</h4>
					<ul class="space-y-1 text-sm text-muted-foreground">
						<li><strong>REQ &times; TEST:</strong> Requirements verification coverage</li>
						<li><strong>REQ &times; CMP:</strong> Requirements allocation to design</li>
						<li><strong>RISK &times; TEST:</strong> Risk verification coverage</li>
						<li><strong>CMP &times; PROC:</strong> Manufacturing dependencies</li>
					</ul>
				</div>
			</div>
		</CardContent>
	</Card>
</div>
