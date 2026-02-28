<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { isProjectOpen, entityCounts, projectName, totalEntities } from '$lib/stores/project.js';
	import { openProject, initProject } from '$lib/stores/project.js';
	import { traceability } from '$lib/api';
	import type { CoverageReport } from '$lib/api/types';
	import { open } from '@tauri-apps/plugin-dialog';
	import {
		FileText,
		AlertTriangle,
		FlaskConical,
		Box,
		Factory,
		Shield,
		FolderOpen,
		FolderPlus,
		ArrowRight,
		TrendingUp
	} from 'lucide-svelte';

	interface StatCard {
		label: string;
		value: number;
		icon: typeof FileText;
		href: string;
		color: string;
	}

	let stats: StatCard[] = $derived.by(() => {
		const counts = $entityCounts;
		if (!counts) return [];

		return [
			{
				label: 'Requirements',
				value: counts.requirements,
				icon: FileText,
				href: '/requirements',
				color: 'text-blue-500'
			},
			{
				label: 'Risks',
				value: counts.risks,
				icon: AlertTriangle,
				href: '/risks',
				color: 'text-amber-500'
			},
			{
				label: 'Tests',
				value: counts.tests + counts.results,
				icon: FlaskConical,
				href: '/verification/tests',
				color: 'text-green-500'
			},
			{
				label: 'Components',
				value: counts.components + counts.assemblies,
				icon: Box,
				href: '/assemblies',
				color: 'text-purple-500'
			},
			{
				label: 'Manufacturing',
				value: counts.processes + counts.lots + counts.work_instructions,
				icon: Factory,
				href: '/manufacturing/processes',
				color: 'text-orange-500'
			},
			{
				label: 'Quality',
				value: counts.ncrs + counts.capas,
				icon: Shield,
				href: '/quality/ncrs',
				color: 'text-red-500'
			}
		];
	});

	async function handleOpenProject() {
		const selected = await open({
			directory: true,
			multiple: false,
			title: 'Open Tessera Project'
		});

		if (selected && typeof selected === 'string') {
			await openProject(selected);
		}
	}

	async function handleInitProject() {
		const selected = await open({
			directory: true,
			multiple: false,
			title: 'Select Directory for New Project'
		});

		if (selected && typeof selected === 'string') {
			await initProject(selected);
		}
	}

	let coverage = $state<CoverageReport | null>(null);
	let coverageLoading = $state(false);

	async function loadCoverage() {
		if (!$isProjectOpen) return;
		coverageLoading = true;
		try {
			coverage = await traceability.getCoverage();
		} catch {
			// Silently fail on dashboard — coverage is supplementary
		} finally {
			coverageLoading = false;
		}
	}

	const healthScore = $derived(coverage?.health_score ?? 0);
	const healthStatus = $derived.by(() => {
		if (healthScore >= 80) return { label: 'Healthy', color: 'text-green-500', bg: 'bg-green-500' };
		if (healthScore >= 50) return { label: 'Needs Attention', color: 'text-yellow-500', bg: 'bg-yellow-500' };
		return { label: 'Critical', color: 'text-red-500', bg: 'bg-red-500' };
	});

	onMount(() => {
		if ($isProjectOpen) loadCoverage();
	});

	$effect(() => {
		if ($isProjectOpen) loadCoverage();
	});
</script>

