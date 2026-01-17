<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { EntityTable, FilterPanel } from '$lib/components/entities';
	import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '$lib/components/ui';
	import { lots, type LotSummary, type LotStats, type ListLotsParams } from '$lib/api/tauri';
	import { isProjectOpen } from '$lib/stores/project';
	import type { FilterFieldDefinition, FilterState, QuickFilter } from '$lib/api/types';

	let items = $state<LotSummary[]>([]);
	let stats = $state<LotStats | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let currentFilters = $state<FilterState>({});

	// Define filter fields for Lots
	const filterFields: FilterFieldDefinition[] = [
		{
			key: 'lot_status',
			label: 'Lot Status',
			type: 'select',
			options: [
				{ value: 'in_progress', label: 'In Progress' },
				{ value: 'on_hold', label: 'On Hold' },
				{ value: 'completed', label: 'Completed' },
				{ value: 'scrapped', label: 'Scrapped' }
			],
			placeholder: 'All statuses'
		},
		{
			key: 'product',
			label: 'Product',
			type: 'text',
			placeholder: 'Filter by product...'
		},
		{
			key: 'search',
			label: 'Search',
			type: 'text',
			placeholder: 'Search lots...'
		}
	];

	// Quick filters for common scenarios
	const quickFilters: QuickFilter[] = [
		{
			id: 'active',
			label: 'Active Lots',
			filters: { active_only: true }
		},
		{
			id: 'in_progress',
			label: 'In Progress',
			filters: { lot_status: 'in_progress' }
		},
		{
			id: 'on_hold',
			label: 'On Hold',
			filters: { lot_status: 'on_hold' }
		},
		{
			id: 'recent',
			label: 'Recent (30 days)',
			filters: { recent_days: 30 }
		}
	];

	// Format lot status with indicator
	function formatLotStatus(value: unknown): string {
		const status = (value as string)?.toLowerCase();
		if (status === 'in_progress') return '🔄 In Progress';
		if (status === 'on_hold') return '⏸️ On Hold';
		if (status === 'completed') return '✅ Completed';
		if (status === 'scrapped') return '🗑️ Scrapped';
		return status || '-';
	}

	// Format quantity
	function formatQuantity(value: unknown): string {
		if (!value) return '-';
		return (value as number).toLocaleString();
	}

	// Format date
	function formatDate(value: unknown): string {
		if (!value) return '-';
		const dateStr = value as string;
		try {
			return new Date(dateStr).toLocaleDateString('en-US', {
				year: 'numeric',
				month: 'short',
				day: 'numeric'
			});
		} catch {
			return dateStr;
		}
	}

	// Table columns with Lot-specific data and custom renderers
	const columns = [
		{ key: 'id', label: 'ID', sortable: true, class: 'font-mono text-xs w-40' },
		{ key: 'title', label: 'Lot', sortable: true },
		{ key: 'lot_number', label: 'Lot #', sortable: true, class: 'w-24 font-mono' },
		{ key: 'quantity', label: 'Qty', sortable: true, class: 'w-20', render: formatQuantity },
		{ key: 'lot_status', label: 'Status', sortable: true, class: 'w-28', render: formatLotStatus },
		{ key: 'start_date', label: 'Started', sortable: true, class: 'w-28', render: formatDate }
	];

	// Convert FilterState to ListLotsParams
	function buildParams(filters: FilterState): ListLotsParams {
		const params: ListLotsParams = {};

		if (filters.lot_status) params.lot_status = filters.lot_status as string;
		if (filters.product) params.product = filters.product as string;
		if (filters.search) params.search = filters.search as string;
		if (filters.active_only) params.active_only = filters.active_only as boolean;
		if (filters.recent_days) params.recent_days = filters.recent_days as number;

		return params;
	}

	async function loadData() {
		if (!$isProjectOpen) return;
		loading = true;
		error = null;
		try {
			const params = buildParams(currentFilters);
			const [listResult, statsResult] = await Promise.all([
				lots.list(params),
				lots.getStats()
			]);
			items = listResult.items;
			stats = statsResult;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function handleFiltersChange(filters: FilterState) {
		currentFilters = filters;
		loadData();
	}

	function handleRowClick(entity: { id: string }) {
		goto(`/manufacturing/lots/${entity.id}`);
	}

	onMount(() => { loadData(); });
	$effect(() => { if ($isProjectOpen) loadData(); });
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Production Lots</h1>
			<p class="text-muted-foreground">Batch/lot tracking and Device History Records</p>
		</div>
		<Button onclick={() => goto('/manufacturing/lots/new')}>New Lot</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid gap-4 md:grid-cols-5">
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Total</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{stats?.total ?? items.length}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">In Progress</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-blue-500">{stats?.by_status?.in_progress ?? 0}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">On Hold</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-yellow-500">{stats?.by_status?.on_hold ?? 0}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Completed</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-500">{stats?.by_status?.completed ?? 0}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Total Quantity</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{stats?.total_quantity?.toLocaleString() ?? 0}</div>
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
				<p class="text-muted-foreground">Open a project to view lots</p>
			</CardContent>
		</Card>
	{:else}
		<!-- Filter Panel -->
		<FilterPanel
			fields={filterFields}
			quickFilters={quickFilters}
			onFiltersChange={handleFiltersChange}
			collapsible={true}
			defaultExpanded={false}
		/>

		<!-- Data Table -->
		<EntityTable
			{columns}
			entities={items}
			{loading}
			searchPlaceholder="Search lots..."
			onRowClick={handleRowClick}
		/>
	{/if}
</div>
