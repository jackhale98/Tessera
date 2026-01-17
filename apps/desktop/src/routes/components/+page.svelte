<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { FilterPanel } from '$lib/components/entities';
	import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '$lib/components/ui';
	import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui';
	import { Input } from '$lib/components/ui';
	import { Search } from 'lucide-svelte';
	import { components } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import { componentsFilterConfig } from '$lib/config/filters';
	import type { FilterState } from '$lib/api/types';
	import type { ComponentSummary, ComponentStats, BomCostSummary, ListComponentsParams } from '$lib/api/tauri';

	let componentsData = $state<ComponentSummary[]>([]);
	let stats = $state<ComponentStats | null>(null);
	let costSummary = $state<BomCostSummary | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let searchQuery = $state('');
	let currentFilters = $state<FilterState>({});

	// Sort state
	let sortColumn = $state<string | null>(null);
	let sortDirection = $state<'asc' | 'desc'>('asc');

	// Filter and sort components
	const filteredComponents = $derived(() => {
		let result = componentsData;

		// Apply client-side search (backend search is also available)
		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			result = result.filter(
				(c) =>
					c.title.toLowerCase().includes(query) ||
					c.id.toLowerCase().includes(query) ||
					c.part_number.toLowerCase().includes(query) ||
					c.tags.some((t) => t.toLowerCase().includes(query))
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

				// Numeric comparison for cost and mass
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
			const params: ListComponentsParams = {};

			// Map filter state to API params
			if (currentFilters.status && Array.isArray(currentFilters.status)) {
				params.status = currentFilters.status as string[];
			}
			if (currentFilters.category) {
				params.category = currentFilters.category as string;
			}
			if (currentFilters.make_buy) {
				params.make_buy = currentFilters.make_buy as string;
			}
			// Note: long_lead_only and single_source_only would need backend support
			// For now they're available in the filter UI but won't filter server-side

			const [componentsResult, statsResult, costResult] = await Promise.all([
				components.list(params),
				components.getStats(),
				components.getBomCostSummary()
			]);

			componentsData = componentsResult.items;
			stats = statsResult;
			costSummary = costResult;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load components:', e);
		} finally {
			loading = false;
		}
	}

	function handleFiltersChange(filters: FilterState) {
		currentFilters = filters;
		loadData();
	}

	function handleRowClick(component: ComponentSummary) {
		goto(`/components/${component.id}`);
	}

	function handleSort(column: string) {
		if (sortColumn === column) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortColumn = column;
			sortDirection = 'asc';
		}
	}

	function formatCurrency(value: number | null | undefined): string {
		if (value == null) return '-';
		return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(value);
	}

	function formatMass(value: number | null | undefined): string {
		if (value == null) return '-';
		if (value < 1) return `${(value * 1000).toFixed(0)} g`;
		return `${value.toFixed(2)} kg`;
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

	function getMakeBuyVariant(makeBuy: string): 'default' | 'secondary' | 'outline' {
		switch (makeBuy) {
			case 'make':
				return 'default';
			case 'buy':
				return 'secondary';
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
			<h1 class="text-2xl font-bold">Bill of Materials</h1>
			<p class="text-muted-foreground">Component and assembly management</p>
		</div>
		<Button onclick={() => goto('/components/new')}>New Component</Button>
	</div>

	<!-- Stats cards -->
	{#if stats && costSummary}
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
					<CardTitle class="text-sm font-medium text-muted-foreground">Make</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.make_count}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Buy</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.buy_count}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Total Cost</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{formatCurrency(costSummary.total_cost)}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Total Mass</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{formatMass(stats.total_mass)}</div>
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
				<p class="text-muted-foreground">Open a project to view components</p>
			</CardContent>
		</Card>
	{:else}
		<div class="space-y-4">
			<!-- Filter Panel -->
			<FilterPanel
				fields={componentsFilterConfig.fields}
				quickFilters={componentsFilterConfig.quickFilters}
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
						placeholder="Search components..."
						bind:value={searchQuery}
						class="pl-9"
					/>
				</div>
				<div class="text-sm text-muted-foreground">
					{filteredComponents().length} of {componentsData.length} items
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
								class="cursor-pointer select-none w-24 hover:bg-muted/50"
								onclick={() => handleSort('category')}
							>
								<div class="flex items-center gap-1">
									Category
									{#if sortColumn === 'category'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-24 hover:bg-muted/50"
								onclick={() => handleSort('make_buy')}
							>
								<div class="flex items-center gap-1">
									Make/Buy
									{#if sortColumn === 'make_buy'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-28 text-right hover:bg-muted/50"
								onclick={() => handleSort('unit_cost')}
							>
								<div class="flex items-center justify-end gap-1">
									Unit Cost
									{#if sortColumn === 'unit_cost'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-24 text-right hover:bg-muted/50"
								onclick={() => handleSort('mass_kg')}
							>
								<div class="flex items-center justify-end gap-1">
									Mass
									{#if sortColumn === 'mass_kg'}
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
						</TableRow>
					</TableHeader>
					<TableBody>
						{#if loading}
							<TableRow>
								<TableCell colspan={7} class="h-24 text-center">
									<div class="flex items-center justify-center gap-2">
										<div class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
										Loading...
									</div>
								</TableCell>
							</TableRow>
						{:else if filteredComponents().length === 0}
							<TableRow>
								<TableCell colspan={7} class="h-24 text-center text-muted-foreground">
									No components found
								</TableCell>
							</TableRow>
						{:else}
							{#each filteredComponents() as component (component.id)}
								<TableRow
									class="cursor-pointer"
									onclick={() => handleRowClick(component)}
								>
									<TableCell class="font-mono text-xs">{component.part_number}</TableCell>
									<TableCell>{component.title}</TableCell>
									<TableCell class="capitalize">{component.category}</TableCell>
									<TableCell>
										<Badge variant={getMakeBuyVariant(component.make_buy)} class="capitalize">
											{component.make_buy}
										</Badge>
									</TableCell>
									<TableCell class="text-right">{formatCurrency(component.unit_cost)}</TableCell>
									<TableCell class="text-right">{formatMass(component.mass_kg)}</TableCell>
									<TableCell>
										<Badge variant={getStatusVariant(component.status)} class="capitalize">
											{component.status}
										</Badge>
									</TableCell>
								</TableRow>
							{/each}
						{/if}
					</TableBody>
				</Table>
			</div>
		</div>
	{/if}
</div>