{#if !$isProjectOpen}
	<!-- Welcome screen when no project is open -->
	<div class="flex h-full items-center justify-center">
		<div class="max-w-lg text-center">
			<div class="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-2xl bg-primary/10">
				<span class="text-4xl font-bold text-primary">T</span>
			</div>
			<h1 class="mb-2 text-3xl font-bold tracking-tight">Welcome to Tessera</h1>
			<p class="mb-8 text-muted-foreground">
				Tessera helps you manage engineering artifacts including requirements, risks,
				tests, BOMs, and more with full traceability.
			</p>
			<div class="flex flex-col gap-3 sm:flex-row sm:justify-center">
				<Button variant="outline" size="lg" onclick={handleOpenProject}>
					<FolderOpen class="mr-2 h-5 w-5" />
					Open Existing Project
				</Button>
				<Button size="lg" onclick={handleInitProject}>
					<FolderPlus class="mr-2 h-5 w-5" />
					Create New Project
				</Button>
			</div>
		</div>
	</div>
{:else}
	<!-- Dashboard when project is open -->
	<div class="space-y-6">
		<!-- Header -->
		<div>
			<h1 class="text-2xl font-bold tracking-tight">{$projectName}</h1>
			<p class="text-muted-foreground">
				{$totalEntities} total entities across all categories
			</p>
		</div>

		<!-- Stats grid -->
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each stats as stat}
				<a href={stat.href} class="block">
					<Card class="transition-colors hover:bg-accent/50">
						<CardHeader class="flex flex-row items-center justify-between pb-2">
							<CardTitle class="text-sm font-medium text-muted-foreground">
								{stat.label}
							</CardTitle>
							<stat.icon class="h-4 w-4 {stat.color}" />
						</CardHeader>
						<CardContent>
							<div class="flex items-center justify-between">
								<div class="text-2xl font-bold">{stat.value}</div>
								<ArrowRight class="h-4 w-4 text-muted-foreground" />
							</div>
						</CardContent>
					</Card>
				</a>
			{/each}
		</div>

		<!-- Quick actions -->
		<Card>
			<CardHeader>
				<CardTitle>Quick Actions</CardTitle>
				<CardDescription>Common tasks to get you started</CardDescription>
			</CardHeader>
			<CardContent>
				<div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-4">
					<Button variant="outline" class="justify-start" onclick={() => goto('/requirements/new')}>
						<FileText class="mr-2 h-4 w-4" />
						New Requirement
					</Button>
					<Button variant="outline" class="justify-start" onclick={() => goto('/risks/new')}>
						<AlertTriangle class="mr-2 h-4 w-4" />
						New Risk
					</Button>
					<Button variant="outline" class="justify-start" onclick={() => goto('/verification/tests/new')}>
						<FlaskConical class="mr-2 h-4 w-4" />
						New Test
					</Button>
					<Button variant="outline" class="justify-start" onclick={() => goto('/components/new')}>
						<Box class="mr-2 h-4 w-4" />
						New Component
					</Button>
				</div>
			</CardContent>
		</Card>

		<!-- Project Health -->
		<Card>
			<CardHeader class="flex flex-row items-center justify-between pb-2">
				<div>
					<CardTitle class="flex items-center gap-2">
						<TrendingUp class="h-5 w-5" />
						Project Health
					</CardTitle>
					<CardDescription>Traceability coverage across your project</CardDescription>
				</div>
				<Button variant="ghost" size="sm" onclick={() => goto('/traceability/coverage')}>
					Details
					<ArrowRight class="ml-1 h-3 w-3" />
				</Button>
			</CardHeader>
			<CardContent>
				{#if coverageLoading}
					<div class="flex h-24 items-center justify-center">
						<div class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
					</div>
				{:else if coverage}
					<div class="flex items-center gap-6">
						<!-- Health score circle -->
						<div class="relative h-20 w-20 shrink-0">
							<svg class="h-20 w-20 -rotate-90" viewBox="0 0 100 100">
								<circle cx="50" cy="50" r="40" fill="none" stroke="currentColor" stroke-width="12" class="text-muted" />
								<circle cx="50" cy="50" r="40" fill="none" stroke="currentColor" stroke-width="12" stroke-linecap="round"
									stroke-dasharray="{healthScore * 2.51} 251" class="{healthStatus.color}" />
							</svg>
							<div class="absolute inset-0 flex flex-col items-center justify-center">
								<span class="text-lg font-bold">{healthScore.toFixed(0)}%</span>
							</div>
						</div>

						<!-- Metric bars -->
						<div class="flex-1 space-y-2">
							<div>
								<div class="flex items-center justify-between text-xs text-muted-foreground mb-0.5">
									<span>Requirements Verified</span>
									<span>{coverage.requirements_verified.covered}/{coverage.requirements_verified.total}</span>
								</div>
								<div class="h-1.5 w-full overflow-hidden rounded-full bg-muted">
									<div class="h-full bg-blue-500 transition-all" style="width: {coverage.requirements_verified.percentage}%"></div>
								</div>
							</div>
							<div>
								<div class="flex items-center justify-between text-xs text-muted-foreground mb-0.5">
									<span>Risks Mitigated</span>
									<span>{coverage.risks_mitigated.covered}/{coverage.risks_mitigated.total}</span>
								</div>
								<div class="h-1.5 w-full overflow-hidden rounded-full bg-muted">
									<div class="h-full bg-red-500 transition-all" style="width: {coverage.risks_mitigated.percentage}%"></div>
								</div>
							</div>
							<div>
								<div class="flex items-center justify-between text-xs text-muted-foreground mb-0.5">
									<span>Tests Linked</span>
									<span>{coverage.tests_linked.covered}/{coverage.tests_linked.total}</span>
								</div>
								<div class="h-1.5 w-full overflow-hidden rounded-full bg-muted">
									<div class="h-full bg-purple-500 transition-all" style="width: {coverage.tests_linked.percentage}%"></div>
								</div>
							</div>
						</div>

						<!-- Health status -->
						<div class="text-center shrink-0">
							<div class="h-3 w-3 rounded-full {healthStatus.bg} mx-auto mb-1"></div>
							<span class="text-xs font-medium {healthStatus.color}">{healthStatus.label}</span>
						</div>
					</div>
				{:else}
					<div class="flex h-24 items-center justify-center text-muted-foreground">
						<p class="text-sm">Coverage data not available</p>
					</div>
				{/if}
			</CardContent>
		</Card>
	</div>
{/if}
