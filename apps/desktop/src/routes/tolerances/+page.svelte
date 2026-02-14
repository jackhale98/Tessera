<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { EntityTable } from '$lib/components/entities';
	import { Card, CardContent, CardHeader, CardTitle, Button } from '$lib/components/ui';
	import { entities } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import type { EntityData } from '$lib/api/types';

	let entitiesData = $state<EntityData[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// Helper types for analysis results
	interface AnalysisResults {
		worst_case?: { result?: string };
		rss?: { cpk?: number; yield_percent?: number };
	}

	// Helper to format WC result with warning indicator
	function formatWcResult(value: unknown, entity: EntityData): string {
		const results = entity.data?.analysis_results as AnalysisResults | undefined;
		const result = results?.worst_case?.result;
		if (!result) return '-';
		const formatted = result.charAt(0).toUpperCase() + result.slice(1);
		// Add warning for fail/marginal
		if (result.toLowerCase() === 'fail') {
			return `⚠ ${formatted}`;
		}
		return formatted;
	}

	// Helper to format Cpk with warning indicator
	function formatCpk(value: unknown, entity: EntityData): string {
		const results = entity.data?.analysis_results as AnalysisResults | undefined;
		const cpk = results?.rss?.cpk;
		if (cpk === undefined || cpk === null) return '-';
		// Add warning for low Cpk (< 1.33 is typically minimum acceptable)
		if (cpk < 1.0) {
			return `⚠ ${cpk.toFixed(2)}`;
		}
		return cpk.toFixed(2);
	}

	// Helper to format Yield
	function formatYield(value: unknown, entity: EntityData): string {
		const results = entity.data?.analysis_results as AnalysisResults | undefined;
		const yieldPct = results?.rss?.yield_percent;
		if (yieldPct === undefined || yieldPct === null) return '-';
		return `${yieldPct.toFixed(1)}%`;
	}

	// Count stackups with issues
	function countFailing(entities: EntityData[]): number {
		return entities.filter(e => {
			const results = e.data?.analysis_results as AnalysisResults | undefined;
			return results?.worst_case?.result?.toLowerCase() === 'fail';
		}).length;
	}

	function countMarginal(entities: EntityData[]): number {
		return entities.filter(e => {
			const results = e.data?.analysis_results as AnalysisResults | undefined;
			return results?.worst_case?.result?.toLowerCase() === 'marginal';
		}).length;
	}

	function countLowCpk(entities: EntityData[]): number {
		return entities.filter(e => {
			const results = e.data?.analysis_results as AnalysisResults | undefined;
			const cpk = results?.rss?.cpk;
			return cpk !== undefined && cpk !== null && cpk < 1.33;
		}).length;
	}

	const columns = [
		{ key: 'id', label: 'ID', sortable: true, class: 'font-mono text-xs w-40' },
		{ key: 'title', label: 'Stackup', sortable: true },
		{
			key: 'data.analysis_results.worst_case.result',
			label: 'WC Result',
			sortable: true,
			class: 'w-24',
			render: formatWcResult
		},
		{
			key: 'data.analysis_results.rss.cpk',
			label: 'Cpk',
			class: 'w-20 font-mono',
			render: formatCpk
		},
		{
			key: 'data.analysis_results.rss.yield_percent',
			label: 'Yield',
			class: 'w-20',
			render: formatYield
		},
		{ key: 'status', label: 'Status', sortable: true, class: 'w-24' }
	];

	async function loadData() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			const result = await entities.list('TOL');
			entitiesData = result.items;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load tolerances:', e);
		} finally {
			loading = false;
		}
	}

	function handleRowClick(entity: EntityData) {
		goto(`/tolerances/${entity.id}`);
	}

	onMount(() => {
		loadData();
	});

	$effect(() => {
		if ($isProjectOpen) {
			loadData();
		}
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Tolerance Stackups</h1>
			<p class="text-muted-foreground">Dimensional tolerance analysis</p>
		</div>
		<Button onclick={() => goto('/tolerances/new')}>New Stackup</Button>
	</div>

	<!-- Stats -->
	<div class="grid gap-4 md:grid-cols-5">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Total Stackups</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{entitiesData.length}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">WC Failing</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold" class:text-destructive={countFailing(entitiesData) > 0}>
					{countFailing(entitiesData)}
					{#if countFailing(entitiesData) > 0}
						<span class="text-sm font-normal ml-1">⚠</span>
					{/if}
				</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">WC Marginal</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold" class:text-yellow-500={countMarginal(entitiesData) > 0}>
					{countMarginal(entitiesData)}
				</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Low Cpk (&lt;1.33)</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold" class:text-yellow-500={countLowCpk(entitiesData) > 0}>
					{countLowCpk(entitiesData)}
				</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Released</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-400">{entitiesData.filter(e => e.status === 'released').length}</div>
			</CardContent>
		</Card>
	</div>

	<!-- Error display -->
	{#if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{/if}

	<!-- Entity table -->
	{#if !$isProjectOpen}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Open a project to view tolerance stackups</p>
			</CardContent>
		</Card>
	{:else}
		<EntityTable
			entities={entitiesData}
			{columns}
			{loading}
			searchPlaceholder="Search stackups..."
			onRowClick={handleRowClick}
		/>
	{/if}
</div>
