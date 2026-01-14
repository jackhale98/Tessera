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
		{ key: 'title', label: 'Work Instruction', sortable: true },
		{ key: 'status', label: 'Status', sortable: true, class: 'w-24' },
		{ key: 'author', label: 'Author', sortable: true, class: 'w-32' }
	];

	async function loadData() {
		if (!$isProjectOpen) return;
		loading = true;
		error = null;
		try {
			const result = await entities.list('WORK');
			entitiesData = result.items;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function handleRowClick(entity: EntityData) {
		goto(`/manufacturing/work-instructions/${entity.id}`);
	}

	onMount(() => { loadData(); });
	$effect(() => { if ($isProjectOpen) loadData(); });
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Work Instructions</h1>
			<p class="text-muted-foreground">Step-by-step manufacturing procedures</p>
		</div>
		<Button onclick={() => goto('/manufacturing/work-instructions/new')}>New Work Instruction</Button>
	</div>

	<div class="grid gap-4 md:grid-cols-3">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Total Instructions</CardTitle>
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

	{#if error}
		<Card class="border-destructive">
			<CardContent class="pt-6"><p class="text-destructive">{error}</p></CardContent>
		</Card>
	{/if}

	{#if !$isProjectOpen}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Open a project to view work instructions</p>
			</CardContent>
		</Card>
	{:else}
		<EntityTable {columns} entities={entitiesData} {loading} searchPlaceholder="Search work instructions..." onRowClick={handleRowClick} />
	{/if}
</div>
