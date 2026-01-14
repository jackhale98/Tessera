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
		{ key: 'title', label: 'Deviation', sortable: true },
		{ key: 'status', label: 'Status', sortable: true, class: 'w-24' },
		{ key: 'author', label: 'Author', sortable: true, class: 'w-32' }
	];

	async function loadData() {
		if (!$isProjectOpen) return;
		loading = true;
		error = null;
		try {
			const result = await entities.list('DEV');
			entitiesData = result.items;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function handleRowClick(entity: EntityData) {
		goto(`/manufacturing/deviations/${entity.id}`);
	}

	onMount(() => { loadData(); });
	$effect(() => { if ($isProjectOpen) loadData(); });
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Deviations</h1>
			<p class="text-muted-foreground">Process and product deviations</p>
		</div>
		<Button onclick={() => goto('/manufacturing/deviations/new')}>New Deviation</Button>
	</div>

	<div class="grid gap-4 md:grid-cols-3">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Total Deviations</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{entitiesData.length}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Open</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-orange-400">{entitiesData.filter(e => e.status === 'draft' || e.status === 'review').length}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Closed</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-400">{entitiesData.filter(e => e.status === 'released' || e.status === 'approved').length}</div>
			</CardContent>
		</Card>
	</div>

	{#if error}
		<Card class="border-destructive">
			<CardContent class="pt-6"><p class="text-destructive">{error}</p></CardContent>
		</Card>
	{/if}

	{#if !$isProjectOpen}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Open a project to view deviations</p>
			</CardContent>
		</Card>
	{:else}
		<EntityTable {columns} entities={entitiesData} {loading} searchPlaceholder="Search deviations..." onRowClick={handleRowClick} />
	{/if}
</div>
