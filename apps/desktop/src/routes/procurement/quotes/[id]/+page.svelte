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
		FileSpreadsheet,
		User,
		Calendar,
		Tag,
		FileText,
		DollarSign,
		Clock,
		Building2
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
	const supplierId = $derived((data.supplier_id as string) ?? null);
	const supplierName = $derived((data.supplier_name as string) ?? null);
	const description = $derived((data.description as string) ?? '');
	const quoteNumber = $derived((data.quote_number as string) ?? null);
	const validUntil = $derived((data.valid_until as string) ?? null);
	const leadTime = $derived((data.lead_time as string) ?? null);
	const currency = $derived((data.currency as string) ?? 'USD');
	const revision = $derived((data.revision as number) ?? 1);

	interface LineItem {
		description: string;
		part_number?: string;
		quantity?: number;
		unit_price?: number;
		total_price?: number;
	}
	const lineItems = $derived((data.line_items as LineItem[]) ?? []);

	const totalPrice = $derived(
		lineItems.reduce((sum, item) => sum + (item.total_price ?? 0), 0)
	);

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
			console.error('Failed to load quote:', e);
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

	function formatCurrency(amount: number): string {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: currency
		}).format(amount);
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
			subtitle={quoteNumber ? `Quote #${quoteNumber}` : 'Supplier Quote'}
			backHref="/procurement/quotes"
			backLabel="Quotes"
			onEdit={() => goto(`/procurement/quotes/${id}/edit`)}
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

				<!-- Line Items -->
				{#if lineItems.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<FileSpreadsheet class="h-5 w-5" />
								Line Items ({lineItems.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="overflow-x-auto">
								<table class="w-full text-sm">
									<thead>
										<tr class="border-b">
											<th class="px-4 py-2 text-left font-medium">Description</th>
											<th class="px-4 py-2 text-left font-medium">Part #</th>
											<th class="px-4 py-2 text-right font-medium">Qty</th>
											<th class="px-4 py-2 text-right font-medium">Unit Price</th>
											<th class="px-4 py-2 text-right font-medium">Total</th>
										</tr>
									</thead>
									<tbody>
										{#each lineItems as item}
											<tr class="border-b">
												<td class="px-4 py-2">{item.description}</td>
												<td class="px-4 py-2 font-mono text-xs">{item.part_number ?? '—'}</td>
												<td class="px-4 py-2 text-right">{item.quantity ?? '—'}</td>
												<td class="px-4 py-2 text-right">
													{item.unit_price ? formatCurrency(item.unit_price) : '—'}
												</td>
												<td class="px-4 py-2 text-right font-medium">
													{item.total_price ? formatCurrency(item.total_price) : '—'}
												</td>
											</tr>
										{/each}
									</tbody>
									<tfoot>
										<tr class="bg-muted/50">
											<td colspan="4" class="px-4 py-2 text-right font-medium">Total:</td>
											<td class="px-4 py-2 text-right font-bold">{formatCurrency(totalPrice)}</td>
										</tr>
									</tfoot>
								</table>
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Links -->
				<LinksSection {linksFrom} {linksTo} loading={linksLoading} />
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Quote Summary -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<DollarSign class="h-4 w-4" />
							Summary
						</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="rounded-lg bg-muted/50 p-4 text-center">
							<div class="text-sm text-muted-foreground">Total Value</div>
							<div class="mt-1 text-2xl font-bold">{formatCurrency(totalPrice)}</div>
						</div>
						{#if lineItems.length > 0}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Line Items</span>
								<span class="text-sm font-medium">{lineItems.length}</span>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Supplier -->
				{#if supplierId || supplierName}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Building2 class="h-4 w-4" />
								Supplier
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if supplierName}
								<div class="font-medium">{supplierName}</div>
							{/if}
							{#if supplierId}
								<button
									class="font-mono text-xs text-primary hover:underline"
									onclick={() => goto(`/procurement/suppliers/${supplierId}`)}
								>
									{supplierId}
								</button>
							{/if}
						</CardContent>
					</Card>
				{/if}

				<!-- Properties -->
				<Card>
					<CardHeader>
						<CardTitle>Properties</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if quoteNumber}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Quote #</span>
								<span class="font-mono text-sm">{quoteNumber}</span>
							</div>
						{/if}
						{#if validUntil}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Valid Until</span>
								<span class="text-sm font-medium">{formatDate(validUntil)}</span>
							</div>
						{/if}
						{#if leadTime}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Lead Time</span>
								<div class="flex items-center gap-1">
									<Clock class="h-3 w-3 text-muted-foreground" />
									<span class="text-sm font-medium">{leadTime}</span>
								</div>
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
				<p class="text-muted-foreground">Quote not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
