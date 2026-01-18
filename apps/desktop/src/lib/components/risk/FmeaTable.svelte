<script lang="ts">
	import { goto } from '$app/navigation';
	import { Badge } from '$lib/components/ui';
	import { StatusBadge } from '$lib/components/common';
	import type { FmeaRiskData, FmeaInitialRisk, LinkedEntity, FmeaMitigation, FmeaControl, RiskSummary } from '$lib/api/tauri';
	import { ExternalLink, CheckCircle, AlertTriangle, Shield, FlaskConical, TrendingDown, ArrowRight } from 'lucide-svelte';

	interface Props {
		// Support both old RiskSummary[] and new FmeaRiskData[]
		risks?: RiskSummary[];
		fmeaData?: FmeaRiskData[];
		onRowClick?: (risk: { id: string }) => void;
		/** Show initial risk values alongside residual (default: true) */
		showInitialRisk?: boolean;
	}

	let { risks = [], fmeaData = [], onRowClick, showInitialRisk = true }: Props = $props();

	// Use fmeaData if provided, otherwise fall back to risks
	// Dedupe by ID to prevent "each_key_duplicate" Svelte error
	const displayData = $derived(() => {
		const source = fmeaData.length > 0 ? fmeaData : risks.map(r => ({
			...r,
			hazards: [],
			mitigations: [],
			controls: [],
			initial_risk: undefined
		} as FmeaRiskData));

		const seen = new Set<string>();
		return source.filter((item) => {
			if (seen.has(item.id)) return false;
			seen.add(item.id);
			return true;
		});
	});

	// Check if any risk has initial risk data
	const hasAnyInitialRisk = $derived(() => {
		return displayData().some(item => item.initial_risk?.severity || item.initial_risk?.occurrence || item.initial_risk?.detection);
	});

	function handleRowClick(item: { id: string }) {
		if (onRowClick) {
			onRowClick(item);
		} else {
			goto(`/risks/${item.id}`);
		}
	}

	function handleEntityClick(e: Event, id: string) {
		e.stopPropagation();
		if (!id) return;

		// Determine route based on ID prefix
		const prefix = id.split('-')[0]?.toUpperCase();
		const routeMap: Record<string, string> = {
			HAZ: '/hazards',
			CTRL: '/controls',
			TEST: '/verification/tests',
			RSLT: '/verification/results'
		};
		const route = routeMap[prefix] || '/entities';
		goto(`${route}/${id}`);
	}

	function getRpnClass(rpn: number | undefined): string {
		if (!rpn) return 'text-muted-foreground';
		if (rpn >= 200) return 'text-red-600 font-bold';
		if (rpn >= 100) return 'text-orange-600 font-bold';
		if (rpn >= 50) return 'text-yellow-600 font-semibold';
		return 'text-green-600';
	}

	function getRiskLevelVariant(level: string | undefined): 'default' | 'secondary' | 'destructive' | 'outline' {
		if (!level) return 'outline';
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			low: 'default',
			medium: 'secondary',
			high: 'secondary',
			critical: 'destructive'
		};
		return variants[level] ?? 'outline';
	}

	function getMitigationStatusClass(status: string | undefined): string {
		switch (status?.toLowerCase()) {
			case 'verified': return 'text-green-600';
			case 'completed': return 'text-blue-600';
			case 'in_progress': return 'text-yellow-600';
			default: return 'text-muted-foreground';
		}
	}

	// Check if a score has been reduced from initial to residual
	function hasScoreReduction(initial: number | undefined, residual: number | undefined): boolean {
		return initial !== undefined && residual !== undefined && initial > residual;
	}

	// Calculate RPN reduction percentage
	function getRpnReductionPct(initialRpn: number | undefined, residualRpn: number | undefined): number {
		if (!initialRpn || !residualRpn) return 0;
		return Math.round((1 - residualRpn / initialRpn) * 100);
	}
</script>

