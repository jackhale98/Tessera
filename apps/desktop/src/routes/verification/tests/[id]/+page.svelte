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
		FlaskConical,
		User,
		Calendar,
		Tag,
		CheckCircle2,
		Layers,
		FileText,
		Wrench,
		ClipboardList,
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
	const testType = $derived((data.type as string) ?? 'verification');
	const testLevel = $derived((data.test_level as string) ?? null);
	const testMethod = $derived((data.test_method as string) ?? null);
	const description = $derived((data.description as string) ?? '');
	const objective = $derived((data.objective as string) ?? null);
	const preconditions = $derived((data.preconditions as string[]) ?? []);
	const acceptanceCriteria = $derived((data.acceptance_criteria as string[]) ?? []);
	const revision = $derived((data.revision as number) ?? 1);
	const estimatedDuration = $derived((data.estimated_duration as string) ?? null);
	const priority = $derived((data.priority as string) ?? null);

	interface ProcedureStep {
		step: number;
		action: string;
		expected: string;
		acceptance?: string;
	}
	const procedure = $derived((data.procedure as ProcedureStep[]) ?? []);

	interface EquipmentItem {
		name: string;
		specification?: string;
		calibration_required?: boolean;
	}
	const equipment = $derived((data.equipment as EquipmentItem[]) ?? []);

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
			console.error('Failed to load test:', e);
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

	function formatTestType(type: string): string {
		return type === 'verification' ? 'Verification Test' : 'Validation Test';
	}

	function formatTestMethod(method: string | null): string {
		if (!method) return 'Not specified';
		const methods: Record<string, string> = {
			inspection: 'Inspection',
			analysis: 'Analysis',
			demonstration: 'Demonstration',
			test: 'Test'
		};
		return methods[method.toLowerCase()] ?? method;
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
			subtitle={formatTestType(testType)}
			backHref="/verification/tests"
			backLabel="Tests"
			onEdit={() => goto(`/verification/tests/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Objective -->
				{#if objective}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<FlaskConical class="h-5 w-5" />
								Objective
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap">{objective}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Description -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<FileText class="h-5 w-5" />
							Description
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p class="whitespace-pre-wrap">{description || 'No description specified.'}</p>
					</CardContent>
				</Card>

				<!-- Preconditions -->
				{#if preconditions.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<CheckCircle2 class="h-5 w-5" />
								Preconditions
							</CardTitle>
						</CardHeader>
						<CardContent>
							<ul class="list-inside list-disc space-y-2">
								{#each preconditions as condition}
									<li class="text-muted-foreground">{condition}</li>
								{/each}
							</ul>
						</CardContent>
					</Card>
				{/if}

				<!-- Procedure -->
				{#if procedure.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<ClipboardList class="h-5 w-5" />
								Procedure ({procedure.length} steps)
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-4">
								{#each procedure as step}
									<div class="rounded-lg border p-4">
										<div class="flex items-start gap-4">
											<span class="flex h-6 w-6 items-center justify-center rounded-full bg-muted text-sm font-medium">
												{step.step}
											</span>
											<div class="flex-1 space-y-2">
												<div>
													<h4 class="text-sm font-medium">Action</h4>
													<p class="whitespace-pre-wrap text-sm text-muted-foreground">{step.action}</p>
												</div>
												<div>
													<h4 class="text-sm font-medium">Expected</h4>
													<p class="whitespace-pre-wrap text-sm text-muted-foreground">{step.expected}</p>
												</div>
												{#if step.acceptance}
													<div>
														<h4 class="text-sm font-medium">Acceptance</h4>
														<p class="whitespace-pre-wrap text-sm text-muted-foreground">{step.acceptance}</p>
													</div>
												{/if}
											</div>
										</div>
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Acceptance Criteria -->
				{#if acceptanceCriteria.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<CheckCircle2 class="h-5 w-5" />
								Acceptance Criteria
							</CardTitle>
						</CardHeader>
						<CardContent>
							<ul class="list-inside list-disc space-y-2">
								{#each acceptanceCriteria as criterion}
									<li class="text-muted-foreground">{criterion}</li>
								{/each}
							</ul>
						</CardContent>
					</Card>
				{/if}

				<!-- Equipment -->
				{#if equipment.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Wrench class="h-5 w-5" />
								Equipment ({equipment.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-3">
								{#each equipment as item}
									<div class="flex items-center justify-between rounded-lg border p-3">
										<div>
											<span class="font-medium">{item.name}</span>
											{#if item.specification}
												<p class="text-sm text-muted-foreground">{item.specification}</p>
											{/if}
										</div>
										{#if item.calibration_required}
											<Badge variant="secondary">Cal. Required</Badge>
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
							<span class="text-sm text-muted-foreground">Type</span>
							<Badge variant={testType === 'verification' ? 'secondary' : 'default'}>
								{testType === 'verification' ? 'Verification' : 'Validation'}
							</Badge>
						</div>
						{#if testLevel}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Level</span>
								<Badge variant="outline">
									<Layers class="mr-1 h-3 w-3" />
									{testLevel}
								</Badge>
							</div>
						{/if}
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Method</span>
							<Badge variant="outline">{formatTestMethod(testMethod)}</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						{#if priority}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Priority</span>
								<Badge variant={priority === 'critical' ? 'destructive' : priority === 'high' ? 'default' : 'outline'} class="capitalize">
									{priority}
								</Badge>
							</div>
						{/if}
						{#if estimatedDuration}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Est. Duration</span>
								<span class="text-sm font-medium">{estimatedDuration}</span>
							</div>
						{/if}
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Revision</span>
							<span class="text-sm font-medium">{revision}</span>
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
				<p class="text-muted-foreground">Test not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
