<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { FilterPanel } from '$lib/components/entities';
	import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '$lib/components/ui';
	import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui';
	import { Input } from '$lib/components/ui';
	import { cn } from '$lib/utils/cn';
	import { Search } from 'lucide-svelte';
	import { requirements } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import { requirementsFilterConfig } from '$lib/config/filters';
	import type { FilterState } from '$lib/api/types';
	import type { RequirementStats, RequirementSummary, ListRequirementsParams } from '$lib/api/tauri';

	let requirementsData = $state<RequirementSummary[]>([]);
	let stats = $state<RequirementStats | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let searchQuery = $state('');
	let currentFilters = $state<FilterState>({});

	// Sort state
	let sortColumn = $state<string | null>(null);
	let sortDirection = $state<'asc' | 'desc'>('asc');

	// Filter and sort requirements
	const filteredRequirements = $derived(() => {
		let result = requirementsData;

		// Apply client-side search (backend search is also available)
		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			result = result.filter(
				(r) =>
					r.title.toLowerCase().includes(query) ||
					r.id.toLowerCase().includes(query) ||
					r.tags.some((t) => t.toLowerCase().includes(query))
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
			const params: ListRequirementsParams = {};

			// Map filter state to API params
			if (currentFilters.status && Array.isArray(currentFilters.status)) {
				params.status = currentFilters.status as string[];
			}
			if (currentFilters.req_type) {
				params.req_type = currentFilters.req_type as string;
			}
			if (currentFilters.level) {
				params.level = currentFilters.level as string;
			}
			if (currentFilters.priority) {
				params.priority = currentFilters.priority as string;
			}
			if (currentFilters.orphans_only) {
				params.orphans_only = currentFilters.orphans_only as boolean;
			}
			if (currentFilters.unverified_only) {
				params.unverified_only = currentFilters.unverified_only as boolean;
			}

			const [listResult, statsResult] = await Promise.all([
				requirements.list(params),
				requirements.getStats()
			]);

			requirementsData = listResult.items;
			stats = statsResult;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load requirements:', e);
		} finally {
			loading = false;
		}
	}

	function handleFiltersChange(filters: FilterState) {
		currentFilters = filters;
		loadData();
	}

	function handleRowClick(req: RequirementSummary) {
		goto(`/requirements/${req.id}`);
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

	function getPriorityVariant(priority: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (priority) {
			case 'critical':
				return 'destructive';
			case 'high':
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
			<h1 class="text-2xl font-bold">Requirements</h1>
			<p class="text-muted-foreground">Manage system and stakeholder requirements</p>
		</div>
		<Button onclick={() => goto('/requirements/new')}>New Requirement</Button>
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
					<div class="text-2xl font-bold text-orange-500">{stats.unverified}</div>
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

	<!-- Main content -->
	{#if !$isProjectOpen}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Open a project to view requirements</p>
			</CardContent>
		</Card>
	{:else}
		<div class="space-y-4">
			<!-- Filter Panel -->
			<FilterPanel
				fields={requirementsFilterConfig.fields}
				quickFilters={requirementsFilterConfig.quickFilters}
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
						placeholder="Search requirements..."
						bind:value={searchQuery}
						class="pl-9"
					/>
				</div>
				<div class="text-sm text-muted-foreground">
					{filteredRequirements().length} of {requirementsData.length} items
				</div>
			</div>

			<!-- Table -->
			<div class="rounded-md border">
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead
								class="cursor-pointer select-none font-mono text-xs w-48 hover:bg-muted/50"
								onclick={() => handleSort('id')}
							>
								<div class="flex items-center gap-1">
									ID
									{#if sortColumn === 'id'}
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
								class="cursor-pointer select-none w-20 hover:bg-muted/50"
								onclick={() => handleSort('req_type')}
							>
								<div class="flex items-center gap-1">
									Type
									{#if sortColumn === 'req_type'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-24 hover:bg-muted/50"
								onclick={() => handleSort('level')}
							>
								<div class="flex items-center gap-1">
									Level
									{#if sortColumn === 'level'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-20 hover:bg-muted/50"
								onclick={() => handleSort('priority')}
							>
								<div class="flex items-center gap-1">
									Priority
									{#if sortColumn === 'priority'}
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
							<TableHead class="w-32">Author</TableHead>
							<TableHead class="w-40">Tags</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#if loading}
							<TableRow>
								<TableCell colspan={8} class="h-24 text-center">
									<div class="flex items-center justify-center gap-2">
										<div class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
										Loading...
									</div>
								</TableCell>
							</TableRow>
						{:else if filteredRequirements().length === 0}
							<TableRow>
								<TableCell colspan={8} class="h-24 text-center text-muted-foreground">
									No requirements found
								</TableCell>
							</TableRow>
						{:else}
							{#each filteredRequirements() as req (req.id)}
								<TableRow
									class="cursor-pointer"
									onclick={() => handleRowClick(req)}
								>
									<TableCell class="font-mono text-xs">{req.id}</TableCell>
									<TableCell>{req.title}</TableCell>
									<TableCell>
										<Badge variant="outline" class="text-xs capitalize">
											{req.req_type}
										</Badge>
									</TableCell>
									<TableCell class="capitalize">{req.level}</TableCell>
									<TableCell>
										<Badge variant={getPriorityVariant(req.priority)} class="text-xs capitalize">
											{req.priority}
										</Badge>
									</TableCell>
									<TableCell>
										<Badge variant={getStatusVariant(req.status)} class="capitalize">
											{req.status}
										</Badge>
									</TableCell>
									<TableCell>{req.author}</TableCell>
									<TableCell>
										<div class="flex flex-wrap gap-1">
											{#each req.tags.slice(0, 3) as tag}
												<Badge variant="outline" class="text-xs">
													{tag}
												</Badge>
											{/each}
											{#if req.tags.length > 3}
												<Badge variant="outline" class="text-xs">
													+{req.tags.length - 3}
												</Badge>
											{/if}
										</div>
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
