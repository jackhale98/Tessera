<script lang="ts">
	import { goto } from '$app/navigation';
	import { Badge } from '$lib/components/ui';
	import { StatusBadge } from '$lib/components/common';
	import type { RiskSummary } from '$lib/api/tauri';
	import { ExternalLink } from 'lucide-svelte';

	interface Props {
		risks: RiskSummary[];
		onRowClick?: (risk: RiskSummary) => void;
	}

	let { risks, onRowClick }: Props = $props();

	function handleRowClick(risk: RiskSummary) {
		if (onRowClick) {
			onRowClick(risk);
		} else {
			goto(`/risks/${risk.id}`);
		}
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

	function truncateId(id: string): string {
		const parts = id.split('-');
		if (parts.length === 2 && parts[1].length > 8) {
			return `${parts[0]}-${parts[1].slice(0, 6)}...`;
		}
		return id;
	}
</script>

<div class="overflow-auto">
	<table class="w-full text-sm">
		<thead class="border-b bg-muted/50">
			<tr>
				<th class="whitespace-nowrap px-3 py-3 text-left font-medium">ID</th>
				<th class="whitespace-nowrap px-3 py-3 text-left font-medium">Item/Function</th>
				<th class="whitespace-nowrap px-3 py-3 text-left font-medium">Failure Mode</th>
				<th class="whitespace-nowrap px-3 py-3 text-center font-medium">S</th>
				<th class="whitespace-nowrap px-3 py-3 text-center font-medium">O</th>
				<th class="whitespace-nowrap px-3 py-3 text-center font-medium">D</th>
				<th class="whitespace-nowrap px-3 py-3 text-center font-medium">RPN</th>
				<th class="whitespace-nowrap px-3 py-3 text-center font-medium">Level</th>
				<th class="whitespace-nowrap px-3 py-3 text-center font-medium">Mitigations</th>
				<th class="whitespace-nowrap px-3 py-3 text-center font-medium">Status</th>
			</tr>
		</thead>
		<tbody class="divide-y">
			{#each risks as risk}
				<tr
					class="cursor-pointer transition-colors hover:bg-muted/50"
					onclick={() => handleRowClick(risk)}
				>
					<td class="whitespace-nowrap px-3 py-3">
						<div class="flex items-center gap-1">
							<span class="font-mono text-xs text-muted-foreground" title={risk.id}>
								{truncateId(risk.id)}
							</span>
							<ExternalLink class="h-3 w-3 text-muted-foreground" />
						</div>
					</td>
					<td class="max-w-xs px-3 py-3">
						<p class="truncate font-medium" title={risk.title}>{risk.title}</p>
						{#if risk.risk_type}
							<p class="text-xs text-muted-foreground capitalize">{risk.risk_type}</p>
						{/if}
					</td>
					<td class="max-w-xs px-3 py-3">
						<p class="truncate text-muted-foreground" title={risk.failure_mode}>
							{risk.failure_mode || '-'}
						</p>
					</td>
					<td class="whitespace-nowrap px-3 py-3 text-center">
						<span class="font-mono font-medium {(risk.severity ?? 0) >= 8 ? 'text-red-500' : ''}">
							{risk.severity ?? '-'}
						</span>
					</td>
					<td class="whitespace-nowrap px-3 py-3 text-center">
						<span class="font-mono font-medium {(risk.occurrence ?? 0) >= 8 ? 'text-orange-500' : ''}">
							{risk.occurrence ?? '-'}
						</span>
					</td>
					<td class="whitespace-nowrap px-3 py-3 text-center">
						<span class="font-mono font-medium {(risk.detection ?? 0) >= 8 ? 'text-yellow-500' : ''}">
							{risk.detection ?? '-'}
						</span>
					</td>
					<td class="whitespace-nowrap px-3 py-3 text-center">
						<span class="font-mono {getRpnClass(risk.rpn)}">
							{risk.rpn ?? '-'}
						</span>
					</td>
					<td class="whitespace-nowrap px-3 py-3 text-center">
						{#if risk.risk_level}
							<Badge variant={getRiskLevelVariant(risk.risk_level)} class="capitalize">
								{risk.risk_level}
							</Badge>
						{:else}
							<span class="text-muted-foreground">-</span>
						{/if}
					</td>
					<td class="whitespace-nowrap px-3 py-3 text-center">
						<Badge variant={risk.mitigation_count > 0 ? 'default' : 'outline'}>
							{risk.mitigation_count}
						</Badge>
					</td>
					<td class="whitespace-nowrap px-3 py-3 text-center">
						<StatusBadge status={risk.status} />
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

{#if risks.length === 0}
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
</div>
