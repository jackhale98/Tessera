<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { EntityTable, FilterPanel } from '$lib/components/entities';
	import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '$lib/components/ui';
	import { ncrs, type NcrSummary, type NcrStats, type ListNcrsParams } from '$lib/api/tauri';
	import { isProjectOpen } from '$lib/stores/project';
	import type { FilterFieldDefinition, FilterState, QuickFilter } from '$lib/api/types';

	let items = $state<NcrSummary[]>([]);
	let stats = $state<NcrStats | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let currentFilters = $state<FilterState>({});

	// Define filter fields for NCRs
	const filterFields: FilterFieldDefinition[] = [
		{
			key: 'ncr_status',
			label: 'NCR Status',
			type: 'select',
			options: [
				{ value: 'open', label: 'Open' },
				{ value: 'containment', label: 'Containment' },
				{ value: 'investigation', label: 'Investigation' },
				{ value: 'disposition', label: 'Disposition' },
				{ value: 'closed', label: 'Closed' }
			],
			placeholder: 'All statuses'
		},
		{
			key: 'ncr_type',
			label: 'Type',
			type: 'select',
			options: [
				{ value: 'internal', label: 'Internal' },
				{ value: 'supplier', label: 'Supplier' },
				{ value: 'customer', label: 'Customer' }
			],
			placeholder: 'All types'
		},
		{
			key: 'severity',
			label: 'Severity',
			type: 'select',
			options: [
				{ value: 'minor', label: 'Minor' },
				{ value: 'major', label: 'Major' },
				{ value: 'critical', label: 'Critical' }
			],
			placeholder: 'All severities'
		},
		{
			key: 'category',
			label: 'Category',
			type: 'select',
			options: [
				{ value: 'material', label: 'Material' },
				{ value: 'process', label: 'Process' },
				{ value: 'design', label: 'Design' },
				{ value: 'documentation', label: 'Documentation' },
				{ value: 'inspection', label: 'Inspection' }
			],
			placeholder: 'All categories'
		},
		{
			key: 'search',
			label: 'Search',
			type: 'text',
			placeholder: 'Search NCRs...'
		}
	];

	// Quick filters for common scenarios
	const quickFilters: QuickFilter[] = [
		{
			id: 'open',
			label: 'Open NCRs',
			filters: { open_only: true }
		},
		{
			id: 'critical',
			label: 'Critical Severity',
			filters: { severity: 'critical' }
		},
		{
			id: 'investigation',
			label: 'Under Investigation',
			filters: { ncr_status: 'investigation' }
		},
		{
			id: 'recent',
			label: 'Recent (30 days)',
			filters: { recent_days: 30 }
		}
	];

	// Format severity with indicator
	function formatSeverity(value: unknown): string {
		const severity = (value as string)?.toLowerCase();
		if (severity === 'critical') return '🔴 Critical';
		if (severity === 'major') return '🟡 Major';
		return '🟢 Minor';
	}

	// Format NCR status with indicator
	function formatNcrStatus(value: unknown): string {
		const status = (value as string)?.toLowerCase();
		if (status === 'open') return '📋 Open';
		if (status === 'containment') return '🛡️ Containment';
		if (status === 'investigation') return '🔍 Investigation';
		if (status === 'disposition') return '⚖️ Disposition';
		if (status === 'closed') return '✅ Closed';
		return status || '-';
	}

	// Format NCR type
	function formatNcrType(value: unknown): string {
		const type = (value as string)?.toLowerCase();
		if (type === 'internal') return 'Internal';
		if (type === 'supplier') return 'Supplier';
		if (type === 'customer') return 'Customer';
		return type || '-';
	}

	// Table columns with NCR-specific data and custom renderers
	const columns = [
		{ key: 'id', label: 'ID', sortable: true, class: 'font-mono text-xs w-40' },
		{ key: 'title', label: 'Non-Conformance', sortable: true },
		{ key: 'ncr_type', label: 'Type', sortable: true, class: 'w-24', render: formatNcrType },
		{ key: 'category', label: 'Category', sortable: true, class: 'w-28' },
		{ key: 'severity', label: 'Severity', sortable: true, class: 'w-24', render: formatSeverity },
		{ key: 'ncr_status', label: 'Status', sortable: true, class: 'w-32', render: formatNcrStatus }
	];

	// Convert FilterState to ListNcrsParams
	function buildParams(filters: FilterState): ListNcrsParams {
		const params: ListNcrsParams = {};

		if (filters.ncr_status) params.ncr_status = filters.ncr_status as string;
		if (filters.ncr_type) params.ncr_type = filters.ncr_type as string;
		if (filters.severity) params.severity = filters.severity as string;
		if (filters.category) params.category = filters.category as string;
		if (filters.search) params.search = filters.search as string;
		if (filters.open_only) params.open_only = filters.open_only as boolean;
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
				ncrs.list(params),
				ncrs.getStats()
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
		goto(`/quality/ncrs/${entity.id}`);
	}

	onMount(() => { loadData(); });
	$effect(() => { if ($isProjectOpen) loadData(); });
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Non-Conformance Reports</h1>
			<p class="text-muted-foreground">Quality non-conformances and dispositions</p>
		</div>
		<Button onclick={() => goto('/quality/ncrs/new')}>New NCR</Button>
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
				<div class="text-2xl font-bold text-red-500">{stats?.open ?? 0}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Under Investigation</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-yellow-500">{stats?.by_ncr_status?.investigation ?? 0}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Critical</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-600">{stats?.by_severity?.critical ?? 0}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Total Cost</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">${stats?.total_cost?.toLocaleString() ?? '0'}</div>
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
				<p class="text-muted-foreground">Open a project to view NCRs</p>
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
			searchPlaceholder="Search NCRs..."
			onRowClick={handleRowClick}
		/>
	{/if}
</div>
