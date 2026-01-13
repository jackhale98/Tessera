<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { EntityTable } from '$lib/components/entities';
	import { Card, CardContent, CardHeader, CardTitle, Button } from '$lib/components/ui';
	import { entities, requirements } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import type { EntityData, EntityListResult } from '$lib/api/types';
	import type { RequirementStats } from '$lib/api/tauri';

	let entitiesData = $state<EntityData[]>([]);
	let stats = $state<RequirementStats | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const columns = [
		{ key: 'id', label: 'ID', sortable: true, class: 'font-mono text-xs w-48' },
		{ key: 'title', label: 'Title', sortable: true },
		{ key: 'status', label: 'Status', sortable: true, class: 'w-24' },
		{ key: 'author', label: 'Author', sortable: true, class: 'w-32' },
		{ key: 'tags', label: 'Tags', class: 'w-40' }
	];

	async function loadData() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			const [entityResult, statsResult] = await Promise.all([
				entities.list('REQ'),
				requirements.getStats()
			]);

			entitiesData = entityResult.items;
			stats = statsResult;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load requirements:', e);
		} finally {
			loading = false;
		}
	}

	function handleRowClick(entity: EntityData) {
		goto(`/requirements/${entity.id}`);
	}

	onMount(() => {
		loadData();
	});

	// Reload when project opens
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
			<h1 class="text-2xl font-bold">Requirements</h1>
			<p class="text-muted-foreground">Manage system and stakeholder requirements</p>
		</div>
		<Button>New Requirement</Button>
	</div>

	<!-- Stats cards -->
	{#if stats}
		<div class="grid gap-4 md:grid-cols-5">
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Total</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.total}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Inputs</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.inputs}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Outputs</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.outputs}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Unverified</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-warning">{stats.unverified}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Orphaned</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-destructive">{stats.orphaned}</div>
				</CardContent>
			</Card>
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
				<p class="text-muted-foreground">Open a project to view requirements</p>
			</CardContent>
		</Card>
	{:else}
		<EntityTable
			entities={entitiesData}
			{columns}
			{loading}
			searchPlaceholder="Search requirements..."
			onRowClick={handleRowClick}
		/>
	{/if}
</div>