<div class="overflow-auto">
	<table class="w-full text-sm">
		<thead class="border-b bg-muted/50">
			<tr>
				<th class="whitespace-nowrap px-3 py-3 text-left font-medium">Hazard</th>
				<th class="whitespace-nowrap px-3 py-3 text-left font-medium">Risk / Failure Mode</th>
				{#if showInitialRisk && hasAnyInitialRisk()}
					<th colspan="4" class="whitespace-nowrap px-3 py-2 text-center font-medium border-b-0">
						<span class="text-orange-600 dark:text-orange-400">Initial Risk</span>
					</th>
				{/if}
				<th colspan="4" class="whitespace-nowrap px-3 py-2 text-center font-medium border-b-0">
					<span class="text-green-600 dark:text-green-400">{showInitialRisk && hasAnyInitialRisk() ? 'Residual Risk' : 'Risk Scores'}</span>
				</th>
				<th class="whitespace-nowrap px-3 py-3 text-center font-medium">Level</th>
				<th class="whitespace-nowrap px-3 py-3 text-left font-medium">Mitigations</th>
				<th class="whitespace-nowrap px-3 py-3 text-left font-medium">Controls</th>
				<th class="whitespace-nowrap px-3 py-3 text-left font-medium">Verification</th>
			</tr>
			<tr class="border-b bg-muted/30">
				<th class="px-3 py-1"></th>
				<th class="px-3 py-1"></th>
				{#if showInitialRisk && hasAnyInitialRisk()}
					<th class="whitespace-nowrap px-2 py-1 text-center text-xs font-medium text-orange-600 dark:text-orange-400">S</th>
					<th class="whitespace-nowrap px-2 py-1 text-center text-xs font-medium text-orange-600 dark:text-orange-400">O</th>
					<th class="whitespace-nowrap px-2 py-1 text-center text-xs font-medium text-orange-600 dark:text-orange-400">D</th>
					<th class="whitespace-nowrap px-2 py-1 text-center text-xs font-medium text-orange-600 dark:text-orange-400">RPN</th>
				{/if}
				<th class="whitespace-nowrap px-2 py-1 text-center text-xs font-medium text-green-600 dark:text-green-400">S</th>
				<th class="whitespace-nowrap px-2 py-1 text-center text-xs font-medium text-green-600 dark:text-green-400">O</th>
				<th class="whitespace-nowrap px-2 py-1 text-center text-xs font-medium text-green-600 dark:text-green-400">D</th>
				<th class="whitespace-nowrap px-2 py-1 text-center text-xs font-medium text-green-600 dark:text-green-400">RPN</th>
				<th class="px-3 py-1"></th>
				<th class="px-3 py-1"></th>
				<th class="px-3 py-1"></th>
				<th class="px-3 py-1"></th>
			</tr>
		</thead>
		<tbody class="divide-y">
			{#each displayData() as item (item.id)}
				<tr
					class="cursor-pointer transition-colors hover:bg-muted/50 align-top"
					onclick={() => handleRowClick(item)}
				>
					<!-- Hazards Column -->
					<td class="px-3 py-3 max-w-[120px]">
						{#if item.hazards && item.hazards.length > 0}
							<div class="space-y-1">
								{#each item.hazards as hazard}
									<button
										class="flex items-center gap-1 text-xs text-left hover:text-primary hover:underline w-full truncate"
										onclick={(e) => handleEntityClick(e, hazard.id)}
										title={hazard.title}
									>
										<AlertTriangle class="h-3 w-3 shrink-0 text-amber-500" />
										<span class="truncate">{hazard.title}</span>
									</button>
								{/each}
							</div>
						{:else}
							<span class="text-xs text-muted-foreground">-</span>
						{/if}
					</td>

					<!-- Risk / Failure Mode -->
					<td class="max-w-xs px-3 py-3">
						<p class="truncate font-medium" title={item.title}>{item.title}</p>
						{#if item.failure_mode}
							<p class="text-xs text-muted-foreground truncate" title={item.failure_mode}>
								{item.failure_mode}
							</p>
						{/if}
						{#if item.risk_type}
							<Badge variant="outline" class="mt-1 text-xs capitalize">{item.risk_type}</Badge>
						{/if}
					</td>

					<!-- Initial Risk S, O, D, RPN columns (shown if any risk has initial data) -->
					{#if showInitialRisk && hasAnyInitialRisk()}
						<td class="whitespace-nowrap px-2 py-3 text-center bg-orange-50/50 dark:bg-orange-950/20">
							<span class="font-mono text-xs text-orange-700 dark:text-orange-400">
								{item.initial_risk?.severity ?? '-'}
							</span>
						</td>
						<td class="whitespace-nowrap px-2 py-3 text-center bg-orange-50/50 dark:bg-orange-950/20">
							<span class="font-mono text-xs text-orange-700 dark:text-orange-400">
								{item.initial_risk?.occurrence ?? '-'}
							</span>
						</td>
						<td class="whitespace-nowrap px-2 py-3 text-center bg-orange-50/50 dark:bg-orange-950/20">
							<span class="font-mono text-xs text-orange-700 dark:text-orange-400">
								{item.initial_risk?.detection ?? '-'}
							</span>
						</td>
						<td class="whitespace-nowrap px-2 py-3 text-center bg-orange-50/50 dark:bg-orange-950/20">
							<span class="font-mono text-xs font-medium text-orange-700 dark:text-orange-400">
								{item.initial_risk?.rpn ?? '-'}
							</span>
						</td>
					{/if}

					<!-- Residual S, O, D, RPN columns -->
					<td class="whitespace-nowrap px-2 py-3 text-center bg-green-50/50 dark:bg-green-950/20">
						<span class="font-mono font-medium {(item.severity ?? 0) >= 8 ? 'text-red-500' : hasScoreReduction(item.initial_risk?.severity, item.severity) ? 'text-green-600 dark:text-green-400' : ''}">
							{item.severity ?? '-'}
						</span>
						{#if hasScoreReduction(item.initial_risk?.severity, item.severity)}
							<TrendingDown class="inline h-3 w-3 text-green-500 ml-0.5" />
						{/if}
					</td>
					<td class="whitespace-nowrap px-2 py-3 text-center bg-green-50/50 dark:bg-green-950/20">
						<span class="font-mono font-medium {(item.occurrence ?? 0) >= 8 ? 'text-orange-500' : hasScoreReduction(item.initial_risk?.occurrence, item.occurrence) ? 'text-green-600 dark:text-green-400' : ''}">
							{item.occurrence ?? '-'}
						</span>
						{#if hasScoreReduction(item.initial_risk?.occurrence, item.occurrence)}
							<TrendingDown class="inline h-3 w-3 text-green-500 ml-0.5" />
						{/if}
					</td>
					<td class="whitespace-nowrap px-2 py-3 text-center bg-green-50/50 dark:bg-green-950/20">
						<span class="font-mono font-medium {(item.detection ?? 0) >= 8 ? 'text-yellow-500' : hasScoreReduction(item.initial_risk?.detection, item.detection) ? 'text-green-600 dark:text-green-400' : ''}">
							{item.detection ?? '-'}
						</span>
						{#if hasScoreReduction(item.initial_risk?.detection, item.detection)}
							<TrendingDown class="inline h-3 w-3 text-green-500 ml-0.5" />
						{/if}
					</td>
					<td class="whitespace-nowrap px-2 py-3 text-center bg-green-50/50 dark:bg-green-950/20">
						<span class="font-mono {getRpnClass(item.rpn)} {getRpnReductionPct(item.initial_risk?.rpn, item.rpn) > 0 ? 'text-green-600 dark:text-green-400' : ''}">
							{item.rpn ?? '-'}
						</span>
						{#if getRpnReductionPct(item.initial_risk?.rpn, item.rpn) > 0}
							<span class="ml-1 text-xs text-green-600 dark:text-green-400">
								(-{getRpnReductionPct(item.initial_risk?.rpn, item.rpn)}%)
							</span>
						{/if}
					</td>
					<td class="whitespace-nowrap px-3 py-3 text-center">
						{#if item.risk_level}
							<Badge variant={getRiskLevelVariant(item.risk_level)} class="capitalize">
								{item.risk_level}
							</Badge>
						{:else}
							<span class="text-muted-foreground">-</span>
						{/if}
					</td>

					<!-- Mitigations Column -->
					<td class="px-3 py-3 max-w-[150px]">
						{#if item.mitigations && item.mitigations.length > 0}
							<div class="space-y-1">
								{#each item.mitigations.slice(0, 3) as mit}
									<div class="text-xs" title={mit.action}>
										<p class="truncate {getMitigationStatusClass(mit.status)}">{mit.action}</p>
										{#if mit.owner}
											<p class="text-muted-foreground truncate">@ {mit.owner}</p>
										{/if}
									</div>
								{/each}
								{#if item.mitigations.length > 3}
									<span class="text-xs text-muted-foreground">+{item.mitigations.length - 3} more</span>
								{/if}
							</div>
						{:else}
							<span class="text-xs text-muted-foreground">None</span>
						{/if}
					</td>

					<!-- Controls Column -->
					<td class="px-3 py-3 max-w-[150px]">
						{#if item.controls && item.controls.length > 0}
							<div class="space-y-1">
								{#each item.controls.filter(c => c.id) as ctrl}
									<button
										class="flex items-center gap-1 text-xs text-left hover:text-primary hover:underline w-full truncate"
										onclick={(e) => handleEntityClick(e, ctrl.id)}
										title={ctrl.title}
									>
										<Shield class="h-3 w-3 shrink-0 text-blue-500" />
										<span class="truncate">{ctrl.title}</span>
									</button>
								{/each}
							</div>
						{:else}
							<span class="text-xs text-muted-foreground">None</span>
						{/if}
					</td>

					<!-- Verification (Tests/Results) Column -->
					<td class="px-3 py-3 max-w-[150px]">
						{#if item.controls && item.controls.flatMap(c => c.tests).length > 0}
							{@const allTests = item.controls.flatMap(c => c.tests)}
							<div class="space-y-1">
								{#each allTests.slice(0, 3) as test}
									<button
										class="flex items-center gap-1 text-xs text-left hover:text-primary hover:underline w-full truncate"
										onclick={(e) => handleEntityClick(e, test.id)}
										title={test.title}
									>
										<FlaskConical class="h-3 w-3 shrink-0 text-purple-500" />
										<span class="truncate">{test.title}</span>
										{#if test.status === 'approved' || test.status === 'released'}
											<CheckCircle class="h-3 w-3 shrink-0 text-green-500" />
										{/if}
									</button>
								{/each}
								{#if allTests.length > 3}
									<span class="text-xs text-muted-foreground">+{allTests.length - 3} more</span>
								{/if}
							</div>
						{:else}
							<span class="text-xs text-muted-foreground">-</span>
						{/if}
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

{#if displayData().length === 0}
	<div class="flex h-32 items-center justify-center text-muted-foreground">
		No risks to display
	</div>
{/if}

<!-- Legend -->
<div class="mt-4 rounded-lg bg-muted/50 p-3 text-xs text-muted-foreground">
	<strong>Column Legend:</strong>
	S = Severity (1-10) |
	O = Occurrence (1-10) |
	D = Detection (1-10) |
	RPN = Risk Priority Number (S × O × D)
	<br />
	{#if showInitialRisk && hasAnyInitialRisk()}
		<strong>Risk Assessment:</strong>
		<span class="text-orange-600 dark:text-orange-400">Initial Risk</span> = Before mitigations |
		<span class="text-green-600 dark:text-green-400">Residual Risk</span> = After mitigations |
		<TrendingDown class="inline h-3 w-3 text-green-500" /> = Reduced from initial
		<br />
	{/if}
	<strong>Icons:</strong>
	<AlertTriangle class="inline h-3 w-3 text-amber-500" /> Hazard |
	<Shield class="inline h-3 w-3 text-blue-500" /> Control |
	<FlaskConical class="inline h-3 w-3 text-purple-500" /> Test/Result |
	<CheckCircle class="inline h-3 w-3 text-green-500" /> Verified
</div>
