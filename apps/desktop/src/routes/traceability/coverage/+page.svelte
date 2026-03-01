<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge, Button } from '$lib/components/ui';
	import { CoverageCard } from '$lib/components/traceability';
	import { traceability, type MaturityMismatch } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import { getEntityRoute } from '$lib/config/entities';
	import type { CoverageReport } from '$lib/api/types';
	import {
		FileCheck,
		TestTube,
		Shield,
		PlayCircle,
		RefreshCw,
		TrendingUp,
		AlertTriangle,
		ArrowRight
	} from 'lucide-svelte';

	let coverage = $state<CoverageReport | null>(null);
	let mismatches = $state<MaturityMismatch[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function loadCoverage() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			const [coverageResult, mismatchResult] = await Promise.all([
				traceability.getCoverage(),
				traceability.getMaturityMismatches()
			]);
			coverage = coverageResult;
			mismatches = mismatchResult;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	const overallHealth = $derived(() => {
		if (!coverage) return 0;
		// Use the pre-calculated health_score from the backend
		return coverage.health_score;
	});

	const healthStatus = $derived(() => {
		const health = overallHealth();
		if (health >= 80) return { label: 'Healthy', color: 'text-green-500', bg: 'bg-green-500' };
		if (health >= 50) return { label: 'Needs Attention', color: 'text-yellow-500', bg: 'bg-yellow-500' };
		return { label: 'Critical', color: 'text-red-500', bg: 'bg-red-500' };
	});

	onMount(() => {
		loadCoverage();
	});

	$effect(() => {
		if ($isProjectOpen) {
			loadCoverage();
		}
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Coverage Report</h1>
			<p class="text-muted-foreground">Track verification and test coverage across your project</p>
		</div>
		<Button variant="outline" onclick={loadCoverage} disabled={loading}>
			<RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
			Refresh
		</Button>
	</div>

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
	{:else if coverage}
		<!-- Overall Health -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<TrendingUp class="h-5 w-5" />
					Overall Project Health
				</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="flex items-center gap-8">
					<div class="relative h-32 w-32">
						<svg class="h-32 w-32 -rotate-90" viewBox="0 0 100 100">
							<circle
								cx="50"
								cy="50"
								r="40"
								fill="none"
								stroke="currentColor"
								stroke-width="12"
								class="text-muted"
							/>
							<circle
								cx="50"
								cy="50"
								r="40"
								fill="none"
								stroke="currentColor"
								stroke-width="12"
								stroke-linecap="round"
								stroke-dasharray="{overallHealth() * 2.51} 251"
								class="{healthStatus().color}"
							/>
						</svg>
						<div class="absolute inset-0 flex flex-col items-center justify-center">
							<span class="text-3xl font-bold">{overallHealth().toFixed(0)}%</span>
						</div>
					</div>
					<div class="space-y-2">
						<div class="flex items-center gap-2">
							<div class="h-3 w-3 rounded-full {healthStatus().bg}"></div>
							<span class="text-lg font-medium">{healthStatus().label}</span>
						</div>
						<p class="max-w-md text-sm text-muted-foreground">
							{#if overallHealth() >= 80}
								Your project has excellent coverage across all metrics. Keep up the good work!
							{:else if overallHealth() >= 50}
								Some areas need attention. Review the metrics below to identify gaps.
							{:else}
								Critical coverage gaps detected. Prioritize verification and testing activities.
							{/if}
						</p>
					</div>
				</div>
			</CardContent>
		</Card>

		<!-- Coverage Metrics -->
		<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
			<CoverageCard
				title="Requirements Verified"
				stats={coverage.requirements_verified}
				icon={FileCheck}
				colorClass="text-blue-500"
				gapWarning="requirements need verification"
			/>
			<CoverageCard
				title="Requirements Satisfied"
				stats={coverage.requirements_satisfied}
				icon={TestTube}
				colorClass="text-green-500"
				gapWarning="requirements need design satisfaction"
			/>
			<CoverageCard
				title="Risks Mitigated"
				stats={coverage.risks_mitigated}
				icon={Shield}
				colorClass="text-red-500"
				gapWarning="risks need mitigation"
			/>
			<CoverageCard
				title="Risks Verified"
				stats={coverage.risks_verified}
				icon={AlertTriangle}
				colorClass="text-orange-500"
				gapWarning="risk mitigations need verification"
			/>
			<CoverageCard
				title="Tests Linked"
				stats={coverage.tests_linked}
				icon={PlayCircle}
				colorClass="text-purple-500"
				gapWarning="tests not linked to requirements"
			/>
			<CoverageCard
				title="Components with Suppliers"
				stats={coverage.components_with_suppliers}
				icon={TrendingUp}
				colorClass="text-cyan-500"
				gapWarning="components missing supplier info"
			/>
		</div>

		<!-- Maturity Mismatches -->
		{#if mismatches.length > 0}
			<Card class="border-orange-500/50">
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						<AlertTriangle class="h-5 w-5 text-orange-500" />
						Maturity Mismatches
						<Badge variant="secondary">{mismatches.length}</Badge>
					</CardTitle>
				</CardHeader>
				<CardContent>
					<p class="mb-4 text-sm text-muted-foreground">
						These entities link to targets with a lower maturity level. For example, an approved requirement linking to a draft test.
					</p>
					<div class="max-h-80 space-y-2 overflow-auto">
						{#each mismatches as mismatch (mismatch.source_id + mismatch.target_id + mismatch.link_type)}
							<div class="flex items-center gap-2 rounded-lg border p-3 text-sm">
								<button
									class="min-w-0 flex-1 text-left hover:underline"
									onclick={() => goto(getEntityRoute(mismatch.source_id.split('-')[0], mismatch.source_id))}
								>
									<div class="flex items-center gap-2">
										<Badge variant="outline" class="shrink-0 font-mono text-xs">
											{mismatch.source_id.split('-')[0]}
										</Badge>
										<span class="truncate font-medium">{mismatch.source_title}</span>
										<Badge class="shrink-0 text-xs bg-green-500/20 text-green-400">
											{mismatch.source_status}
										</Badge>
									</div>
								</button>
								<div class="flex shrink-0 items-center gap-1 text-xs text-muted-foreground">
									<ArrowRight class="h-3 w-3" />
									<span>{mismatch.link_type}</span>
									<ArrowRight class="h-3 w-3" />
								</div>
								<button
									class="min-w-0 flex-1 text-left hover:underline"
									onclick={() => goto(getEntityRoute(mismatch.target_id.split('-')[0], mismatch.target_id))}
								>
									<div class="flex items-center gap-2">
										<Badge variant="outline" class="shrink-0 font-mono text-xs">
											{mismatch.target_id.split('-')[0]}
										</Badge>
										<span class="truncate font-medium">{mismatch.target_title}</span>
										<Badge class="shrink-0 text-xs bg-orange-500/20 text-orange-400">
											{mismatch.target_status}
										</Badge>
									</div>
								</button>
							</div>
						{/each}
					</div>
				</CardContent>
			</Card>
		{/if}
	{:else}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">No coverage data available. Open a project first.</p>
			</CardContent>
		</Card>
	{/if}
</div>
