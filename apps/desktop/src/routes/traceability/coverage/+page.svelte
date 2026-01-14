<script lang="ts">
	import { onMount } from 'svelte';
	import { Card, CardContent, CardHeader, CardTitle, Button } from '$lib/components/ui';
	import { CoverageCard } from '$lib/components/traceability';
	import { traceability } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import type { CoverageReport } from '$lib/api/types';
	import {
		FileCheck,
		TestTube,
		Shield,
		PlayCircle,
		RefreshCw,
		TrendingUp,
		AlertTriangle
	} from 'lucide-svelte';

	let coverage = $state<CoverageReport | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function loadCoverage() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			coverage = await traceability.getCoverage();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	const overallHealth = $derived(() => {
		if (!coverage) return 0;
		const scores = [
			coverage.requirements_verified.percentage,
			coverage.requirements_tested.percentage,
			coverage.risks_mitigated.percentage,
			coverage.tests_executed.percentage
		];
		return scores.reduce((a, b) => a + b, 0) / scores.length;
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
		<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
			<CoverageCard
				title="Requirements Verified"
				stats={coverage.requirements_verified}
				icon={FileCheck}
				colorClass="text-blue-500"
			/>
			<CoverageCard
				title="Requirements Tested"
				stats={coverage.requirements_tested}
				icon={TestTube}
				colorClass="text-green-500"
			/>
			<CoverageCard
				title="Risks Mitigated"
				stats={coverage.risks_mitigated}
				icon={Shield}
				colorClass="text-red-500"
			/>
			<CoverageCard
				title="Tests Executed"
				stats={coverage.tests_executed}
				icon={PlayCircle}
				colorClass="text-purple-500"
			/>
		</div>

		<!-- Detailed Breakdown -->
		<div class="grid gap-6 lg:grid-cols-2">
			<!-- Requirements Coverage -->
			<Card>
				<CardHeader>
					<CardTitle>Requirements Coverage Details</CardTitle>
				</CardHeader>
				<CardContent class="space-y-4">
					<div>
						<div class="mb-2 flex items-center justify-between text-sm">
							<span>Verified Requirements</span>
							<span class="font-medium">
								{coverage.requirements_verified.covered} of {coverage.requirements_verified.total}
							</span>
						</div>
						<div class="h-3 w-full overflow-hidden rounded-full bg-muted">
							<div
								class="h-full bg-blue-500 transition-all"
								style="width: {coverage.requirements_verified.percentage}%"
							></div>
						</div>
					</div>
					<div>
						<div class="mb-2 flex items-center justify-between text-sm">
							<span>Tested Requirements</span>
							<span class="font-medium">
								{coverage.requirements_tested.covered} of {coverage.requirements_tested.total}
							</span>
						</div>
						<div class="h-3 w-full overflow-hidden rounded-full bg-muted">
							<div
								class="h-full bg-green-500 transition-all"
								style="width: {coverage.requirements_tested.percentage}%"
							></div>
						</div>
					</div>
					{#if coverage.requirements_verified.total - coverage.requirements_verified.covered > 0}
						<div class="flex items-center gap-2 rounded-lg bg-yellow-500/10 p-3 text-sm text-yellow-600 dark:text-yellow-400">
							<AlertTriangle class="h-4 w-4" />
							{coverage.requirements_verified.total - coverage.requirements_verified.covered} requirements need verification
						</div>
					{/if}
				</CardContent>
			</Card>

			<!-- Risk & Test Coverage -->
			<Card>
				<CardHeader>
					<CardTitle>Risk & Test Coverage Details</CardTitle>
				</CardHeader>
				<CardContent class="space-y-4">
					<div>
						<div class="mb-2 flex items-center justify-between text-sm">
							<span>Mitigated Risks</span>
							<span class="font-medium">
								{coverage.risks_mitigated.covered} of {coverage.risks_mitigated.total}
							</span>
						</div>
						<div class="h-3 w-full overflow-hidden rounded-full bg-muted">
							<div
								class="h-full bg-red-500 transition-all"
								style="width: {coverage.risks_mitigated.percentage}%"
							></div>
						</div>
					</div>
					<div>
						<div class="mb-2 flex items-center justify-between text-sm">
							<span>Executed Tests</span>
							<span class="font-medium">
								{coverage.tests_executed.covered} of {coverage.tests_executed.total}
							</span>
						</div>
						<div class="h-3 w-full overflow-hidden rounded-full bg-muted">
							<div
								class="h-full bg-purple-500 transition-all"
								style="width: {coverage.tests_executed.percentage}%"
							></div>
						</div>
					</div>
					{#if coverage.risks_mitigated.total - coverage.risks_mitigated.covered > 0}
						<div class="flex items-center gap-2 rounded-lg bg-red-500/10 p-3 text-sm text-red-600 dark:text-red-400">
							<AlertTriangle class="h-4 w-4" />
							{coverage.risks_mitigated.total - coverage.risks_mitigated.covered} risks need mitigation
						</div>
					{/if}
				</CardContent>
			</Card>
		</div>
	{:else}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">No coverage data available. Open a project first.</p>
			</CardContent>
		</Card>
	{/if}
</div>
