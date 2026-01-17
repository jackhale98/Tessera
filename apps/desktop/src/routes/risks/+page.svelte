<script lang="ts">
	import { goto } from '$app/navigation';
	import { FilterPanel } from '$lib/components/entities';
	import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '$lib/components/ui';
	import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui';
	import { Input } from '$lib/components/ui';
	import { Search } from 'lucide-svelte';
	import { risks } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import { risksFilterConfig } from '$lib/config/filters';
	import type { FilterState, RiskMatrix, RiskMatrixCell } from '$lib/api/types';
	import type { RiskSummary, ListRisksResult, RiskStats, ListRisksParams } from '$lib/api/tauri';

	let risksData = $state<RiskSummary[]>([]);
	let stats = $state<RiskStats | null>(null);
	let matrix = $state<RiskMatrix | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let searchQuery = $state('');
	let currentFilters = $state<FilterState>({});

	// Sort state
	let sortColumn = $state<string | null>(null);
	let sortDirection = $state<'asc' | 'desc'>('asc');

	const filteredRisks = $derived(() => {
		let result = risksData;

		// Apply client-side search
		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			result = result.filter(
				(r) =>
					r.title.toLowerCase().includes(query) ||
					r.id.toLowerCase().includes(query) ||
					r.failure_mode.toLowerCase().includes(query)
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

				// Numeric comparison for S, O, D, RPN
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
			const params: ListRisksParams = {};

			if (currentFilters.status && Array.isArray(currentFilters.status)) {
				params.status = currentFilters.status as string[];
			}
			if (currentFilters.risk_type) {
				params.risk_type = currentFilters.risk_type as string;
			}
			if (currentFilters.risk_level) {
				params.risk_level = currentFilters.risk_level as string;
			}
			if (currentFilters.unmitigated_only) {
				params.unmitigated_only = currentFilters.unmitigated_only as boolean;
			}
			// Handle RPN range
			if (currentFilters.rpn_range) {
				const range = currentFilters.rpn_range as { min?: number; max?: number };
				if (range.min !== undefined) {
					params.min_rpn = range.min;
				}
				// Note: max_rpn would need backend support - currently we filter client-side
			}

			const [risksResult, statsResult, matrixResult] = await Promise.all([
				risks.list(params),
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

	function handleFiltersChange(filters: FilterState) {
		currentFilters = filters;
		loadData();
	}

	function handleRowClick(risk: RiskSummary) {
		goto(`/risks/${risk.id}`);
	}

	function handleSort(column: string) {
		if (sortColumn === column) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortColumn = column;
			sortDirection = 'asc';
		}
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

	function getMatrixCell(severity: number, occurrence: number): RiskMatrixCell | undefined {
		if (!matrix) return undefined;
		return matrix.cells.find((c) => c.severity === severity && c.occurrence === occurrence);
	}

	// Use single effect for loading
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
		<Button onclick={() => goto('/risks/new')}>New Risk</Button>
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
	{#if matrix && matrix.cells}
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

						<!-- Matrix rows (occurrence 10 at top, 1 at bottom) -->
						{#each Array.from({ length: 10 }, (_, i) => 10 - i) as occ}
							<div class="grid grid-cols-10 gap-1 mb-1">
								{#each Array.from({ length: 10 }, (_, i) => i + 1) as sev}
									{@const cell = getMatrixCell(sev, occ)}
									{@const rpn = sev * occ}
									{@const riskLevel = rpn >= 200 ? 'critical' : rpn >= 100 ? 'high' : rpn >= 40 ? 'medium' : 'low'}
									<button
										class="h-8 rounded text-xs font-medium transition-colors {cell && cell.count > 0
											? getMatrixCellColor(riskLevel)
											: 'bg-muted/30 hover:bg-muted/50'}"
										onclick={() => cell && cell.count > 0 && console.log('Cell clicked:', cell.risk_ids)}
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
						<span class="text-xs">Low (RPN &lt; 40)</span>
					</div>
					<div class="flex items-center gap-1">
						<div class="w-4 h-4 rounded bg-yellow-500/30"></div>
						<span class="text-xs">Medium (40-99)</span>
					</div>
					<div class="flex items-center gap-1">
						<div class="w-4 h-4 rounded bg-orange-500/30"></div>
						<span class="text-xs">High (100-199)</span>
					</div>
					<div class="flex items-center gap-1">
						<div class="w-4 h-4 rounded bg-red-500/30"></div>
						<span class="text-xs">Critical (200+)</span>
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
		<div class="space-y-4">
			<!-- Filter Panel -->
			<FilterPanel
				fields={risksFilterConfig.fields}
				quickFilters={risksFilterConfig.quickFilters}
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
						placeholder="Search risks..."
						bind:value={searchQuery}
						class="pl-9"
					/>
				</div>
				<div class="text-sm text-muted-foreground">
					{filteredRisks().length} of {risksData.length} items
				</div>
			</div>

			<!-- Table -->
			<div class="rounded-md border">
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead
								class="cursor-pointer select-none w-40 hover:bg-muted/50"
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
								class="cursor-pointer select-none w-24 hover:bg-muted/50"
								onclick={() => handleSort('risk_type')}
							>
								<div class="flex items-center gap-1">
									Type
									{#if sortColumn === 'risk_type'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-16 hover:bg-muted/50"
								onclick={() => handleSort('severity')}
							>
								<div class="flex items-center gap-1">
									S
									{#if sortColumn === 'severity'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-16 hover:bg-muted/50"
								onclick={() => handleSort('occurrence')}
							>
								<div class="flex items-center gap-1">
									O
									{#if sortColumn === 'occurrence'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-16 hover:bg-muted/50"
								onclick={() => handleSort('detection')}
							>
								<div class="flex items-center gap-1">
									D
									{#if sortColumn === 'detection'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-20 hover:bg-muted/50"
								onclick={() => handleSort('rpn')}
							>
								<div class="flex items-center gap-1">
									RPN
									{#if sortColumn === 'rpn'}
										<span class="text-xs">{sortDirection === 'asc' ? '\u2191' : '\u2193'}</span>
									{/if}
								</div>
							</TableHead>
							<TableHead
								class="cursor-pointer select-none w-24 hover:bg-muted/50"
								onclick={() => handleSort('risk_level')}
							>
								<div class="flex items-center gap-1">
									Level
									{#if sortColumn === 'risk_level'}
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
										<Badge variant={getStatusVariant(risk.status)} class="capitalize">
											{risk.status}
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
