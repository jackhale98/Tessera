<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge, PriorityBadge } from '$lib/components/common';
	import EntityHistory from '$lib/components/EntityHistory.svelte';
	import { entities, traceability } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import {
		FileText,
		User,
		Calendar,
		Tag,
		CheckCircle2,
		FileQuestion,
		Layers,
		BookOpen,
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
	const reqType = $derived((data.req_type as string) ?? 'input');
	const level = $derived((data.level as string) ?? 'system');
	const text = $derived((data.text as string) ?? '');
	const rationale = $derived((data.rationale as string) ?? null);
	const priority = $derived((data.priority as string) ?? 'medium');
	const acceptanceCriteria = $derived((data.acceptance_criteria as string[]) ?? []);
	const source = $derived(data.source as { document?: string; revision?: string; section?: string; date?: string } | null);
	const category = $derived((data.category as string) ?? null);
	const revision = $derived((data.revision as number) ?? 1);

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
			console.error('Failed to load requirement:', e);
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

	function formatLevel(level: string): string {
		return level.charAt(0).toUpperCase() + level.slice(1);
	}

	function formatReqType(type: string): string {
		return type === 'input' ? 'Input Requirement' : 'Output Requirement';
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
			subtitle={formatReqType(reqType)}
			backHref="/requirements"
			backLabel="Requirements"
			onEdit={() => goto(`/requirements/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Requirement Text -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<FileText class="h-5 w-5" />
							Requirement Text
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p class="whitespace-pre-wrap">{text || 'No requirement text specified.'}</p>
					</CardContent>
				</Card>

				<!-- Rationale -->
				{#if rationale}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<FileQuestion class="h-5 w-5" />
								Rationale
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap text-muted-foreground">{rationale}</p>
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

				<!-- Source -->
				{#if source}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<BookOpen class="h-5 w-5" />
								Source
							</CardTitle>
						</CardHeader>
						<CardContent>
							<dl class="grid gap-2 sm:grid-cols-2">
								{#if source.document}
									<div>
										<dt class="text-sm text-muted-foreground">Document</dt>
										<dd class="font-medium">{source.document}</dd>
									</div>
								{/if}
								{#if source.revision}
									<div>
										<dt class="text-sm text-muted-foreground">Revision</dt>
										<dd class="font-medium">{source.revision}</dd>
									</div>
								{/if}
								{#if source.section}
									<div>
										<dt class="text-sm text-muted-foreground">Section</dt>
										<dd class="font-medium">{source.section}</dd>
									</div>
								{/if}
								{#if source.date}
									<div>
										<dt class="text-sm text-muted-foreground">Date</dt>
										<dd class="font-medium">{formatDate(source.date)}</dd>
									</div>
								{/if}
							</dl>
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
							<Badge variant={reqType === 'input' ? 'secondary' : 'default'}>
								{reqType === 'input' ? 'Input' : 'Output'}
							</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Level</span>
							<Badge variant="outline">
								<Layers class="mr-1 h-3 w-3" />
								{formatLevel(level)}
							</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Priority</span>
							<PriorityBadge {priority} />
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						{#if category}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Category</span>
								<span class="text-sm font-medium">{category}</span>
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
				<p class="text-muted-foreground">Requirement not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
