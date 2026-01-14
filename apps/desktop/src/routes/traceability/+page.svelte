<script lang="ts">
	import { onMount } from 'svelte';
	import {
		Card,
		CardContent,
		CardHeader,
		CardTitle,
		Input,
		Button,
		Badge
	} from '$lib/components/ui';
	import { TraceGraph } from '$lib/components/traceability';
	import { traceability, entities } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import type { TraceResult, EntityData } from '$lib/api/types';
	import { Search, GitBranch, AlertCircle, RefreshCw } from 'lucide-svelte';

	let searchQuery = $state('');
	let searchResults = $state<EntityData[]>([]);
	let selectedEntityId = $state<string | null>(null);
	let traceResult = $state<TraceResult | null>(null);
	let orphans = $state<string[]>([]);
	let cycles = $state<string[][]>([]);
	let loading = $state(false);
	let searching = $state(false);
	let error = $state<string | null>(null);

	async function searchEntities() {
		if (!searchQuery.trim() || !$isProjectOpen) return;

		searching = true;
		error = null;

		try {
			// Search across multiple entity types
			const results = await entities.list('REQ', { search: searchQuery, limit: 10 });
			searchResults = results.items;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			searching = false;
		}
	}

	async function selectEntity(id: string) {
		selectedEntityId = id;
		loading = true;
		error = null;

		try {
			const result = await traceability.traceFrom({ id, depth: 1 });
			traceResult = result;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	async function loadAnalysis() {
		if (!$isProjectOpen) return;

		loading = true;
		try {
			const [orphanResult, cycleResult] = await Promise.all([
				traceability.findOrphans(),
				traceability.findCycles()
			]);
			orphans = orphanResult;
			cycles = cycleResult;
		} catch (e) {
			console.error('Failed to load analysis:', e);
		} finally {
			loading = false;
		}
	}

	function handleEntitySelect(id: string) {
		selectEntity(id);
	}

	onMount(() => {
		loadAnalysis();
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Trace Explorer</h1>
			<p class="text-muted-foreground">Explore entity relationships and traceability</p>
		</div>
		<Button variant="outline" onclick={loadAnalysis} disabled={loading}>
			<RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
			Refresh Analysis
		</Button>
	</div>

	<!-- Search -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<Search class="h-5 w-5" />
				Find Entity
			</CardTitle>
		</CardHeader>
		<CardContent>
			<div class="flex gap-4">
				<Input
					type="text"
					placeholder="Search by ID or title..."
					bind:value={searchQuery}
					onkeydown={(e) => e.key === 'Enter' && searchEntities()}
					class="flex-1"
				/>
				<Button onclick={searchEntities} disabled={searching}>
					{searching ? 'Searching...' : 'Search'}
				</Button>
			</div>

			{#if searchResults.length > 0}
				<div class="mt-4 space-y-2">
					{#each searchResults as entity}
						<button
							class="flex w-full items-center justify-between rounded-lg border p-3 text-left transition-colors hover:bg-muted/50 {selectedEntityId === entity.id ? 'border-primary bg-primary/5' : ''}"
							onclick={() => selectEntity(entity.id)}
						>
							<div>
								<p class="font-medium">{entity.title}</p>
								<p class="font-mono text-sm text-muted-foreground">{entity.id}</p>
							</div>
							<Badge variant="outline">{entity.prefix}</Badge>
						</button>
					{/each}
				</div>
			{/if}
		</CardContent>
	</Card>

	<!-- Trace Graph -->
	{#if traceResult}
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<GitBranch class="h-5 w-5" />
					Traceability Graph
				</CardTitle>
			</CardHeader>
			<CardContent>
				{#if loading}
					<div class="flex h-64 items-center justify-center">
						<div class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
					</div>
				{:else if error}
					<div class="flex h-64 items-center justify-center text-destructive">
						<AlertCircle class="mr-2 h-5 w-5" />
						{error}
					</div>
				{:else}
					<TraceGraph {traceResult} onSelectEntity={handleEntitySelect} />
				{/if}
			</CardContent>
		</Card>
	{/if}

	<!-- Analysis Results -->
	<div class="grid gap-6 lg:grid-cols-2">
		<!-- Orphans -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<AlertCircle class="h-5 w-5 text-yellow-500" />
					Orphaned Entities
					{#if orphans.length > 0}
						<Badge variant="secondary">{orphans.length}</Badge>
					{/if}
				</CardTitle>
			</CardHeader>
			<CardContent>
				{#if orphans.length === 0}
					<p class="text-center text-muted-foreground">No orphaned entities found</p>
				{:else}
					<div class="max-h-64 space-y-2 overflow-auto">
						{#each orphans as id}
							<button
								class="flex w-full items-center justify-between rounded-lg border p-2 text-left transition-colors hover:bg-muted/50"
								onclick={() => selectEntity(id)}
							>
								<span class="font-mono text-sm">{id}</span>
								<Badge variant="outline">{id.split('-')[0]}</Badge>
							</button>
						{/each}
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Cycles -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<RefreshCw class="h-5 w-5 text-red-500" />
					Circular Dependencies
					{#if cycles.length > 0}
						<Badge variant="destructive">{cycles.length}</Badge>
					{/if}
				</CardTitle>
			</CardHeader>
			<CardContent>
				{#if cycles.length === 0}
					<p class="text-center text-muted-foreground">No circular dependencies found</p>
				{:else}
					<div class="max-h-64 space-y-2 overflow-auto">
						{#each cycles as cycle, i}
							<div class="rounded-lg border p-2">
								<p class="mb-1 text-xs text-muted-foreground">Cycle {i + 1}</p>
								<div class="flex flex-wrap gap-1">
									{#each cycle as id}
										<Badge variant="outline" class="font-mono text-xs">{id.split('-')[0]}</Badge>
									{/each}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</CardContent>
		</Card>
	</div>
</div>
