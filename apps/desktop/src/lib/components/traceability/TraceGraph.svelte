<script lang="ts">
	import { goto } from '$app/navigation';
	import { Badge } from '$lib/components/ui';
	import { StatusBadge } from '$lib/components/common';
	import type { TraceResult, TraceLink, EntityPrefix } from '$lib/api/types';
	import { ArrowLeft, ArrowRight, Circle } from 'lucide-svelte';

	interface Props {
		traceResult: TraceResult;
		onSelectEntity?: (id: string) => void;
	}

	let { traceResult, onSelectEntity }: Props = $props();

	function getEntityColor(prefix: EntityPrefix): string {
		const colors: Record<string, string> = {
			REQ: 'bg-blue-500',
			RISK: 'bg-red-500',
			HAZ: 'bg-orange-500',
			TEST: 'bg-green-500',
			RSLT: 'bg-emerald-500',
			CMP: 'bg-purple-500',
			ASM: 'bg-violet-500',
			FEAT: 'bg-cyan-500',
			MATE: 'bg-teal-500',
			TOL: 'bg-indigo-500',
			PROC: 'bg-amber-500',
			CTRL: 'bg-yellow-500',
			WORK: 'bg-lime-500',
			LOT: 'bg-pink-500',
			DEV: 'bg-rose-500',
			NCR: 'bg-red-600',
			CAPA: 'bg-orange-600',
			QUOT: 'bg-sky-500',
			SUP: 'bg-slate-500'
		};
		return colors[prefix] ?? 'bg-gray-500';
	}

	function getEntityRoute(prefix: EntityPrefix): string {
		const routes: Record<string, string> = {
			REQ: 'requirements',
			RISK: 'risks',
			HAZ: 'hazards',
			TEST: 'verification/tests',
			RSLT: 'verification/results',
			CMP: 'components',
			ASM: 'assemblies',
			FEAT: 'features',
			MATE: 'mates',
			TOL: 'tolerances',
			PROC: 'manufacturing/processes',
			CTRL: 'controls',
			WORK: 'manufacturing/work-instructions',
			LOT: 'manufacturing/lots',
			DEV: 'manufacturing/deviations',
			NCR: 'quality/ncrs',
			CAPA: 'quality/capas',
			QUOT: 'procurement/quotes',
			SUP: 'procurement/suppliers'
		};
		return routes[prefix] ?? 'entities';
	}

	function handleEntityClick(link: TraceLink) {
		if (onSelectEntity) {
			onSelectEntity(link.entity_id);
		} else {
			goto(`/${getEntityRoute(link.entity_type)}/${link.entity_id}`);
		}
	}

	function handleCenterClick() {
		if (onSelectEntity) {
			onSelectEntity(traceResult.entity_id);
		} else {
			goto(`/${getEntityRoute(traceResult.entity_type)}/${traceResult.entity_id}`);
		}
	}
</script>

<div class="flex items-center justify-center gap-8 py-8">
	<!-- Upstream (links TO this entity) -->
	<div class="flex flex-col items-end gap-2">
		{#if traceResult.upstream.length > 0}
			<div class="mb-2 text-sm font-medium text-muted-foreground">Upstream</div>
			{#each traceResult.upstream as link}
				<button
					class="flex items-center gap-2 rounded-lg border bg-card p-3 text-left transition-colors hover:bg-muted/50"
					onclick={() => handleEntityClick(link)}
				>
					<div class="flex h-8 w-8 items-center justify-center rounded-full {getEntityColor(link.entity_type)}">
						<span class="text-xs font-bold text-white">{link.entity_type}</span>
					</div>
					<div class="max-w-48">
						<p class="truncate text-sm font-medium">{link.title}</p>
						<p class="text-xs text-muted-foreground">{link.link_type}</p>
					</div>
				</button>
			{/each}
		{:else}
			<div class="flex h-20 items-center text-sm text-muted-foreground">No upstream links</div>
		{/if}
	</div>

	<!-- Arrows left -->
	{#if traceResult.upstream.length > 0}
		<div class="flex flex-col items-center justify-center">
			<ArrowRight class="h-6 w-6 text-muted-foreground" />
		</div>
	{/if}

	<!-- Center entity -->
	<button
		class="flex flex-col items-center gap-2 rounded-xl border-2 border-primary bg-card p-6 shadow-lg transition-transform hover:scale-105"
		onclick={handleCenterClick}
	>
		<div class="flex h-16 w-16 items-center justify-center rounded-full {getEntityColor(traceResult.entity_type)}">
			<span class="text-lg font-bold text-white">{traceResult.entity_type}</span>
		</div>
		<div class="max-w-64 text-center">
			<p class="font-medium">{traceResult.title}</p>
			<p class="mt-1 font-mono text-xs text-muted-foreground">{traceResult.entity_id}</p>
		</div>
	</button>

	<!-- Arrows right -->
	{#if traceResult.downstream.length > 0}
		<div class="flex flex-col items-center justify-center">
			<ArrowRight class="h-6 w-6 text-muted-foreground" />
		</div>
	{/if}

	<!-- Downstream (links FROM this entity) -->
	<div class="flex flex-col items-start gap-2">
		{#if traceResult.downstream.length > 0}
			<div class="mb-2 text-sm font-medium text-muted-foreground">Downstream</div>
			{#each traceResult.downstream as link}
				<button
					class="flex items-center gap-2 rounded-lg border bg-card p-3 text-left transition-colors hover:bg-muted/50"
					onclick={() => handleEntityClick(link)}
				>
					<div class="flex h-8 w-8 items-center justify-center rounded-full {getEntityColor(link.entity_type)}">
						<span class="text-xs font-bold text-white">{link.entity_type}</span>
					</div>
					<div class="max-w-48">
						<p class="truncate text-sm font-medium">{link.title}</p>
						<p class="text-xs text-muted-foreground">{link.link_type}</p>
					</div>
				</button>
			{/each}
		{:else}
			<div class="flex h-20 items-center text-sm text-muted-foreground">No downstream links</div>
		{/if}
	</div>
</div>
