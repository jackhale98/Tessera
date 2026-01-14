<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { entities, traceability } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import {
		Shield,
		User,
		Calendar,
		Tag,
		FileText,
		Search,
		ListChecks,
		CheckCircle2,
		Clock
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
	const capaType = $derived((data.capa_type as string) ?? 'corrective');
	const description = $derived((data.description as string) ?? '');
	const rootCauseAnalysis = $derived((data.root_cause_analysis as string) ?? null);
	const effectiveness = $derived((data.effectiveness as string) ?? null);
	const effectivenessDate = $derived((data.effectiveness_date as string) ?? null);
	const revision = $derived((data.revision as number) ?? 1);

	interface Action {
		description: string;
		owner?: string;
		due_date?: string;
		status?: string;
		completion_date?: string;
	}
	const actions = $derived((data.actions as Action[]) ?? []);

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
			console.error('Failed to load CAPA:', e);
		} finally {
			loading = false;
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

	function formatCapaType(type: string): string {
		return type === 'corrective' ? 'Corrective Action' : 'Preventive Action';
	}

	function getActionStatusVariant(status: string | undefined): 'default' | 'secondary' | 'destructive' | 'outline' {
		if (!status) return 'outline';
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			open: 'outline',
			in_progress: 'secondary',
			completed: 'default',
			verified: 'default',
			overdue: 'destructive'
		};
		return variants[status.toLowerCase()] ?? 'outline';
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
			subtitle={formatCapaType(capaType)}
			backHref="/quality/capas"
			backLabel="CAPAs"
			onEdit={() => goto(`/quality/capas/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
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

				<!-- Root Cause Analysis -->
				{#if rootCauseAnalysis}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Search class="h-5 w-5" />
								Root Cause Analysis
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap text-muted-foreground">{rootCauseAnalysis}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Actions -->
				{#if actions.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<ListChecks class="h-5 w-5" />
								Actions ({actions.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-4">
								{#each actions as action, i}
									<div class="rounded-lg border p-4">
										<div class="flex items-start justify-between gap-4">
											<div class="flex-1">
												<div class="flex items-center gap-2">
													<span class="text-sm font-medium text-muted-foreground">#{i + 1}</span>
													{#if action.status}
														<Badge variant={getActionStatusVariant(action.status)} class="capitalize">
															{action.status.replace(/_/g, ' ')}
														</Badge>
													{/if}
												</div>
												<p class="mt-2">{action.description}</p>
											</div>
										</div>
										{#if action.owner || action.due_date || action.completion_date}
											<div class="mt-3 flex items-center gap-4 text-sm text-muted-foreground">
												{#if action.owner}
													<span class="flex items-center gap-1">
														<User class="h-3 w-3" />
														{action.owner}
													</span>
												{/if}
												{#if action.due_date}
													<span class="flex items-center gap-1">
														<Clock class="h-3 w-3" />
														Due: {formatDate(action.due_date)}
													</span>
												{/if}
												{#if action.completion_date}
													<span class="flex items-center gap-1">
														<CheckCircle2 class="h-3 w-3" />
														Completed: {formatDate(action.completion_date)}
													</span>
												{/if}
											</div>
										{/if}
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Effectiveness -->
				{#if effectiveness}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<CheckCircle2 class="h-5 w-5" />
								Effectiveness Review
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							<p class="whitespace-pre-wrap">{effectiveness}</p>
							{#if effectivenessDate}
								<div class="flex items-center gap-2 text-sm text-muted-foreground">
									<Calendar class="h-4 w-4" />
									Reviewed on {formatDate(effectivenessDate)}
								</div>
							{/if}
						</CardContent>
					</Card>
				{/if}

				<!-- Links -->
				<LinksSection {linksFrom} {linksTo} loading={linksLoading} />
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
							<Badge variant={capaType === 'corrective' ? 'secondary' : 'default'}>
								{capaType === 'corrective' ? 'Corrective' : 'Preventive'}
							</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Actions</span>
							<Badge variant="outline">{actions.length}</Badge>
						</div>
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
				<p class="text-muted-foreground">CAPA not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
