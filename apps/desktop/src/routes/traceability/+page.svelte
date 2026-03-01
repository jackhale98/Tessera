<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import {
		Card,
		CardContent,
		CardHeader,
		CardTitle,
		Button,
		Badge
	} from '$lib/components/ui';
	import { EntityPicker } from '$lib/components/common';
	import { TraceGraph } from '$lib/components/traceability';
	import { traceability } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import { ALL_ENTITY_TYPES, getEntityRoute } from '$lib/config/entities';
	import type { TraceResult, CycleEntity } from '$lib/api/types';
	import { Search, GitBranch, AlertCircle, RefreshCw, ArrowRight } from 'lucide-svelte';

	let selectedEntityId = $state<string | null>(null);
	let traceResult = $state<TraceResult | null>(null);
	let orphans = $state<string[]>([]);
	let cycles = $state<CycleEntity[][]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);

	async function selectEntity(id: string) {
		selectedEntityId = id;
		loading = true;
		error = null;

		try {
			const result = await traceability.traceFrom({ id });
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

	function navigateToEntity(entityType: string, id: string) {
		goto(getEntityRoute(entityType, id));
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
			<EntityPicker
				entityTypes={[...ALL_ENTITY_TYPES]}
				placeholder="Search by ID or title..."
				onSelect={(entity) => selectEntity(entity.id)}
				onClear={() => { selectedEntityId = null; traceResult = null; }}
			/>
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
					<TraceGraph {traceResult} />
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
					<div class="max-h-80 space-y-3 overflow-auto">
						{#each cycles as cycle, i}
							<div class="rounded-lg border p-3">
								<p class="mb-2 text-xs font-medium text-muted-foreground">Cycle {i + 1}</p>
								<div class="flex flex-wrap items-center gap-1">
									{#each cycle as entity, j}
										<button
											class="inline-flex items-center gap-1 rounded-md border px-2 py-1 text-xs font-mono transition-colors hover:bg-muted/50"
											onclick={() => navigateToEntity(entity.entity_type, entity.id)}
											title={entity.title}
										>
											<Badge variant="outline" class="text-xs px-1 py-0">{entity.entity_type}</Badge>
											<span class="truncate max-w-32">{entity.title || entity.id}</span>
										</button>
										{#if j < cycle.length - 1}
											<ArrowRight class="h-3 w-3 text-muted-foreground shrink-0" />
										{:else}
											<ArrowRight class="h-3 w-3 text-destructive shrink-0" />
										{/if}
									{/each}
									<!-- Arrow back to first entity to show the cycle -->
									{#if cycle.length > 0}
										<Badge variant="outline" class="text-xs px-1 py-0 border-destructive text-destructive">{cycle[0].entity_type}</Badge>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</CardContent>
		</Card>
	</div>
</div>
