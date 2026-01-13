<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '$lib/components/ui';
	import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui';
	import { Input } from '$lib/components/ui';
	import { risks } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import type { RiskMatrix, RiskMatrixCell } from '$lib/api/types';
	import type { RiskSummary, ListRisksResult, RiskStats } from '$lib/api/tauri';

	let risksData = $state<RiskSummary[]>([]);
	let stats = $state<RiskStats | null>(null);
	let matrix = $state<RiskMatrix | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let searchQuery = $state('');

	const filteredRisks = $derived(() => {
		if (!searchQuery) return risksData;
		const query = searchQuery.toLowerCase();
		return risksData.filter(
			(r) =>
				r.title.toLowerCase().includes(query) ||
				r.id.toLowerCase().includes(query) ||
				r.failure_mode.toLowerCase().includes(query)
		);
	});

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
			console.error('Failed to load risks:', e);
		} finally {
			loading = false;
		}
	}

	function handleRowClick(risk: RiskSummary) {
		goto(`/risks/${risk.id}`);
	}

	function getRiskLevelColor(level: string | undefined): string {
		switch (level?.toLowerCase()) {
			case 'low':
				return 'bg-green-500/20 text-green-400';
			case 'medium':
				return 'bg-yellow-500/20 text-yellow-400';
			case 'high':
				return 'bg-orange-500/20 text-orange-400';
			case 'critical':
				return 'bg-red-500/20 text-red-400';
			default:
				return 'bg-muted text-muted-foreground';
		}
	}

	function getMatrixCellColor(level: string): string {
		switch (level.toLowerCase()) {
			case 'low':
				return 'bg-green-500/30 hover:bg-green-500/40';
			case 'medium':
				return 'bg-yellow-500/30 hover:bg-yellow-500/40';
			case 'high':
				return 'bg-orange-500/30 hover:bg-orange-500/40';
			case 'critical':
				return 'bg-red-500/30 hover:bg-red-500/40';
			default:
				return 'bg-muted hover:bg-muted/80';
		}
	}

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
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Risk Management</h1>
			<p class="text-muted-foreground">FMEA and risk analysis</p>
		</div>
		<Button>New Risk</Button>
	</div>

	<!-- Stats cards -->
	{#if stats}
		<div class="grid gap-4 md:grid-cols-4">
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Total Risks</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.total}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Avg RPN</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.average_rpn?.toFixed(0) ?? '-'}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">High Priority</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-orange-400">{stats.high_priority_count}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Unmitigated</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-destructive">{stats.unmitigated_count}</div>
				</CardContent>
			</Card>
		</div>
	{/if}

	<!-- Risk Matrix -->
	{#if matrix && matrix.cells.length > 0}
		<Card>
			<CardHeader>
				<CardTitle>Risk Matrix</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="grid grid-cols-11 gap-1">
					<!-- Y-axis label -->
					<div class="col-span-1 flex items-center justify-center">
						<span class="text-xs text-muted-foreground -rotate-90 whitespace-nowrap">Occurrence</span>
					</div>

					<!-- Matrix grid -->
					<div class="col-span-10">
						<!-- Column headers (Severity) -->
						<div class="grid grid-cols-10 gap-1 mb-1">
							{#each Array.from({ length: 10 }, (_, i) => i + 1) as sev}
								<div class="text-center text-xs text-muted-foreground">{sev}</div>
							{/each}
						</div>

						<!-- Matrix rows -->
						{#each Array.from({ length: 10 }, (_, i) => 10 - i) as occ}
							<div class="grid grid-cols-10 gap-1 mb-1">
								{#each Array.from({ length: 10 }, (_, i) => i + 1) as sev}
									{@const cell = matrix.cells.find((c: RiskMatrixCell) => c.severity === sev && c.occurrence === occ)}
									<button
										class="h-8 rounded text-xs font-medium transition-colors {cell
											? getMatrixCellColor(cell.risk_level)
											: 'bg-muted/30'}"
										onclick={() => cell && console.log('Cell clicked:', cell.risk_ids)}
									>
										{cell?.count || ''}
									</button>
								{/each}
							</div>
						{/each}

						<!-- X-axis label -->
						<div class="text-center text-xs text-muted-foreground mt-2">Severity</div>
					</div>
				</div>

				<!-- Legend -->
				<div class="flex items-center justify-center gap-4 mt-4">
					<div class="flex items-center gap-1">
						<div class="w-4 h-4 rounded bg-green-500/30"></div>
						<span class="text-xs">Low</span>
					</div>
					<div class="flex items-center gap-1">
						<div class="w-4 h-4 rounded bg-yellow-500/30"></div>
						<span class="text-xs">Medium</span>
					</div>
					<div class="flex items-center gap-1">
						<div class="w-4 h-4 rounded bg-orange-500/30"></div>
						<span class="text-xs">High</span>
					</div>
					<div class="flex items-center gap-1">
						<div class="w-4 h-4 rounded bg-red-500/30"></div>
						<span class="text-xs">Critical</span>
					</div>
				</div>
			</CardContent>
		</Card>
	{/if}

	<!-- Error display -->
	{#if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{/if}

	<!-- Risks table -->
	{#if !$isProjectOpen}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Open a project to view risks</p>
			</CardContent>
		</Card>
	{:else}
		<Card>
			<CardHeader>
				<div class="flex items-center justify-between">
					<CardTitle>Risk Register</CardTitle>
					<Input
						type="search"
						placeholder="Search risks..."
						bind:value={searchQuery}
						class="max-w-sm"
					/>
				</div>
			</CardHeader>
			<CardContent>
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead class="w-40">ID</TableHead>
							<TableHead>Title</TableHead>
							<TableHead class="w-24">Type</TableHead>
							<TableHead class="w-16">S</TableHead>
							<TableHead class="w-16">O</TableHead>
							<TableHead class="w-16">D</TableHead>
							<TableHead class="w-20">RPN</TableHead>
							<TableHead class="w-24">Level</TableHead>
							<TableHead class="w-24">Status</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#if loading}
							<TableRow>
								<TableCell colspan={9} class="h-24 text-center">
									<div class="flex items-center justify-center gap-2">
										<div class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
										Loading...
									</div>
								</TableCell>
							</TableRow>
						{:else if filteredRisks().length === 0}
							<TableRow>
								<TableCell colspan={9} class="h-24 text-center text-muted-foreground">
									No risks found
								</TableCell>
							</TableRow>
						{:else}
							{#each filteredRisks() as risk (risk.id)}
								<TableRow class="cursor-pointer" onclick={() => handleRowClick(risk)}>
									<TableCell class="font-mono text-xs">{risk.id}</TableCell>
									<TableCell>{risk.title}</TableCell>
									<TableCell class="capitalize">{risk.risk_type}</TableCell>
									<TableCell>{risk.severity ?? '-'}</TableCell>
									<TableCell>{risk.occurrence ?? '-'}</TableCell>
									<TableCell>{risk.detection ?? '-'}</TableCell>
									<TableCell class="font-bold">{risk.rpn ?? '-'}</TableCell>
									<TableCell>
										<Badge class={getRiskLevelColor(risk.risk_level)}>
											{risk.risk_level ?? 'N/A'}
										</Badge>
									</TableCell>
									<TableCell>
										<Badge variant="outline" class="capitalize">{risk.status}</Badge>
									</TableCell>
								</TableRow>
							{/each}
						{/if}
					</TableBody>
				</Table>
			</CardContent>
		</Card>
	{/if}
</div>
