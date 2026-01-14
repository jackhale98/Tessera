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
		AlertTriangle,
		User,
		Calendar,
		Tag,
		FileText,
		Scale,
		CheckCircle2
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
	const deviationType = $derived((data.deviation_type as string) ?? 'process');
	const description = $derived((data.description as string) ?? '');
	const justification = $derived((data.justification as string) ?? null);
	const impactAnalysis = $derived((data.impact_analysis as string) ?? null);
	const approver = $derived((data.approver as string) ?? null);
	const approvalDate = $derived((data.approval_date as string) ?? null);
	const expirationDate = $derived((data.expiration_date as string) ?? null);
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
			console.error('Failed to load deviation:', e);
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

	function formatDeviationType(type: string): string {
		const types: Record<string, string> = {
			process: 'Process Deviation',
			product: 'Product Deviation',
			material: 'Material Deviation',
			equipment: 'Equipment Deviation'
		};
		return types[type.toLowerCase()] ?? type;
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
			subtitle={formatDeviationType(deviationType)}
			backHref="/manufacturing/deviations"
			backLabel="Deviations"
			onEdit={() => goto(`/manufacturing/deviations/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Description -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<AlertTriangle class="h-5 w-5" />
							Description
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p class="whitespace-pre-wrap">{description || 'No description specified.'}</p>
					</CardContent>
				</Card>

				<!-- Justification -->
				{#if justification}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Scale class="h-5 w-5" />
								Justification
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap text-muted-foreground">{justification}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Impact Analysis -->
				{#if impactAnalysis}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<FileText class="h-5 w-5" />
								Impact Analysis
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap text-muted-foreground">{impactAnalysis}</p>
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
							<Badge variant="outline">{formatDeviationType(deviationType)}</Badge>
						</div>
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

				<!-- Approval -->
				{#if approver || approvalDate}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<CheckCircle2 class="h-4 w-4" />
								Approval
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if approver}
								<div class="flex items-center gap-2">
									<User class="h-4 w-4 text-muted-foreground" />
									<span class="text-sm text-muted-foreground">Approver</span>
									<span class="ml-auto text-sm font-medium">{approver}</span>
								</div>
							{/if}
							{#if approvalDate}
								<div class="flex items-center gap-2">
									<Calendar class="h-4 w-4 text-muted-foreground" />
									<span class="text-sm text-muted-foreground">Approved</span>
									<span class="ml-auto text-sm font-medium">{formatDate(approvalDate)}</span>
								</div>
							{/if}
							{#if expirationDate}
								<div class="flex items-center gap-2">
									<Calendar class="h-4 w-4 text-muted-foreground" />
									<span class="text-sm text-muted-foreground">Expires</span>
									<span class="ml-auto text-sm font-medium">{formatDate(expirationDate)}</span>
								</div>
							{/if}
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
				<p class="text-muted-foreground">Deviation not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
