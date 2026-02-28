<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { EntityTable } from '$lib/components/entities';
	import { Card, CardContent, CardHeader, CardTitle, Button, Select } from '$lib/components/ui';
	import { entities } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import type { EntityData } from '$lib/api/types';

	let entitiesData = $state<EntityData[]>([]);
	let componentsData = $state<EntityData[]>([]);
	let componentMap = $state<Record<string, string>>({});
	let selectedComponent = $state('');
	let loading = $state(true);
	let error = $state<string | null>(null);

	const columns = [
		{ key: 'id', label: 'ID', sortable: true, class: 'font-mono text-xs w-48' },
		{ key: 'title', label: 'Feature', sortable: true },
		{ key: 'data.component_title', label: 'Component', sortable: true, class: 'w-40' },
		{ key: 'data.nominal_display', label: 'Nominal', sortable: true, class: 'w-28 font-mono text-xs' },
		{ key: 'data.tolerance_display', label: 'Tolerance', class: 'w-32 font-mono text-xs' },
		{ key: 'status', label: 'Status', sortable: true, class: 'w-24' },
		{ key: 'author', label: 'Author', sortable: true, class: 'w-32' }
	];

	async function loadData() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			const [featResult, cmpResult] = await Promise.all([
				entities.list('FEAT', { include_data: true } as any),
				entities.list('CMP')
			]);

			// Build component id → title map
			const map: Record<string, string> = {};
			for (const cmp of cmpResult.items) {
				map[cmp.id] = cmp.title;
			}
			componentMap = map;
			componentsData = cmpResult.items;

			// Enrich features with component_title, nominal, and tolerance for table display
			entitiesData = featResult.items.map(f => {
				const data = (f.data ?? {}) as Record<string, unknown>;
				const dims = (data.dimensions as Array<Record<string, unknown>> | undefined)?.[0];
				const nominal = dims?.nominal;
				const plusTol = dims?.plus_tol;
				const minusTol = dims?.minus_tol;
				const units = dims?.units as string | undefined;
				let nominalDisplay = '';
				let toleranceDisplay = '';
				if (nominal !== undefined && nominal !== null) {
					nominalDisplay = `${nominal}${units ? ` ${units}` : ''}`;
				}
				if (plusTol !== undefined && minusTol !== undefined) {
					toleranceDisplay = `+${plusTol}/${minusTol}`;
				}
				return {
					...f,
					data: {
						...data,
						component_title: map[data.component as string] ?? '',
						nominal_display: nominalDisplay,
						tolerance_display: toleranceDisplay
					}
				};
			});
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load features:', e);
		} finally {
			loading = false;
		}
	}

	// Filter entities by selected component
	const filteredEntities = $derived(
		selectedComponent
			? entitiesData.filter(f => (f.data as Record<string, unknown>)?.component === selectedComponent)
			: entitiesData
	);

	function handleRowClick(entity: EntityData) {
		goto(`/features/${entity.id}`);
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
			<h1 class="text-2xl font-bold">Geometric Features</h1>
			<p class="text-muted-foreground">Part features for tolerance analysis</p>
		</div>
		<Button onclick={() => goto('/features/new')}>New Feature</Button>
	</div>

	<!-- Stats -->
	<div class="grid gap-4 md:grid-cols-3">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Total Features</CardTitle>
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

	<!-- Component filter -->
	{#if componentsData.length > 0}
		<div class="flex items-center gap-2">
			<span class="text-sm font-medium text-muted-foreground">Filter by component:</span>
			<Select bind:value={selectedComponent} class="w-64">
				<option value="">All Components</option>
				{#each componentsData as cmp}
					<option value={cmp.id}>{cmp.title}</option>
				{/each}
			</Select>
			{#if selectedComponent}
				<Button variant="ghost" size="sm" onclick={() => selectedComponent = ''}>Clear</Button>
			{/if}
		</div>
	{/if}

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
				<p class="text-muted-foreground">Open a project to view features</p>
			</CardContent>
		</Card>
	{:else}
		<EntityTable
			entities={filteredEntities}
			{columns}
			{loading}
			searchPlaceholder="Search features..."
			onRowClick={handleRowClick}
		/>
	{/if}
</div>
