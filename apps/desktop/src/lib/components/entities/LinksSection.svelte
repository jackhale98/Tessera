<script lang="ts">
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui';
	import { StatusBadge } from '$lib/components/common';
	import { Link2, ArrowRight, ArrowLeft } from 'lucide-svelte';
	import type { LinkInfo } from '$lib/api/tauri';

	interface Props {
		linksFrom: LinkInfo[];
		linksTo: LinkInfo[];
		loading?: boolean;
	}

	let { linksFrom = [], linksTo = [], loading = false }: Props = $props();

	function getEntityRoute(id: string): string {
		const prefix = id.split('-')[0]?.toUpperCase();
		const routeMap: Record<string, string> = {
			REQ: '/requirements',
			RISK: '/risks',
			HAZ: '/hazards',
			TEST: '/verification/tests',
			RSLT: '/verification/results',
			CMP: '/components',
			ASM: '/assemblies',
			FEAT: '/features',
			MATE: '/mates',
			TOL: '/tolerances',
			PROC: '/manufacturing/processes',
			CTRL: '/controls',
			WORK: '/manufacturing/work-instructions',
			LOT: '/manufacturing/lots',
			DEV: '/manufacturing/deviations',
			NCR: '/quality/ncrs',
			CAPA: '/quality/capas',
			QUOT: '/procurement/quotes',
			SUP: '/procurement/suppliers'
		};
		return `${routeMap[prefix] ?? '/entities'}/${id}`;
	}

	function formatLinkType(linkType: string): string {
		return linkType.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase());
	}

	// Group links by type
	const groupedLinksFrom = $derived(
		linksFrom.reduce(
			(acc, link) => {
				const type = link.link_type || 'related';
				if (!acc[type]) acc[type] = [];
				acc[type].push(link);
				return acc;
			},
			{} as Record<string, LinkInfo[]>
		)
	);

	const groupedLinksTo = $derived(
		linksTo.reduce(
			(acc, link) => {
				const type = link.link_type || 'related';
				if (!acc[type]) acc[type] = [];
				acc[type].push(link);
				return acc;
			},
			{} as Record<string, LinkInfo[]>
		)
	);

	const hasLinks = $derived(linksFrom.length > 0 || linksTo.length > 0);
</script>

<Card>
	<CardHeader>
		<CardTitle class="flex items-center gap-2">
			<Link2 class="h-5 w-5" />
			Links & Traceability
		</CardTitle>
	</CardHeader>
	<CardContent>
		{#if loading}
			<div class="flex items-center justify-center py-8">
				<div class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
			</div>
		{:else if !hasLinks}
			<p class="py-4 text-center text-muted-foreground">No links defined</p>
		{:else}
			<div class="space-y-6">
				<!-- Outgoing links (from this entity) -->
				{#if linksFrom.length > 0}
					<div class="space-y-3">
						<h4 class="flex items-center gap-2 text-sm font-medium text-muted-foreground">
							<ArrowRight class="h-4 w-4" />
							Outgoing Links ({linksFrom.length})
						</h4>
						{#each Object.entries(groupedLinksFrom) as [linkType, links]}
							<div class="space-y-2">
								<p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
									{formatLinkType(linkType)}
								</p>
								<div class="space-y-1">
									{#each links as link}
										<button
											class="flex w-full items-center justify-between rounded-lg border p-3 text-left transition-colors hover:bg-muted/50"
											onclick={() => goto(getEntityRoute(link.target_id))}
										>
											<div class="min-w-0 flex-1">
												<p class="truncate font-medium">
													{link.target_title || link.target_id}
												</p>
												<p class="font-mono text-xs text-muted-foreground">
													{link.target_id}
												</p>
											</div>
											{#if link.target_status}
												<StatusBadge status={link.target_status} class="ml-2" />
											{/if}
										</button>
									{/each}
								</div>
							</div>
						{/each}
					</div>
				{/if}

				<!-- Incoming links (to this entity) -->
				{#if linksTo.length > 0}
					<div class="space-y-3">
						<h4 class="flex items-center gap-2 text-sm font-medium text-muted-foreground">
							<ArrowLeft class="h-4 w-4" />
							Incoming Links ({linksTo.length})
						</h4>
						{#each Object.entries(groupedLinksTo) as [linkType, links]}
							<div class="space-y-2">
								<p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
									{formatLinkType(linkType)}
								</p>
								<div class="space-y-1">
									{#each links as link}
										<button
											class="flex w-full items-center justify-between rounded-lg border p-3 text-left transition-colors hover:bg-muted/50"
											onclick={() => goto(getEntityRoute(link.source_id))}
										>
											<div class="min-w-0 flex-1">
												<p class="truncate font-medium">
													{link.target_title || link.source_id}
												</p>
												<p class="font-mono text-xs text-muted-foreground">
													{link.source_id}
												</p>
											</div>
											{#if link.target_status}
												<StatusBadge status={link.target_status} class="ml-2" />
											{/if}
										</button>
									{/each}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	</CardContent>
</Card>
