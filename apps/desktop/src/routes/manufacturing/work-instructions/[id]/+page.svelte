<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { entities, traceability } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import EntityHistory from '$lib/components/EntityHistory.svelte';
	import {
		ClipboardList,
		User,
		Calendar,
		Tag,
		FileText,
		ListOrdered,
		AlertCircle,
		Wrench,
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
	const description = $derived((data.description as string) ?? '');
	const revision = $derived((data.revision as number) ?? 1);
	const equipment = $derived((data.equipment as string[]) ?? []);
	const materials = $derived((data.materials as string[]) ?? []);
	const safetyNotes = $derived((data.safety_notes as string[]) ?? []);

	interface Step {
		number: number;
		instruction: string;
		notes?: string;
		critical?: boolean;
	}
	const steps = $derived((data.steps as Step[]) ?? []);

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
			console.error('Failed to load work instruction:', e);
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
			subtitle="Work Instruction"
			backHref="/manufacturing/work-instructions"
			backLabel="Work Instructions"
			onEdit={() => goto(`/manufacturing/work-instructions/${id}/edit`)}
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

				<!-- Safety Notes -->
				{#if safetyNotes.length > 0}
					<Card class="border-orange-500/50">
						<CardHeader>
							<CardTitle class="flex items-center gap-2 text-orange-500">
								<AlertCircle class="h-5 w-5" />
								Safety Notes
							</CardTitle>
						</CardHeader>
						<CardContent>
							<ul class="list-inside list-disc space-y-2">
								{#each safetyNotes as note}
									<li class="text-orange-600 dark:text-orange-400">{note}</li>
								{/each}
							</ul>
						</CardContent>
					</Card>
				{/if}

				<!-- Steps -->
				{#if steps.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<ListOrdered class="h-5 w-5" />
								Procedure Steps ({steps.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-4">
								{#each steps as step}
									<div class="rounded-lg border p-4 {step.critical ? 'border-orange-500/50 bg-orange-500/5' : ''}">
										<div class="flex items-start gap-4">
											<div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-sm font-bold text-primary">
												{step.number}
											</div>
											<div class="flex-1">
												<div class="flex items-center gap-2">
													<p class="font-medium">{step.instruction}</p>
													{#if step.critical}
														<Badge variant="destructive" class="text-xs">Critical</Badge>
													{/if}
												</div>
												{#if step.notes}
													<p class="mt-2 text-sm text-muted-foreground">{step.notes}</p>
												{/if}
											</div>
										</div>
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
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Revision</span>
							<span class="text-sm font-medium">{revision}</span>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Steps</span>
							<span class="text-sm font-medium">{steps.length}</span>
						</div>
					</CardContent>
				</Card>

				<!-- Equipment -->
				{#if equipment.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Wrench class="h-4 w-4" />
								Equipment
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="flex flex-wrap gap-2">
								{#each equipment as item}
									<Badge variant="outline">{item}</Badge>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Materials -->
				{#if materials.length > 0}
					<Card>
						<CardHeader>
							<CardTitle>Materials</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="flex flex-wrap gap-2">
								{#each materials as material}
									<Badge variant="secondary">{material}</Badge>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

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
				<p class="text-muted-foreground">Work instruction not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
