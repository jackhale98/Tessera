<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { EntityTable, FilterPanel } from '$lib/components/entities';
	import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '$lib/components/ui';
	import { deviations, type DeviationSummary, type DeviationStats, type ListDeviationsParams } from '$lib/api/tauri';
	import { isProjectOpen } from '$lib/stores/project';
	import type { FilterFieldDefinition, FilterState, QuickFilter } from '$lib/api/types';

	let items = $state<DeviationSummary[]>([]);
	let stats = $state<DeviationStats | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let currentFilters = $state<FilterState>({});

	// Define filter fields for deviations
	const filterFields: FilterFieldDefinition[] = [
		{
			key: 'dev_status',
			label: 'Deviation Status',
			type: 'select',
			options: [
				{ value: 'pending', label: 'Pending' },
				{ value: 'approved', label: 'Approved' },
				{ value: 'active', label: 'Active' },
				{ value: 'expired', label: 'Expired' },
				{ value: 'closed', label: 'Closed' },
				{ value: 'rejected', label: 'Rejected' }
			],
			placeholder: 'All statuses'
		},
		{
			key: 'deviation_type',
			label: 'Type',
			type: 'select',
			options: [
				{ value: 'temporary', label: 'Temporary' },
				{ value: 'permanent', label: 'Permanent' },
				{ value: 'emergency', label: 'Emergency' }
			],
			placeholder: 'All types'
		},
		{
			key: 'category',
			label: 'Category',
			type: 'select',
			options: [
				{ value: 'material', label: 'Material' },
				{ value: 'process', label: 'Process' },
				{ value: 'equipment', label: 'Equipment' },
				{ value: 'tooling', label: 'Tooling' },
				{ value: 'specification', label: 'Specification' },
				{ value: 'documentation', label: 'Documentation' }
			],
			placeholder: 'All categories'
		},
		{
			key: 'risk_level',
			label: 'Risk Level',
			type: 'select',
			options: [
				{ value: 'low', label: 'Low' },
				{ value: 'medium', label: 'Medium' },
				{ value: 'high', label: 'High' }
			],
			placeholder: 'All risk levels'
		},
		{
			key: 'search',
			label: 'Search',
			type: 'text',
			placeholder: 'Search deviations...'
		}
	];

	// Quick filters for common scenarios
	const quickFilters: QuickFilter[] = [
		{
			id: 'active',
			label: 'Active Deviations',
			filters: { active_only: true }
		},
		{
			id: 'pending',
			label: 'Pending Approval',
			filters: { dev_status: 'pending' }
		},
		{
			id: 'high_risk',
			label: 'High Risk',
			filters: { risk_level: 'high' }
		},
		{
			id: 'expiring_soon',
			label: 'Recent (30 days)',
			filters: { recent_days: 30 }
		}
	];

	// Format risk level with emoji indicator
	function formatRiskLevel(value: unknown): string {
		const level = (value as string)?.toLowerCase();
		if (level === 'high') return '🔴 High';
		if (level === 'medium') return '🟡 Medium';
		return '🟢 Low';
	}

	// Format deviation status with emoji indicator
	function formatDevStatus(value: unknown): string {
		const status = (value as string)?.toLowerCase();
		if (status === 'active') return '✅ Active';
		if (status === 'pending') return '⏳ Pending';
		if (status === 'approved') return '✓ Approved';
		if (status === 'expired') return '⚠️ Expired';
		if (status === 'closed') return '📁 Closed';
		if (status === 'rejected') return '❌ Rejected';
		return status || '-';
	}

	// Format expiration date with warning for soon/past
	function formatExpirationDate(value: unknown): string {
		if (!value) return '-';
		const dateStr = value as string;
		const date = new Date(dateStr);
		const now = new Date();
		const daysUntil = Math.ceil((date.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
		if (daysUntil < 0) return `⚠️ ${dateStr}`;
		if (daysUntil <= 7) return `⏰ ${dateStr}`;
		return dateStr;
	}

	// Table columns with deviation-specific data and custom renderers
	const columns = [
		{ key: 'id', label: 'ID', sortable: true, class: 'font-mono text-xs w-40' },
		{ key: 'title', label: 'Deviation', sortable: true },
		{ key: 'deviation_type', label: 'Type', sortable: true, class: 'w-24' },
		{ key: 'category', label: 'Category', sortable: true, class: 'w-28' },
		{ key: 'risk_level', label: 'Risk', sortable: true, class: 'w-20', render: formatRiskLevel },
		{ key: 'dev_status', label: 'Status', sortable: true, class: 'w-24', render: formatDevStatus },
		{ key: 'expiration_date', label: 'Expires', sortable: true, class: 'w-28', render: formatExpirationDate }
	];

	// Convert FilterState to ListDeviationsParams
	function buildParams(filters: FilterState): ListDeviationsParams {
		const params: ListDeviationsParams = {};

		if (filters.dev_status) params.dev_status = filters.dev_status as string;
		if (filters.deviation_type) params.deviation_type = filters.deviation_type as string;
		if (filters.category) params.category = filters.category as string;
		if (filters.risk_level) params.risk_level = filters.risk_level as string;
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
				deviations.list(params),
				deviations.getStats()
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
		goto(`/manufacturing/deviations/${entity.id}`);
	}

	onMount(() => { loadData(); });
	$effect(() => { if ($isProjectOpen) loadData(); });
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Deviations</h1>
			<p class="text-muted-foreground">Process and product deviations tracking</p>
		</div>
		<Button onclick={() => goto('/manufacturing/deviations/new')}>New Deviation</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid gap-4 md:grid-cols-4">
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
				<CardTitle class="text-sm font-medium text-muted-foreground">Active</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-500">{stats?.active ?? 0}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">Pending Approval</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-yellow-500">{stats?.by_dev_status?.pending ?? 0}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">High Risk</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-500">{stats?.by_risk?.high ?? 0}</div>
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
			searchPlaceholder="Search deviations..."
			onRowClick={handleRowClick}
		/>
	{/if}
</div>
