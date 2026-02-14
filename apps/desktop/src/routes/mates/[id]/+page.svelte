<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import EntityHistory from '$lib/components/EntityHistory.svelte';
	import { StatusBadge } from '$lib/components/common';
	import { entities, traceability } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import { Plug, User, Calendar, Tag, CircleDot, ArrowLeftRight, BarChart3, History, AlertTriangle } from 'lucide-svelte';

	const id = $derived($page.params.id);

	let entity = $state<EntityData | null>(null);
	let linksFrom = $state<LinkInfo[]>([]);
	let linksTo = $state<LinkInfo[]>([]);
	let loading = $state(true);
	let linksLoading = $state(true);
	let error = $state<string | null>(null);

	// Type-safe data access
	const data = $derived(entity?.data ?? {});
	const mateType = $derived((data.mate_type as string) ?? 'clearance');
	const description = $derived((data.description as string) ?? null);
	const notes = $derived((data.notes as string) ?? null);
	const entityRevision = $derived((data.entity_revision as number) ?? 1);

	// Feature references
	interface MateFeatureRef {
		id: string;
		name?: string;
		component_id?: string;
		component_name?: string;
	}
	const featureA = $derived(data.feature_a as MateFeatureRef | null);
	const featureB = $derived(data.feature_b as MateFeatureRef | null);

	// Fit analysis
	interface StatisticalFit {
		mean_clearance: number;
		sigma_clearance: number;
		clearance_3sigma_min: number;
		clearance_3sigma_max: number;
		probability_interference: number;
		fit_result_3sigma: string;
	}

	interface FitAnalysis {
		worst_case_min_clearance: number;
		worst_case_max_clearance: number;
		fit_result: string;
		statistical?: StatisticalFit;
	}
	const fitAnalysis = $derived(data.fit_analysis as FitAnalysis | null);

	// Check if specified mate type matches calculated fit result
	const hasTypeMismatch = $derived(() => {
		if (!fitAnalysis?.fit_result) return false;
		return mateType.toLowerCase() !== fitAnalysis.fit_result.toLowerCase();
	});

	async function loadData() {
		if (!id) return;

		loading = true;
		linksLoading = true;
		error = null;

		try {
			const [entityResult, fromLinks, toLinks] = await Promise.all([
				entities.get(id),
				traceability.getLinksFrom(id),
				traceability.getLinksTo(id)
			]);

			entity = entityResult;
			linksFrom = fromLinks;
			linksTo = toLinks;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load mate:', e);
		} finally {
			loading = false;
			linksLoading = false;
		}
	}

	// Separate function to refresh just links (used after adding/removing links)
	async function refreshLinks() {
		if (!id) return;
		linksLoading = true;
		try {
			const [fromLinks, toLinks] = await Promise.all([
				traceability.getLinksFrom(id),
				traceability.getLinksTo(id)
			]);
			linksFrom = fromLinks;
			linksTo = toLinks;
		} catch (e) {
			console.error('Failed to refresh links:', e);
		} finally {
			linksLoading = false;
		}
	}

	function formatDate(dateStr: string): string {
		try {
			return new Date(dateStr).toLocaleDateString('en-US', {
				year: 'numeric',
				month: 'short',
				day: 'numeric'
			});
		} catch {
			return dateStr;
		}
	}

	function getMateTypeVariant(type: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			clearance: 'default',
			transition: 'secondary',
			interference: 'destructive'
		};
		return variants[type.toLowerCase()] ?? 'outline';
	}

	function getFitResultVariant(result: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			clearance: 'default',
			transition: 'secondary',
			interference: 'destructive'
		};
		return variants[result.toLowerCase()] ?? 'outline';
	}

	function formatClearance(value: number): string {
		if (value < 0) {
			return `${value.toFixed(4)} mm (interference)`;
		}
		return `${value.toFixed(4)} mm`;
	}

	// Track if we've loaded for this ID to prevent double-loads
	let loadedId = $state<string | null>(null);

	$effect(() => {
		// Only load if we have an ID and haven't already loaded this ID
		if (id && id !== loadedId) {
			loadedId = id;
			loadData();
		}
	});
</script>

