<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { EntityTable, FilterPanel } from '$lib/components/entities';
	import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '$lib/components/ui';
	import { capas, type CapaSummary, type CapaStats, type ListCapasParams } from '$lib/api/tauri';
	import { isProjectOpen } from '$lib/stores/project';
	import type { FilterFieldDefinition, FilterState, QuickFilter } from '$lib/api/types';

	let items = $state<CapaSummary[]>([]);
	let stats = $state<CapaStats | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let currentFilters = $state<FilterState>({});

	// Define filter fields for CAPAs
	const filterFields: FilterFieldDefinition[] = [
		{
			key: 'capa_status',
			label: 'CAPA Status',
			type: 'select',
			options: [
				{ value: 'initiation', label: 'Initiation' },
				{ value: 'investigation', label: 'Investigation' },
				{ value: 'implementation', label: 'Implementation' },
				{ value: 'verification', label: 'Verification' },
				{ value: 'closed', label: 'Closed' }
			],
			placeholder: 'All statuses'
		},
		{
			key: 'capa_type',
			label: 'Type',
			type: 'select',
			options: [
				{ value: 'corrective', label: 'Corrective' },
				{ value: 'preventive', label: 'Preventive' }
			],
			placeholder: 'All types'
		},
		{
			key: 'search',
			label: 'Search',
			type: 'text',
			placeholder: 'Search CAPAs...'
		}
	];

	// Quick filters for common scenarios
	const quickFilters: QuickFilter[] = [
		{
			id: 'open',
			label: 'Open CAPAs',
			filters: { open_only: true }
		},
		{
			id: 'overdue',
			label: 'Overdue',
			filters: { overdue_only: true }
		},
		{
			id: 'verification',
			label: 'Pending Verification',
			filters: { capa_status: 'verification' }
		},
		{
			id: 'recent',
			label: 'Recent (30 days)',
			filters: { recent_days: 30 }
		}
	];

	// Format CAPA status with indicator
	function formatCapaStatus(value: unknown): string {
		const status = (value as string)?.toLowerCase();
		if (status === 'initiation') return '📝 Initiation';
		if (status === 'investigation') return '🔍 Investigation';
		if (status === 'implementation') return '🔧 Implementation';
		if (status === 'verification') return '✓ Verification';
		if (status === 'closed') return '✅ Closed';
		return status || '-';
	}

	// Format CAPA type
	function formatCapaType(value: unknown): string {
		const type = (value as string)?.toLowerCase();
		if (type === 'corrective') return 'Corrective';
		if (type === 'preventive') return 'Preventive';
		return type || '-';
	}

	// Format due date with warning
	function formatDueDate(value: unknown): string {
		if (!value) return '-';
		const dateStr = value as string;
		const date = new Date(dateStr);
		const now = new Date();
		const daysUntil = Math.ceil((date.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
		if (daysUntil < 0) return `⚠️ ${dateStr}`;
		if (daysUntil <= 7) return `⏰ ${dateStr}`;
		return dateStr;
	}

	// Format effectiveness verification
	function formatVerified(value: unknown): string {
		if (value === true) return '✅ Yes';
		if (value === false) return '❌ No';
		return '-';
	}

	// Table columns with CAPA-specific data and custom renderers
	const columns = [
		{ key: 'id', label: 'ID', sortable: true, class: 'font-mono text-xs w-40' },
		{ key: 'title', label: 'CAPA', sortable: true },
		{ key: 'capa_type', label: 'Type', sortable: true, class: 'w-24', render: formatCapaType },
		{ key: 'capa_status', label: 'Status', sortable: true, class: 'w-32', render: formatCapaStatus },
		{ key: 'due_date', label: 'Due Date', sortable: true, class: 'w-28', render: formatDueDate },
		{ key: 'effectiveness_verified', label: 'Verified', sortable: true, class: 'w-20', render: formatVerified }
	];

	// Convert FilterState to ListCapasParams
	function buildParams(filters: FilterState): ListCapasParams {
		const params: ListCapasParams = {};

		if (filters.capa_status) params.capa_status = filters.capa_status as string;
		if (filters.capa_type) params.capa_type = filters.capa_type as string;
		if (filters.search) params.search = filters.search as string;
		if (filters.open_only) params.open_only = filters.open_only as boolean;
		if (filters.overdue_only) params.overdue_only = filters.overdue_only as boolean;
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
				capas.list(params),
				capas.getStats()
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
		goto(`/quality/capas/${entity.id}`);
	}

	onMount(() => { loadData(); });
	$effect(() => { if ($isProjectOpen) loadData(); });
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">CAPAs</h1>
			<p class="text-muted-foreground">Corrective and Preventive Actions</p>
		</div>
		<Button onclick={() => goto('/quality/capas/new')}>New CAPA</Button>
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
				<CardTitle class="text-sm font-medium text-muted-foreground">Open</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-blue-500">{stats?.open ?? 0}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Overdue</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-500">{stats?.overdue ?? 0}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Pending Verification</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-yellow-500">{stats?.by_capa_status?.verification ?? 0}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Verified Effective</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-500">{stats?.verified_effective ?? 0}</div>
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
				<p class="text-muted-foreground">Open a project to view CAPAs</p>
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
			searchPlaceholder="Search CAPAs..."
			onRowClick={handleRowClick}
		/>
	{/if}
</div>
