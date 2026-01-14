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

	// Helper to get mate type label
	function formatMateType(value: unknown): string {
		if (typeof value !== 'string') return '-';
		return value.charAt(0).toUpperCase() + value.slice(1);
	}

	// Helper to format clearance value
	function formatClearance(value: unknown): string {
		if (typeof value !== 'number') return '-';
		return `${value >= 0 ? '+' : ''}${value.toFixed(4)}`;
	}

	// Helper to format fit result with pass/fail indication
	function formatFitResult(value: unknown, entity: EntityData): string {
		const fitAnalysis = entity.data?.fit_analysis as { fit_result?: string } | undefined;
		const result = fitAnalysis?.fit_result;
		if (!result) return '-';
		// Return the fit result - display will show color via cell content
		return result.charAt(0).toUpperCase() + result.slice(1);
	}

	const columns = [
		{ key: 'id', label: 'ID', sortable: true, class: 'font-mono text-xs w-40' },
		{ key: 'title', label: 'Mate', sortable: true },
		{
			key: 'data.mate_type',
			label: 'Target',
			sortable: true,
			class: 'w-24',
			render: formatMateType
		},
		{
			key: 'data.fit_analysis.worst_case_min_clearance',
			label: 'Min (WC)',
			class: 'w-24 font-mono text-xs',
			render: formatClearance
		},
		{
			key: 'data.fit_analysis.worst_case_max_clearance',
			label: 'Max (WC)',
			class: 'w-24 font-mono text-xs',
			render: formatClearance
		},
		{
			key: 'data.fit_analysis.fit_result',
			label: 'Result',
			sortable: true,
			class: 'w-24',
			render: formatFitResult
		},
		{ key: 'status', label: 'Status', sortable: true, class: 'w-24' }
	];

	async function loadData() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			const result = await entities.list('MATE');
			entitiesData = result.items;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load mates:', e);
		} finally {
			loading = false;
		}
	}

	function handleRowClick(entity: EntityData) {
		goto(`/mates/${entity.id}`);
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
			<h1 class="text-2xl font-bold">Assembly Mates</h1>
			<p class="text-muted-foreground">Feature-to-feature assembly constraints</p>
		</div>
		<Button onclick={() => goto('/mates/new')}>New Mate</Button>
	</div>

	<!-- Stats -->
	<div class="grid gap-4 md:grid-cols-3">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Total Mates</CardTitle>
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
				<p class="text-muted-foreground">Open a project to view mates</p>
			</CardContent>
		</Card>
	{:else}
		<EntityTable
			entities={entitiesData}
			{columns}
			{loading}
			searchPlaceholder="Search mates..."
			onRowClick={handleRowClick}
		/>
	{/if}
</div>