<div class="space-y-6">
	{#if loading}
		<div class="flex h-64 items-center justify-center">
			<div class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
		</div>
	{:else if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{:else if entity}
		<!-- Header -->
		<EntityDetailHeader
			id={entity.id}
			title={entity.title}
			status={entity.status}
			subtitle={`${mateType.charAt(0).toUpperCase() + mateType.slice(1)} Fit`}
			backHref="/mates"
			backLabel="Mates"
			onEdit={() => goto(`/mates/${id}/edit`)}
		/>

		<!-- Type Mismatch Warning -->
		{#if hasTypeMismatch()}
			<Card class="border-destructive bg-destructive/10">
				<CardContent class="flex items-center gap-3 pt-6">
					<AlertTriangle class="h-5 w-5 text-destructive" />
					<div>
						<p class="font-medium text-destructive">Fit Type Mismatch</p>
						<p class="text-sm text-muted-foreground">
							Specified mate type is <Badge variant={getMateTypeVariant(mateType)} class="capitalize mx-1">{mateType}</Badge>
							but calculated fit result is <Badge variant={getFitResultVariant(fitAnalysis?.fit_result ?? '')} class="capitalize mx-1">{fitAnalysis?.fit_result}</Badge>.
							The tolerances don't achieve the intended fit.
						</p>
					</div>
				</CardContent>
			</Card>
		{/if}

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Description -->
				{#if description}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Plug class="h-5 w-5" />
								Description
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap">{description}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Feature Connection -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<ArrowLeftRight class="h-5 w-5" />
							Feature Connection
						</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="flex items-center gap-4">
							<!-- Feature A -->
							<div class="flex-1">
								<button
									class="w-full rounded-lg border p-4 text-left transition-colors hover:bg-muted/50"
									onclick={() => featureA && goto(`/features/${featureA.id}`)}
									disabled={!featureA}
								>
									<div class="flex items-center gap-2 text-sm text-muted-foreground">
										<CircleDot class="h-4 w-4" />
										Feature A (Hole/Bore)
									</div>
									{#if featureA}
										<p class="mt-2 font-medium">{featureA.name || featureA.id}</p>
										{#if featureA.component_name || featureA.component_id}
											<p class="mt-1 text-sm text-muted-foreground">
												on {featureA.component_name || featureA.component_id}
											</p>
										{/if}
									{:else}
										<p class="mt-2 text-muted-foreground">Not specified</p>
									{/if}
								</button>
							</div>

							<Plug class="h-6 w-6 text-muted-foreground" />

							<!-- Feature B -->
							<div class="flex-1">
								<button
									class="w-full rounded-lg border p-4 text-left transition-colors hover:bg-muted/50"
									onclick={() => featureB && goto(`/features/${featureB.id}`)}
									disabled={!featureB}
								>
									<div class="flex items-center gap-2 text-sm text-muted-foreground">
										<CircleDot class="h-4 w-4" />
										Feature B (Shaft/Pin)
									</div>
									{#if featureB}
										<p class="mt-2 font-medium">{featureB.name || featureB.id}</p>
										{#if featureB.component_name || featureB.component_id}
											<p class="mt-1 text-sm text-muted-foreground">
												on {featureB.component_name || featureB.component_id}
											</p>
										{/if}
									{:else}
										<p class="mt-2 text-muted-foreground">Not specified</p>
									{/if}
								</button>
							</div>
						</div>
					</CardContent>
				</Card>

				<!-- Fit Analysis -->
				{#if fitAnalysis}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<BarChart3 class="h-5 w-5" />
								Fit Analysis
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-6">
							<!-- Worst Case -->
							<div>
								<h4 class="mb-3 text-sm font-medium text-muted-foreground">Worst Case Analysis</h4>
								<div class="grid gap-4 sm:grid-cols-3">
									<div class="rounded-lg border p-4 text-center">
										<div class="text-sm text-muted-foreground">Min Clearance</div>
										<div class="mt-1 font-mono text-lg font-bold">
											{fitAnalysis.worst_case_min_clearance.toFixed(4)}
										</div>
										<div class="text-xs text-muted-foreground">mm</div>
									</div>
									<div class="rounded-lg border p-4 text-center">
										<div class="text-sm text-muted-foreground">Max Clearance</div>
										<div class="mt-1 font-mono text-lg font-bold">
											{fitAnalysis.worst_case_max_clearance.toFixed(4)}
										</div>
										<div class="text-xs text-muted-foreground">mm</div>
									</div>
									<div class="rounded-lg border p-4 text-center">
										<div class="text-sm text-muted-foreground">Fit Result</div>
										<div class="mt-2">
											<Badge variant={getFitResultVariant(fitAnalysis.fit_result)} class="capitalize">
												{fitAnalysis.fit_result}
											</Badge>
										</div>
									</div>
								</div>
							</div>

							<!-- Statistical Analysis -->
							{#if fitAnalysis.statistical}
								<div>
									<h4 class="mb-3 text-sm font-medium text-muted-foreground">Statistical Analysis (3σ)</h4>
									<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">Mean Clearance</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{fitAnalysis.statistical.mean_clearance.toFixed(4)}
											</div>
											<div class="text-xs text-muted-foreground">mm</div>
										</div>
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">Sigma</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{fitAnalysis.statistical.sigma_clearance.toFixed(4)}
											</div>
											<div class="text-xs text-muted-foreground">mm</div>
										</div>
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">3σ Range</div>
											<div class="mt-1 font-mono text-sm font-bold">
												{fitAnalysis.statistical.clearance_3sigma_min.toFixed(4)} to
												{fitAnalysis.statistical.clearance_3sigma_max.toFixed(4)}
											</div>
											<div class="text-xs text-muted-foreground">mm</div>
										</div>
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">P(Interference)</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{(fitAnalysis.statistical.probability_interference * 100).toFixed(2)}%
											</div>
										</div>
									</div>
									<div class="mt-4 text-center">
										<span class="text-sm text-muted-foreground">3σ Fit Result: </span>
										<Badge
											variant={getFitResultVariant(fitAnalysis.statistical.fit_result_3sigma)}
											class="capitalize"
										>
											{fitAnalysis.statistical.fit_result_3sigma}
										</Badge>
									</div>
								</div>
							{/if}
						</CardContent>
					</Card>
				{/if}

				<!-- Notes -->
				{#if notes}
					<Card>
						<CardHeader>
							<CardTitle>Notes</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap text-muted-foreground">{notes}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Links -->
				<LinksSection
					{linksFrom}
					{linksTo}
					loading={linksLoading}
					entityId={entity?.id}
					onLinksChanged={refreshLinks}
				/>

				<!-- History -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<History class="h-5 w-5" />
							History
						</CardTitle>
					</CardHeader>
					<CardContent>
						<EntityHistory entityId={entity.id} />
					</CardContent>
				</Card>
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Mate Info -->
				<Card>
					<CardHeader>
						<CardTitle>Mate Information</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Mate Type</span>
							<Badge variant={getMateTypeVariant(mateType)} class="capitalize">
								{mateType}
							</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						{#if fitAnalysis}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Fit Result</span>
								<Badge variant={getFitResultVariant(fitAnalysis.fit_result)} class="capitalize">
									{fitAnalysis.fit_result}
								</Badge>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Metadata -->
				<Card>
					<CardHeader>
						<CardTitle>Metadata</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center gap-2">
							<User class="h-4 w-4 text-muted-foreground" />
							<span class="text-sm text-muted-foreground">Author</span>
							<span class="ml-auto text-sm font-medium">{entity.author}</span>
						</div>
						<div class="flex items-center gap-2">
							<Calendar class="h-4 w-4 text-muted-foreground" />
							<span class="text-sm text-muted-foreground">Created</span>
							<span class="ml-auto text-sm font-medium">{formatDate(entity.created)}</span>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Entity Revision</span>
							<span class="text-sm font-medium">{entityRevision}</span>
						</div>
					</CardContent>
				</Card>

				<!-- Tags -->
				{#if entity.tags && entity.tags.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Tag class="h-4 w-4" />
								Tags
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="flex flex-wrap gap-2">
								{#each entity.tags as tag}
									<Badge variant="outline">{tag}</Badge>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}
			</div>
		</div>
	{:else}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Mate not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
