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
		AlertOctagon,
		User,
		Calendar,
		Tag,
		FileText,
		ClipboardCheck,
		AlertTriangle
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
	const ncrType = $derived((data.ncr_type as string) ?? 'product');
	const ncrStatus = $derived((data.ncr_status as string) ?? 'open');
	const severity = $derived((data.severity as string) ?? 'minor');
	const category = $derived((data.category as string) ?? null);
	const description = $derived((data.description as string) ?? '');
	const disposition = $derived((data.disposition as string) ?? null);
	const dispositionRationale = $derived((data.disposition_rationale as string) ?? null);
	const affectedQuantity = $derived(data.affected_quantity as number | null);
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
			console.error('Failed to load NCR:', e);
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

	function formatNcrType(type: string): string {
		const types: Record<string, string> = {
			product: 'Product',
			process: 'Process',
			supplier: 'Supplier',
			documentation: 'Documentation'
		};
		return types[type.toLowerCase()] ?? type;
	}

	function formatNcrStatus(status: string): string {
		const statuses: Record<string, string> = {
			open: 'Open',
			under_investigation: 'Under Investigation',
			pending_disposition: 'Pending Disposition',
			closed: 'Closed'
		};
		return statuses[status.toLowerCase()] ?? status;
	}

	function getNcrStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			open: 'destructive',
			under_investigation: 'secondary',
			pending_disposition: 'outline',
			closed: 'default'
		};
		return variants[status.toLowerCase()] ?? 'outline';
	}

	function getSeverityVariant(severity: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			minor: 'outline',
			major: 'secondary',
			critical: 'destructive'
		};
		return variants[severity.toLowerCase()] ?? 'outline';
	}

	function formatDisposition(disp: string): string {
		const dispositions: Record<string, string> = {
			use_as_is: 'Use As Is',
			rework: 'Rework',
			repair: 'Repair',
			scrap: 'Scrap',
			return_to_supplier: 'Return to Supplier'
		};
		return dispositions[disp.toLowerCase()] ?? disp;
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
			subtitle="Non-Conformance Report"
			backHref="/quality/ncrs"
			backLabel="NCRs"
			onEdit={() => goto(`/quality/ncrs/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Description -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<AlertOctagon class="h-5 w-5" />
							Non-Conformance Description
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p class="whitespace-pre-wrap">{description || 'No description specified.'}</p>
					</CardContent>
				</Card>

				<!-- Disposition -->
				{#if disposition}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<ClipboardCheck class="h-5 w-5" />
								Disposition
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							<div class="flex items-center gap-4">
								<Badge variant="secondary" class="text-lg px-4 py-1">
									{formatDisposition(disposition)}
								</Badge>
								{#if affectedQuantity}
									<span class="text-sm text-muted-foreground">
										Affected Quantity: {affectedQuantity}
									</span>
								{/if}
							</div>
							{#if dispositionRationale}
								<div>
									<h4 class="text-sm font-medium text-muted-foreground">Rationale</h4>
									<p class="mt-1">{dispositionRationale}</p>
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
				<!-- NCR Status -->
				<Card>
					<CardHeader>
						<CardTitle>NCR Status</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">NCR Status</span>
							<Badge variant={getNcrStatusVariant(ncrStatus)}>
								{formatNcrStatus(ncrStatus)}
							</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Severity</span>
							<Badge variant={getSeverityVariant(severity)} class="capitalize">
								{severity}
							</Badge>
						</div>
					</CardContent>
				</Card>

				<!-- Properties -->
				<Card>
					<CardHeader>
						<CardTitle>Properties</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Type</span>
							<Badge variant="outline">{formatNcrType(ncrType)}</Badge>
						</div>
						{#if category}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Category</span>
								<span class="text-sm font-medium">{category}</span>
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
				<p class="text-muted-foreground">NCR not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
