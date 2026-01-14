<script lang="ts">
	import type { RiskSummary } from '$lib/api/tauri';

	interface Props {
		risks: RiskSummary[];
		maxBars?: number;
	}

	let { risks, maxBars = 10 }: Props = $props();

	// Sort by RPN descending and take top N
	const sortedRisks = $derived(
		[...risks]
			.filter((r) => r.rpn !== undefined && r.rpn > 0)
			.sort((a, b) => (b.rpn ?? 0) - (a.rpn ?? 0))
			.slice(0, maxBars)
	);

	const maxRpn = $derived(Math.max(...sortedRisks.map((r) => r.rpn ?? 0), 1));

	function getRpnColor(rpn: number): string {
		if (rpn >= 200) return 'bg-red-500';
		if (rpn >= 100) return 'bg-orange-500';
		if (rpn >= 50) return 'bg-yellow-500';
		return 'bg-green-500';
	}

	function getRiskLevelLabel(level: string | undefined): string {
		if (!level) return 'Unknown';
		return level.charAt(0).toUpperCase() + level.slice(1);
	}
</script>

{#if sortedRisks.length === 0}
	<div class="flex h-48 items-center justify-center text-muted-foreground">
		No risks with RPN data available
	</div>
{:else}
	<div class="space-y-3">
		{#each sortedRisks as risk}
			<div class="group">
				<div class="mb-1 flex items-center justify-between text-sm">
					<span class="max-w-xs truncate font-medium" title={risk.title}>
						{risk.title}
					</span>
					<span class="font-mono font-bold {risk.rpn && risk.rpn >= 100 ? 'text-red-500' : 'text-muted-foreground'}">
						{risk.rpn ?? 0}
					</span>
				</div>
				<div class="h-6 w-full overflow-hidden rounded bg-muted">
					<div
						class="h-full transition-all duration-500 {getRpnColor(risk.rpn ?? 0)} group-hover:opacity-80"
						style="width: {((risk.rpn ?? 0) / maxRpn) * 100}%"
					>
						<div class="flex h-full items-center px-2">
							<span class="text-xs font-medium text-white">
								S:{risk.severity ?? '-'} × O:{risk.occurrence ?? '-'} × D:{risk.detection ?? '-'}
							</span>
						</div>
					</div>
				</div>
			</div>
		{/each}
	</div>

	<!-- RPN Formula explanation -->
	<div class="mt-4 rounded-lg bg-muted/50 p-3 text-xs text-muted-foreground">
		<strong>RPN = Severity × Occurrence × Detection</strong>
		<br />
		Scale: 1-10 for each factor. Maximum RPN = 1000.
	</div>
{/if}
