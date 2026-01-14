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

	const columns = [
		{ key: 'id', label: 'ID', sortable: true, class: 'font-mono text-xs w-48' },
		{ key: 'title', label: 'Hazard', sortable: true },
		{ key: 'status', label: 'Status', sortable: true, class: 'w-24' },
		{ key: 'author', label: 'Author', sortable: true, class: 'w-32' },
		{ key: 'tags', label: 'Tags', class: 'w-40' }
	];

	async function loadData() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			const result = await entities.list('HAZ');
			entitiesData = result.items;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load hazards:', e);
		} finally {
			loading = false;
		}
	}

	function handleRowClick(entity: EntityData) {
		goto(`/hazards/${entity.id}`);
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
			<h1 class="text-2xl font-bold">Hazard Analysis</h1>
			<p class="text-muted-foreground">Safety hazard identification and analysis</p>
		</div>
		<Button onclick={() => goto('/hazards/new')}>New Hazard</Button>
	</div>

	<!-- Stats -->
	<div class="grid gap-4 md:grid-cols-4">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Total Hazards</CardTitle>
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
				<CardTitle class="text-sm font-medium text-muted-foreground">In Review</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-yellow-400">{entitiesData.filter(e => e.status === 'review').length}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Approved</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-400">{entitiesData.filter(e => e.status === 'approved').length}</div>
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
				<p class="text-muted-foreground">Open a project to view hazards</p>
			</CardContent>
		</Card>
	{:else}
		<EntityTable
			entities={entitiesData}
			{columns}
			{loading}
			searchPlaceholder="Search hazards..."
			onRowClick={handleRowClick}
		/>
	{/if}
</div>
