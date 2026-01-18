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
		ClipboardCheck,
		User,
		Calendar,
		Tag,
		CheckCircle2,
		XCircle,
		AlertTriangle,
		FileText,
		ListChecks,
		History
	} from 'lucide-svelte';

	const id = $derived($page.params.id);

	let entity = $state<EntityData | null>(null);
	let linksFrom = $state<LinkInfo[]>([]);
	let linksTo = $state<LinkInfo[]>([]);
	let loading = $state(true);
	let linksLoading = $state(true);
	let error = $state<string | null>(null);

	// Track if we've loaded for this ID to prevent double-loads
	let loadedId = $state<string | null>(null);

	// Type-safe data access
	const data = $derived(entity?.data ?? {});
	const testId = $derived((data.test_id as string) ?? null);
	const verdict = $derived((data.verdict as string) ?? 'incomplete');
	const summary = $derived((data.summary as string) ?? '');
	const executionDate = $derived((data.execution_date as string) ?? null);
	const tester = $derived((data.tester as string) ?? null);
	const revision = $derived((data.revision as number) ?? 1);

	interface StepResult {
		step: string;
		result: string;
		notes?: string;
	}
	const stepResults = $derived((data.step_results as StepResult[]) ?? []);

	interface Deviation {
		description: string;
		impact?: string;
	}
	const deviations = $derived((data.deviations as Deviation[]) ?? []);

	interface Failure {
		description: string;
		root_cause?: string;
	}
	const failures = $derived((data.failures as Failure[]) ?? []);

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
			console.error('Failed to load result:', e);
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

	function getVerdictVariant(verdict: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			pass: 'default',
			fail: 'destructive',
			conditional: 'secondary',
			incomplete: 'outline',
			not_applicable: 'outline'
		};
		return variants[verdict.toLowerCase()] ?? 'outline';
	}

	function formatVerdict(verdict: string): string {
		const verdicts: Record<string, string> = {
			pass: 'Pass',
			fail: 'Fail',
			conditional: 'Conditional',
			incomplete: 'Incomplete',
			not_applicable: 'N/A'
		};
		return verdicts[verdict.toLowerCase()] ?? verdict;
	}

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
			subtitle="Test Result"
			backHref="/verification/results"
			backLabel="Results"
			onEdit={() => goto(`/verification/results/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Summary -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<FileText class="h-5 w-5" />
							Summary
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p class="whitespace-pre-wrap">{summary || 'No summary provided.'}</p>
					</CardContent>
				</Card>

				<!-- Verdict -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							{#if verdict === 'pass'}
								<CheckCircle2 class="h-5 w-5 text-green-500" />
							{:else if verdict === 'fail'}
								<XCircle class="h-5 w-5 text-destructive" />
							{:else}
								<AlertTriangle class="h-5 w-5 text-orange-500" />
							{/if}
							Verdict
						</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="flex items-center gap-4">
							<Badge variant={getVerdictVariant(verdict)} class="text-lg px-4 py-1">
								{formatVerdict(verdict)}
							</Badge>
							{#if executionDate}
								<span class="text-sm text-muted-foreground">
									Executed on {formatDate(executionDate)}
								</span>
							{/if}
							{#if tester}
								<span class="text-sm text-muted-foreground">
									by {tester}
								</span>
							{/if}
						</div>
					</CardContent>
				</Card>

				<!-- Step Results -->
				{#if stepResults.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<ListChecks class="h-5 w-5" />
								Step Results ({stepResults.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-3">
								{#each stepResults as step, i}
									<div class="rounded-lg border p-3">
										<div class="flex items-center justify-between">
											<span class="text-sm font-medium">Step {i + 1}: {step.step}</span>
											<Badge variant={step.result === 'pass' ? 'default' : step.result === 'fail' ? 'destructive' : 'outline'}>
												{step.result}
											</Badge>
										</div>
										{#if step.notes}
											<p class="mt-2 text-sm text-muted-foreground">{step.notes}</p>
										{/if}
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Deviations -->
				{#if deviations.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<AlertTriangle class="h-5 w-5 text-orange-500" />
								Deviations ({deviations.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-3">
								{#each deviations as deviation}
									<div class="rounded-lg border border-orange-500/20 bg-orange-500/5 p-3">
										<p>{deviation.description}</p>
										{#if deviation.impact}
											<p class="mt-2 text-sm text-muted-foreground">Impact: {deviation.impact}</p>
										{/if}
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Failures -->
				{#if failures.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<XCircle class="h-5 w-5 text-destructive" />
								Failures ({failures.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-3">
								{#each failures as failure}
									<div class="rounded-lg border border-destructive/20 bg-destructive/5 p-3">
										<p>{failure.description}</p>
										{#if failure.root_cause}
											<p class="mt-2 text-sm text-muted-foreground">Root Cause: {failure.root_cause}</p>
										{/if}
									</div>
								{/each}
							</div>
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
				<!-- Properties -->
				<Card>
					<CardHeader>
						<CardTitle>Properties</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Verdict</span>
							<Badge variant={getVerdictVariant(verdict)}>
								{formatVerdict(verdict)}
							</Badge>
						</div>
						{#if testId}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Test ID</span>
								<button
									class="font-mono text-xs text-primary hover:underline"
									onclick={() => goto(`/verification/tests/${testId}`)}
								>
									{testId}
								</button>
							</div>
						{/if}
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Revision</span>
							<span class="text-sm font-medium">{revision}</span>
						</div>
					</CardContent>
				</Card>

				<!-- Execution Info -->
				<Card>
					<CardHeader>
						<CardTitle>Execution</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if executionDate}
							<div class="flex items-center gap-2">
								<Calendar class="h-4 w-4 text-muted-foreground" />
								<span class="text-sm text-muted-foreground">Date</span>
								<span class="ml-auto text-sm font-medium">{formatDate(executionDate)}</span>
							</div>
						{/if}
						{#if tester}
							<div class="flex items-center gap-2">
								<User class="h-4 w-4 text-muted-foreground" />
								<span class="text-sm text-muted-foreground">Tester</span>
								<span class="ml-auto text-sm font-medium">{tester}</span>
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
				<p class="text-muted-foreground">Result not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
