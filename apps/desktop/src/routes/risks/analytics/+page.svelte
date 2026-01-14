<script lang="ts">
	import { onMount } from 'svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui';
	import { RiskMatrixHeatmap, RpnChart, RiskStatsCards } from '$lib/components/risk';
	import { risks } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import type { RiskMatrix } from '$lib/api/types';
	import type { RiskSummary, RiskStats } from '$lib/api/tauri';

	let risksData = $state<RiskSummary[]>([]);
	let stats = $state<RiskStats | null>(null);
	let matrix = $state<RiskMatrix | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function loadData() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			const [risksResult, statsResult, matrixResult] = await Promise.all([
				risks.list(),
				risks.getStats(),
				risks.getMatrix()
			]);

			risksData = risksResult.items;
			stats = statsResult;
			matrix = matrixResult;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load risk analytics:', e);
		} finally {
			loading = false;
		}
	}

	// Calculate additional metrics
	const risksByType = $derived(() => {
		const byType: Record<string, number> = {};
		for (const risk of risksData) {
			const type = risk.risk_type || 'unknown';
			byType[type] = (byType[type] || 0) + 1;
		}
		return byType;
	});

	const risksByStatus = $derived(() => {
		const byStatus: Record<string, number> = {};
		for (const risk of risksData) {
			byStatus[risk.status] = (byStatus[risk.status] || 0) + 1;
		}
		return byStatus;
	});

	const highRpnRisks = $derived(() => {
		return risksData
			.filter(r => r.rpn !== undefined && r.rpn >= 100)
			.sort((a, b) => (b.rpn ?? 0) - (a.rpn ?? 0));
	});

	onMount(() => {
		loadData();
	});

	$effect(() => {
		if ($isProjectOpen) {
			loadData();
		}
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div>
		<h1 class="text-2xl font-bold">Risk Analytics</h1>
		<p class="text-muted-foreground">Risk metrics and visualizations</p>
	</div>

	<!-- Error display -->
	{#if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{/if}

	{#if !$isProjectOpen}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Open a project to view risk analytics</p>
			</CardContent>
		</Card>
	{:else if loading}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<div class="flex items-center gap-2">
					<div class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
					Loading analytics...
				</div>
			</CardContent>
		</Card>
	{:else}
		<!-- Stats Cards -->
		{#if stats}
			<RiskStatsCards {stats} />
		{/if}

		<!-- Two column layout for matrix and chart -->
		<div class="grid gap-6 lg:grid-cols-2">
			<!-- Risk Matrix Heatmap -->
			{#if matrix}
				<Card>
					<CardHeader>
						<CardTitle>Risk Matrix</CardTitle>
					</CardHeader>
					<CardContent>
						<RiskMatrixHeatmap {matrix} />
					</CardContent>
				</Card>
			{/if}

			<!-- RPN Distribution Chart -->
			<Card>
				<CardHeader>
					<CardTitle>Top Risks by RPN</CardTitle>
				</CardHeader>
				<CardContent>
					<RpnChart risks={risksData} maxBars={10} />
				</CardContent>
			</Card>
		</div>

		<!-- Risk Distribution -->
		<div class="grid gap-6 md:grid-cols-2">
			<!-- By Type -->
			<Card>
				<CardHeader>
					<CardTitle>Risks by Type</CardTitle>
				</CardHeader>
				<CardContent>
					{#if Object.keys(risksByType()).length === 0}
						<p class="text-muted-foreground text-center py-8">No data available</p>
					{:else}
						<div class="space-y-3">
							{#each Object.entries(risksByType()) as [type, count]}
								{@const percentage = (count / risksData.length) * 100}
								<div>
									<div class="flex items-center justify-between text-sm mb-1">
										<span class="capitalize font-medium">{type}</span>
										<span class="text-muted-foreground">{count} ({percentage.toFixed(0)}%)</span>
									</div>
									<div class="h-2 w-full overflow-hidden rounded-full bg-muted">
										<div
											class="h-full bg-primary transition-all"
											style="width: {percentage}%"
										></div>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</CardContent>
			</Card>

			<!-- By Status -->
			<Card>
				<CardHeader>
					<CardTitle>Risks by Status</CardTitle>
				</CardHeader>
				<CardContent>
					{#if Object.keys(risksByStatus()).length === 0}
						<p class="text-muted-foreground text-center py-8">No data available</p>
					{:else}
						<div class="space-y-3">
							{#each Object.entries(risksByStatus()) as [status, count]}
								{@const percentage = (count / risksData.length) * 100}
								{@const statusColors: Record<string, string> = {
									draft: 'bg-gray-500',
									review: 'bg-blue-500',
									approved: 'bg-green-500',
									released: 'bg-purple-500',
									obsolete: 'bg-red-500'
								}}
								<div>
									<div class="flex items-center justify-between text-sm mb-1">
										<span class="capitalize font-medium">{status}</span>
										<span class="text-muted-foreground">{count} ({percentage.toFixed(0)}%)</span>
									</div>
									<div class="h-2 w-full overflow-hidden rounded-full bg-muted">
										<div
											class="h-full transition-all {statusColors[status] ?? 'bg-primary'}"
											style="width: {percentage}%"
										></div>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</CardContent>
			</Card>
		</div>

		<!-- High Priority Risks Table -->
		{#if highRpnRisks().length > 0}
			<Card>
				<CardHeader>
					<CardTitle class="text-red-500">High RPN Risks (≥100)</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="overflow-auto">
						<table class="w-full text-sm">
							<thead class="border-b bg-muted/50">
								<tr>
									<th class="whitespace-nowrap px-3 py-3 text-left font-medium">ID</th>
									<th class="whitespace-nowrap px-3 py-3 text-left font-medium">Title</th>
									<th class="whitespace-nowrap px-3 py-3 text-center font-medium">S</th>
									<th class="whitespace-nowrap px-3 py-3 text-center font-medium">O</th>
									<th class="whitespace-nowrap px-3 py-3 text-center font-medium">D</th>
									<th class="whitespace-nowrap px-3 py-3 text-center font-medium">RPN</th>
									<th class="whitespace-nowrap px-3 py-3 text-center font-medium">Level</th>
								</tr>
							</thead>
							<tbody class="divide-y">
								{#each highRpnRisks() as risk}
									<tr class="hover:bg-muted/50">
										<td class="whitespace-nowrap px-3 py-2 font-mono text-xs">{risk.id}</td>
										<td class="px-3 py-2">{risk.title}</td>
										<td class="whitespace-nowrap px-3 py-2 text-center font-mono">{risk.severity ?? '-'}</td>
										<td class="whitespace-nowrap px-3 py-2 text-center font-mono">{risk.occurrence ?? '-'}</td>
										<td class="whitespace-nowrap px-3 py-2 text-center font-mono">{risk.detection ?? '-'}</td>
										<td class="whitespace-nowrap px-3 py-2 text-center font-mono font-bold text-red-500">{risk.rpn ?? '-'}</td>
										<td class="whitespace-nowrap px-3 py-2 text-center capitalize">{risk.risk_level ?? '-'}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				</CardContent>
			</Card>
		{/if}
	{/if}
</div>
