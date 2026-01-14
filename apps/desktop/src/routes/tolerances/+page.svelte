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

	// Helper to format WC result
	function formatWcResult(value: unknown, entity: EntityData): string {
		const results = entity.data?.analysis_results as { worst_case?: { result?: string } } | undefined;
		const result = results?.worst_case?.result;
		if (!result) return '-';
		return result.charAt(0).toUpperCase() + result.slice(1);
	}

	// Helper to format Cpk
	function formatCpk(value: unknown, entity: EntityData): string {
		const results = entity.data?.analysis_results as { rss?: { cpk?: number } } | undefined;
		const cpk = results?.rss?.cpk;
		if (cpk === undefined || cpk === null) return '-';
		return cpk.toFixed(2);
	}

	// Helper to format Yield
	function formatYield(value: unknown, entity: EntityData): string {
		const results = entity.data?.analysis_results as { rss?: { yield_percent?: number } } | undefined;
		const yieldPct = results?.rss?.yield_percent;
		if (yieldPct === undefined || yieldPct === null) return '-';
		return `${yieldPct.toFixed(1)}%`;
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
	<div class="grid gap-4 md:grid-cols-3">
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
				<CardTitle class="text-sm font-medium text-muted-foreground">Draft</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{entitiesData.filter(e => e.status === 'draft').length}</div>
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
