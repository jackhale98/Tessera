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
	import {
		Ruler,
		User,
		Calendar,
		Tag,
		Target,
		TrendingUp,
		AlertTriangle,
		CheckCircle2,
		BarChart3,
		ArrowUp,
		ArrowDown,
		History
	} from 'lucide-svelte';

	const id = $derived($page.params.id);

	let entity = $state<EntityData | null>(null);
	let linksFrom = $state<LinkInfo[]>([]);
	let linksTo = $state<LinkInfo[]>([]);
	let loading = $state(true);
	let linksLoading = $state(true);
	let error = $state<string | null>(null);

	// Type-safe data access
	const data = $derived(entity?.data ?? {});
	const description = $derived((data.description as string) ?? null);
	const disposition = $derived((data.disposition as string) ?? 'under_review');
	const sigmaLevel = $derived((data.sigma_level as number) ?? 6.0);
	const meanShiftK = $derived((data.mean_shift_k as number) ?? 0);
	const includeGdt = $derived((data.include_gdt as boolean) ?? false);
	const entityRevision = $derived((data.entity_revision as number) ?? 1);

	// Target specification
	interface TargetSpec {
		name: string;
		nominal: number;
		upper_limit: number;
		lower_limit: number;
		units: string;
		critical: boolean;
	}
	const target = $derived(data.target as TargetSpec | null);

	// Contributors
	interface FeatureRef {
		id: string;
		name?: string;
		component_id?: string;
		component_name?: string;
	}

	interface Contributor {
		name: string;
		feature?: FeatureRef;
		direction: string;
		nominal: number;
		plus_tol: number;
		minus_tol: number;
		distribution?: string;
		source?: string;
	}
	const contributors = $derived((data.contributors as Contributor[]) ?? []);

	// Analysis results
	interface WorstCaseResult {
		min: number;
		max: number;
		margin: number;
		result: string;
	}

	interface RssResult {
		mean: number;
		sigma_3: number;
		margin: number;
		cp: number;
		cpk: number;
		yield_percent: number;
		sensitivity?: number[];
	}

	interface MonteCarloResult {
		iterations: number;
		mean: number;
		std_dev: number;
		min: number;
		max: number;
		yield_percent: number;
		percentile_2_5: number;
		percentile_97_5: number;
		pp?: number;
		ppk?: number;
	}

	interface AnalysisResults {
		worst_case?: WorstCaseResult;
		rss?: RssResult;
		monte_carlo?: MonteCarloResult;
	}
	const analysisResults = $derived(data.analysis_results as AnalysisResults | null);

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
			console.error('Failed to load stackup:', e);
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

	function getResultVariant(result: string | undefined): 'default' | 'secondary' | 'destructive' | 'outline' {
		if (!result) return 'outline';
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			pass: 'default',
			marginal: 'secondary',
			fail: 'destructive'
		};
		return variants[result.toLowerCase()] ?? 'outline';
	}

	function getDispositionVariant(disp: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			approved: 'default',
			under_review: 'secondary',
			underreview: 'secondary',
			rejected: 'destructive'
		};
		return variants[disp.toLowerCase()] ?? 'outline';
	}

	function formatDisposition(disp: string): string {
		return disp.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase());
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
			subtitle="Tolerance Stackup Analysis"
			backHref="/tolerances"
			backLabel="Stackups"
			onEdit={() => goto(`/tolerances/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Description -->
				{#if description}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Ruler class="h-5 w-5" />
								Description
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap">{description}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Target Specification -->
				{#if target}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Target class="h-5 w-5" />
								Target Specification
								{#if target.critical}
									<Badge variant="destructive">Critical</Badge>
								{/if}
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="grid gap-4 sm:grid-cols-4">
								<div class="rounded-lg border p-4 text-center">
									<div class="text-sm text-muted-foreground">Name</div>
									<div class="mt-1 font-medium">{target.name}</div>
								</div>
								<div class="rounded-lg border p-4 text-center">
									<div class="text-sm text-muted-foreground">Nominal</div>
									<div class="mt-1 font-mono text-lg font-bold">
										{target.nominal} {target.units}
									</div>
								</div>
								<div class="rounded-lg border p-4 text-center">
									<div class="text-sm text-muted-foreground">LSL</div>
									<div class="mt-1 font-mono text-lg font-bold">
										{target.lower_limit} {target.units}
									</div>
								</div>
								<div class="rounded-lg border p-4 text-center">
									<div class="text-sm text-muted-foreground">USL</div>
									<div class="mt-1 font-mono text-lg font-bold">
										{target.upper_limit} {target.units}
									</div>
								</div>
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Contributors -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<TrendingUp class="h-5 w-5" />
							Contributors ({contributors.length})
						</CardTitle>
					</CardHeader>
					<CardContent>
						{#if contributors.length === 0}
							<p class="py-4 text-center text-muted-foreground">No contributors defined</p>
						{:else}
							<div class="space-y-2">
								{#each contributors as contrib, i}
									<div class="rounded-lg border p-4">
										<div class="flex items-start justify-between">
											<div class="flex items-center gap-3">
												<span class="text-sm text-muted-foreground">#{i + 1}</span>
												{#if contrib.direction === 'positive'}
													<ArrowUp class="h-4 w-4 text-green-500" />
												{:else}
													<ArrowDown class="h-4 w-4 text-red-500" />
												{/if}
												<div>
													<p class="font-medium">{contrib.name}</p>
													{#if contrib.feature}
														<button
															class="mt-1 text-sm text-primary hover:underline"
															onclick={() => contrib.feature && goto(`/features/${contrib.feature.id}`)}
														>
															{contrib.feature.name || contrib.feature.id}
															{#if contrib.feature.component_name}
																<span class="text-muted-foreground">
																	on {contrib.feature.component_name}
																</span>
															{/if}
														</button>
													{/if}
													{#if contrib.source}
														<p class="mt-1 text-xs text-muted-foreground">{contrib.source}</p>
													{/if}
												</div>
											</div>
											<div class="text-right">
												<p class="font-mono">
													{contrib.nominal} +{contrib.plus_tol}/-{contrib.minus_tol}
												</p>
												{#if contrib.distribution}
													<Badge variant="outline" class="mt-1 capitalize text-xs">
														{contrib.distribution}
													</Badge>
												{/if}
											</div>
										</div>
									</div>
								{/each}
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Analysis Results -->
				{#if analysisResults}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<BarChart3 class="h-5 w-5" />
								Analysis Results
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-6">
							<!-- Worst Case -->
							{#if analysisResults.worst_case}
								<div>
									<div class="mb-3 flex items-center justify-between">
										<h4 class="text-sm font-medium text-muted-foreground">Worst Case Analysis</h4>
										<Badge variant={getResultVariant(analysisResults.worst_case.result)} class="capitalize">
											{analysisResults.worst_case.result}
										</Badge>
									</div>
									<div class="grid gap-4 sm:grid-cols-3">
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">Minimum</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{analysisResults.worst_case.min.toFixed(4)}
											</div>
										</div>
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">Maximum</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{analysisResults.worst_case.max.toFixed(4)}
											</div>
										</div>
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">Margin</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{analysisResults.worst_case.margin.toFixed(4)}
											</div>
										</div>
									</div>
								</div>
							{/if}

							<!-- RSS Analysis -->
							{#if analysisResults.rss}
								<div>
									<div class="mb-3 flex items-center justify-between">
										<h4 class="text-sm font-medium text-muted-foreground">RSS Analysis</h4>
										<span class="text-sm text-muted-foreground">
											Yield: {analysisResults.rss.yield_percent.toFixed(2)}%
										</span>
									</div>
									<div class="grid gap-4 sm:grid-cols-4">
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">Mean</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{analysisResults.rss.mean.toFixed(4)}
											</div>
										</div>
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">3σ</div>
											<div class="mt-1 font-mono text-lg font-bold">
												±{analysisResults.rss.sigma_3.toFixed(4)}
											</div>
										</div>
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">Cp</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{analysisResults.rss.cp.toFixed(3)}
											</div>
										</div>
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">Cpk</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{analysisResults.rss.cpk.toFixed(3)}
											</div>
										</div>
									</div>
								</div>
							{/if}

							<!-- Monte Carlo -->
							{#if analysisResults.monte_carlo}
								<div>
									<div class="mb-3 flex items-center justify-between">
										<h4 class="text-sm font-medium text-muted-foreground">
											Monte Carlo ({analysisResults.monte_carlo.iterations.toLocaleString()} iterations)
										</h4>
										<span class="text-sm text-muted-foreground">
											Yield: {analysisResults.monte_carlo.yield_percent.toFixed(2)}%
										</span>
									</div>
									<div class="grid gap-4 sm:grid-cols-4">
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">Mean</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{analysisResults.monte_carlo.mean.toFixed(4)}
											</div>
										</div>
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">Std Dev</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{analysisResults.monte_carlo.std_dev.toFixed(4)}
											</div>
										</div>
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">2.5th %ile</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{analysisResults.monte_carlo.percentile_2_5.toFixed(4)}
											</div>
										</div>
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">97.5th %ile</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{analysisResults.monte_carlo.percentile_97_5.toFixed(4)}
											</div>
										</div>
									</div>
									{#if analysisResults.monte_carlo.pp || analysisResults.monte_carlo.ppk}
										<div class="mt-4 grid gap-4 sm:grid-cols-2">
											{#if analysisResults.monte_carlo.pp}
												<div class="rounded-lg border p-4 text-center">
													<div class="text-sm text-muted-foreground">Pp</div>
													<div class="mt-1 font-mono text-lg font-bold">
														{analysisResults.monte_carlo.pp.toFixed(3)}
													</div>
												</div>
											{/if}
											{#if analysisResults.monte_carlo.ppk}
												<div class="rounded-lg border p-4 text-center">
													<div class="text-sm text-muted-foreground">Ppk</div>
													<div class="mt-1 font-mono text-lg font-bold">
														{analysisResults.monte_carlo.ppk.toFixed(3)}
													</div>
												</div>
											{/if}
										</div>
									{/if}
								</div>
							{/if}
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
				<!-- Stackup Info -->
				<Card>
					<CardHeader>
						<CardTitle>Stackup Information</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Disposition</span>
							<Badge variant={getDispositionVariant(disposition)}>
								{formatDisposition(disposition)}
							</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Contributors</span>
							<Badge variant="outline">{contributors.length}</Badge>
						</div>
						{#if target?.critical}
							<div class="flex items-center gap-2 text-destructive">
								<AlertTriangle class="h-4 w-4" />
								<span class="text-sm font-medium">Critical Dimension</span>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Analysis Settings -->
				<Card>
					<CardHeader>
						<CardTitle>Analysis Settings</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Sigma Level</span>
							<span class="font-medium">{sigmaLevel}σ</span>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Mean Shift (k)</span>
							<span class="font-medium">{meanShiftK}</span>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Include GD&T</span>
							<Badge variant={includeGdt ? 'default' : 'outline'}>
								{includeGdt ? 'Yes' : 'No'}
							</Badge>
						</div>
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
				<p class="text-muted-foreground">Stackup not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
