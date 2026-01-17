<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { FilterPanel } from '$lib/components/entities';
	import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '$lib/components/ui';
	import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui';
	import { Input } from '$lib/components/ui';
	import { Search } from 'lucide-svelte';
	import { assemblies } from '$lib/api/tauri';
	import { isProjectOpen } from '$lib/stores/project';
	import { assembliesFilterConfig } from '$lib/config/filters';
	import type { FilterState } from '$lib/api/types';
	import type { AssemblySummary, AssemblyStats, ListAssembliesParams } from '$lib/api/tauri';

	let assembliesData = $state<AssemblySummary[]>([]);
	let stats = $state<AssemblyStats | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let searchQuery = $state('');
	let currentFilters = $state<FilterState>({});

	// Sort state
	let sortColumn = $state<string | null>(null);
	let sortDirection = $state<'asc' | 'desc'>('asc');

	// Filter and sort assemblies
	const filteredAssemblies = $derived(() => {
		let result = assembliesData;

		// Apply client-side search
		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			result = result.filter(
				(a) =>
					a.title.toLowerCase().includes(query) ||
					a.id.toLowerCase().includes(query) ||
					a.part_number.toLowerCase().includes(query)
			);
		}

		// Apply sorting
		if (sortColumn) {
			result = [...result].sort((a, b) => {
				const aVal = (a as unknown as Record<string, unknown>)[sortColumn!];
				const bVal = (b as unknown as Record<string, unknown>)[sortColumn!];

				if (aVal === bVal) return 0;
				if (aVal === null || aVal === undefined) return 1;
				if (bVal === null || bVal === undefined) return -1;

				// Numeric comparison for counts
				if (typeof aVal === 'number' && typeof bVal === 'number') {
					return sortDirection === 'asc' ? aVal - bVal : bVal - aVal;
				}

				const comparison = String(aVal).localeCompare(String(bVal));
				return sortDirection === 'asc' ? comparison : -comparison;
			});
		}

		return result;
	});

	async function loadData() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			// Build params from current filters
			const params: ListAssembliesParams = {};

			// Map filter state to API params
			if (currentFilters.status && Array.isArray(currentFilters.status)) {
				params.status = currentFilters.status as string[];
			}
			if (currentFilters.top_level_only) {
				params.top_level_only = currentFilters.top_level_only as boolean;
			}
			if (currentFilters.sub_only) {
				params.sub_only = currentFilters.sub_only as boolean;
			}
			if (currentFilters.has_subassemblies) {
				params.has_subassemblies = currentFilters.has_subassemblies as boolean;
			}
			if (currentFilters.empty_bom) {
				params.empty_bom = currentFilters.empty_bom as boolean;
			}

			const [assembliesResult, statsResult] = await Promise.all([
				assemblies.list(params),
				assemblies.getStats()
			]);

			assembliesData = assembliesResult.items;
			stats = statsResult;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load assemblies:', e);
		} finally {
			loading = false;
		}
	}

	function handleFiltersChange(filters: FilterState) {
		currentFilters = filters;
		loadData();
	}

	function handleRowClick(assembly: AssemblySummary) {
		goto(`/assemblies/${assembly.id}`);
	}

	function handleSort(column: string) {
		if (sortColumn === column) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortColumn = column;
			sortDirection = 'asc';
		}
	}

	function getStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (status) {
			case 'approved':
			case 'released':
				return 'default';
			case 'review':
				return 'secondary';
			case 'obsolete':
				return 'destructive';
			default:
				return 'outline';
		}
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
			<h1 class="text-2xl font-bold">Assemblies</h1>
			<p class="text-muted-foreground">Product assembly structures and BOM hierarchy</p>
		</div>
		<Button onclick={() => goto('/assemblies/new')}>New Assembly</Button>
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
					<CardTitle class="text-sm font-medium text-muted-foreground">Top Level</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.top_level}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Sub-assemblies</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.sub_assemblies}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">BOM Items</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.total_bom_items}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Empty BOM</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-orange-500">{stats.empty_bom}</div>
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

	<!-- Main content -->
	{#if !$isProjectOpen}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Open a project to view assemblies</p>
			</CardContent>
		</Card>
	{:else}
		<div class="space-y-4">
			<!-- Filter Panel -->
			<FilterPanel
				fields={assembliesFilterConfig.fields}
				quickFilters={assembliesFilterConfig.quickFilters}
				onFiltersChange={handleFiltersChange}
				collapsible={true}
				defaultExpanded={false}
			/>

			<!-- Search and count bar -->
			<div class="flex items-center gap-4">
				<div class="relative max-w-sm flex-1">
					<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
					<Input
						type="search"
						placeholder="Search assemblies..."
						bind:value={searchQuery}
						class="pl-9"
					/>
				</div>
				<div class="text-sm text-muted-foreground">
					{filteredAssemblies().length} of {assembliesData.length} items
				</div>
			</div>

			<!-- Table -->
			<div class="rounded-md border">
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead
								class="cursor-pointer select-none w-40 hover:bg-muted/50"
								onclick={() => handleSort('part_number')}
							>
								<div class="flex items-center gap-1">
									Part Number
									{#if sortColumn === 'part_number'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none hover:bg-muted/50"
								onclick={() => handleSort('title')}
							>
								<div class="flex items-center gap-1">
									Title
									{#if sortColumn === 'title'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-28 text-center hover:bg-muted/50"
								onclick={() => handleSort('bom_count')}
							>
								<div class="flex items-center justify-center gap-1">
									Components
									{#if sortColumn === 'bom_count'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-32 text-center hover:bg-muted/50"
								onclick={() => handleSort('subassembly_count')}
							>
								<div class="flex items-center justify-center gap-1">
									Sub-assemblies
									{#if sortColumn === 'subassembly_count'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-24 hover:bg-muted/50"
								onclick={() => handleSort('status')}
							>
								<div class="flex items-center gap-1">
									Status
									{#if sortColumn === 'status'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-32 hover:bg-muted/50"
								onclick={() => handleSort('author')}
							>
								<div class="flex items-center gap-1">
									Author
									{#if sortColumn === 'author'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#if loading}
							<TableRow>
								<TableCell colspan={6} class="h-24 text-center">
									<div class="flex items-center justify-center gap-2">
										<div class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
										Loading...
									</div>
								</TableCell>
							</TableRow>
						{:else if filteredAssemblies().length === 0}
							<TableRow>
								<TableCell colspan={6} class="h-24 text-center text-muted-foreground">
									No assemblies found
								</TableCell>
							</TableRow>
						{:else}
							{#each filteredAssemblies() as assembly (assembly.id)}
								<TableRow
									class="cursor-pointer"
									onclick={() => handleRowClick(assembly)}
								>
									<TableCell class="font-mono text-xs">{assembly.part_number}</TableCell>
									<TableCell>{assembly.title}</TableCell>
									<TableCell class="text-center">
										{#if assembly.bom_count > 0}
											<Badge variant="secondary">{assembly.bom_count}</Badge>
										{:else}
											<span class="text-muted-foreground">-</span>
										{/if}
									</TableCell>
									<TableCell class="text-center">
										{#if assembly.subassembly_count > 0}
											<Badge variant="outline">{assembly.subassembly_count}</Badge>
										{:else}
											<span class="text-muted-foreground">-</span>
										{/if}
									</TableCell>
									<TableCell>
										<Badge variant={getStatusVariant(assembly.status)} class="capitalize">
											{assembly.status}
										</Badge>
									</TableCell>
									<TableCell>{assembly.author}</TableCell>
								</TableRow>
							{/each}
						{/if}
					</TableBody>
				</Table>
			</div>
		</div>
	{/if}
</div>
