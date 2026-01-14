<script lang="ts">
	import { goto } from '$app/navigation';
	import type { RiskMatrix, RiskMatrixCell, RiskLevel } from '$lib/api/types';

	interface Props {
		matrix: RiskMatrix;
		onCellClick?: (cell: RiskMatrixCell) => void;
	}

	let { matrix, onCellClick }: Props = $props();

	// Create a lookup for cells
	const cellLookup = $derived(() => {
		const lookup = new Map<string, RiskMatrixCell>();
		for (const cell of matrix.cells) {
			lookup.set(`${cell.severity}:${cell.occurrence}`, cell);
		}
		return lookup;
	});

	function getCell(severity: number, occurrence: number): RiskMatrixCell | undefined {
		return cellLookup().get(`${severity}:${occurrence}`);
	}

	function getRiskLevelColor(level: RiskLevel): string {
		const colors: Record<RiskLevel, string> = {
			low: 'bg-green-500 hover:bg-green-600',
			medium: 'bg-yellow-500 hover:bg-yellow-600',
			high: 'bg-orange-500 hover:bg-orange-600',
			critical: 'bg-red-500 hover:bg-red-600'
		};
		return colors[level] ?? 'bg-gray-500';
	}

	function getBackgroundLevel(severity: number, occurrence: number): RiskLevel {
		const rpn = severity * occurrence;
		if (rpn >= 64) return 'critical';
		if (rpn >= 36) return 'high';
		if (rpn >= 16) return 'medium';
		return 'low';
	}

	function getBackgroundColor(severity: number, occurrence: number): string {
		const level = getBackgroundLevel(severity, occurrence);
		const colors: Record<RiskLevel, string> = {
			low: 'bg-green-100 dark:bg-green-900/30',
			medium: 'bg-yellow-100 dark:bg-yellow-900/30',
			high: 'bg-orange-100 dark:bg-orange-900/30',
			critical: 'bg-red-100 dark:bg-red-900/30'
		};
		return colors[level];
	}

	function handleCellClick(cell: RiskMatrixCell) {
		if (onCellClick) {
			onCellClick(cell);
		} else if (cell.risk_ids.length === 1) {
			goto(`/risks/${cell.risk_ids[0]}`);
		}
	}

	// Generate severity and occurrence values (1-10)
	const severityValues = $derived(Array.from({ length: matrix.max_severity }, (_, i) => matrix.max_severity - i));
	const occurrenceValues = $derived(Array.from({ length: matrix.max_occurrence }, (_, i) => i + 1));
</script>

<div class="overflow-auto">
	<div class="min-w-fit">
		<!-- Y-axis label -->
		<div class="mb-2 text-center text-sm font-medium text-muted-foreground">
			Severity →
		</div>

		<div class="flex">
			<!-- Y-axis (Severity) -->
			<div class="flex flex-col items-end pr-2">
				<div class="h-8"></div> <!-- Spacer for header -->
				{#each severityValues as sev}
					<div class="flex h-12 w-8 items-center justify-center text-sm font-medium">
						{sev}
					</div>
				{/each}
			</div>

			<!-- Grid -->
			<div>
				<!-- X-axis header (Occurrence) -->
				<div class="flex">
					{#each occurrenceValues as occ}
						<div class="flex h-8 w-12 items-center justify-center text-sm font-medium">
							{occ}
						</div>
					{/each}
				</div>

				<!-- Matrix cells -->
				{#each severityValues as sev}
					<div class="flex">
						{#each occurrenceValues as occ}
							{@const cell = getCell(sev, occ)}
							<button
								class="flex h-12 w-12 items-center justify-center border border-border/50 text-sm font-bold transition-all {getBackgroundColor(sev, occ)} {cell && cell.count > 0 ? 'cursor-pointer' : 'cursor-default'}"
								onclick={() => cell && cell.count > 0 && handleCellClick(cell)}
								disabled={!cell || cell.count === 0}
								title={cell && cell.count > 0 ? `${cell.count} risk(s) - Click to view` : `S=${sev}, O=${occ}`}
							>
								{#if cell && cell.count > 0}
									<span class="flex h-8 w-8 items-center justify-center rounded-full {getRiskLevelColor(cell.risk_level)} text-white">
										{cell.count}
									</span>
								{/if}
							</button>
						{/each}
					</div>
				{/each}
			</div>
		</div>

		<!-- X-axis label -->
		<div class="mt-2 text-center text-sm font-medium text-muted-foreground">
			← Occurrence
		</div>
	</div>
</div>

<!-- Legend -->
<div class="mt-6 flex flex-wrap items-center justify-center gap-4 text-sm">
	<div class="flex items-center gap-2">
		<div class="h-4 w-4 rounded bg-green-500"></div>
		<span>Low Risk</span>
	</div>
	<div class="flex items-center gap-2">
		<div class="h-4 w-4 rounded bg-yellow-500"></div>
		<span>Medium Risk</span>
	</div>
	<div class="flex items-center gap-2">
		<div class="h-4 w-4 rounded bg-orange-500"></div>
		<span>High Risk</span>
	</div>
	<div class="flex items-center gap-2">
		<div class="h-4 w-4 rounded bg-red-500"></div>
		<span>Critical Risk</span>
	</div>
</div>
