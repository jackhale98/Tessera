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
		Receipt,
		User,
		Calendar,
		Tag,
		FileText,
		DollarSign,
		Clock,
		Building2,
		Package,
		TrendingDown,
		History
	} from 'lucide-svelte';

	const id = $derived($page.params.id);

	let entity = $state<EntityData | null>(null);
	let linksFrom = $state<LinkInfo[]>([]);
	let linksTo = $state<LinkInfo[]>([]);
	let loading = $state(true);
	let linksLoading = $state(true);
	let error = $state<string | null>(null);

	// Supplier info
	let supplierName = $state<string | null>(null);

	// Component/Assembly info
	let linkedItemName = $state<string | null>(null);

	// Track if we've loaded for this ID to prevent double-loads
	let loadedId = $state<string | null>(null);

	// Type-safe data access
	const data = $derived(entity?.data ?? {});
	const supplierId = $derived((data.supplier as string) ?? null);
	const componentId = $derived((data.component as string) ?? null);
	const assemblyId = $derived((data.assembly as string) ?? null);
	const description = $derived((data.description as string) ?? '');
	const quoteRef = $derived((data.quote_ref as string) ?? null);
	const quoteDate = $derived((data.quote_date as string) ?? null);
	const validUntil = $derived((data.valid_until as string) ?? null);
	const leadTimeDays = $derived((data.lead_time_days as number) ?? null);
	const currency = $derived((data.currency as string) ?? 'USD');
	const moq = $derived((data.moq as number) ?? null);
	const toolingCost = $derived((data.tooling_cost as number) ?? null);
	const quoteStatus = $derived((data.quote_status as string) ?? 'pending');
	const revision = $derived((data.entity_revision as number) ?? 1);

	interface PriceBreak {
		min_qty: number;
		unit_price: number;
		lead_time_days?: number;
	}
	const priceBreaks = $derived((data.price_breaks as PriceBreak[]) ?? []);

	interface NreCost {
		description: string;
		cost: number;
		one_time?: boolean;
	}
	const nreCosts = $derived((data.nre_costs as NreCost[]) ?? []);

	// Calculated values
	const unitPrice = $derived(() => {
		if (priceBreaks.length === 0) return null;
		const sorted = [...priceBreaks].sort((a, b) => a.min_qty - b.min_qty);
		return sorted[0]?.unit_price ?? null;
	});

	const totalNre = $derived(() => {
		const nreSum = nreCosts.reduce((sum, n) => sum + n.cost, 0);
		return nreSum + (toolingCost ?? 0);
	});

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

			// Load supplier name
			if (entityResult?.data?.supplier) {
				try {
					const supplier = await entities.get(entityResult.data.supplier as string);
					supplierName = supplier?.title ?? null;
				} catch {
					supplierName = null;
				}
			}

			// Load component/assembly name
			const linkedId = (entityResult?.data?.component as string) ?? (entityResult?.data?.assembly as string);
			if (linkedId) {
				try {
					const linked = await entities.get(linkedId);
					linkedItemName = linked?.title ?? null;
				} catch {
					linkedItemName = null;
				}
			}
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

	function getQuoteStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (status.toLowerCase()) {
			case 'accepted':
				return 'default';
			case 'received':
				return 'secondary';
			case 'rejected':
			case 'expired':
				return 'destructive';
			default:
				return 'outline';
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
			subtitle={quoteRef ? `Ref: ${quoteRef}` : supplierName ?? 'Supplier Quote'}
			backHref="/procurement/quotes"
			backLabel="Quotes"
			onEdit={() => goto(`/procurement/quotes/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Description -->
				{#if description}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<FileText class="h-5 w-5" />
								Description
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap">{description}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Price Breaks -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<TrendingDown class="h-5 w-5" />
							Price Breaks ({priceBreaks.length})
						</CardTitle>
					</CardHeader>
					<CardContent>
						{#if priceBreaks.length > 0}
							<div class="overflow-x-auto">
								<table class="w-full text-sm">
									<thead>
										<tr class="border-b">
											<th class="px-4 py-2 text-left font-medium">Min Quantity</th>
											<th class="px-4 py-2 text-right font-medium">Unit Price</th>
											<th class="px-4 py-2 text-right font-medium">Lead Time</th>
										</tr>
									</thead>
									<tbody>
										{#each [...priceBreaks].sort((a, b) => a.min_qty - b.min_qty) as pb, index}
											<tr class="border-b {index === 0 ? 'bg-green-500/10' : ''}">
												<td class="px-4 py-3">
													<span class="font-medium">{pb.min_qty}+</span>
													{#if index === 0}
														<Badge variant="outline" class="ml-2 text-xs">Base</Badge>
													{/if}
												</td>
												<td class="px-4 py-3 text-right">
													<span class="font-bold text-green-600 dark:text-green-400">
														{formatCurrency(pb.unit_price)}
													</span>
												</td>
												<td class="px-4 py-3 text-right text-muted-foreground">
													{#if pb.lead_time_days}
														{pb.lead_time_days} days
													{:else if leadTimeDays}
														{leadTimeDays} days
													{:else}
														—
													{/if}
												</td>
											</tr>
										{/each}
									</tbody>
								</table>
							</div>
						{:else}
							<p class="text-muted-foreground">No price breaks defined.</p>
						{/if}
					</CardContent>
				</Card>

				<!-- NRE / Tooling Costs -->
				{#if totalNre() > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<DollarSign class="h-5 w-5" />
								NRE / Tooling Costs
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								{#if toolingCost}
									<div class="flex justify-between rounded-lg border p-3">
										<span>Tooling Cost</span>
										<span class="font-medium">{formatCurrency(toolingCost)}</span>
									</div>
								{/if}
								{#each nreCosts as nre}
									<div class="flex justify-between rounded-lg border p-3">
										<span>
											{nre.description}
											{#if nre.one_time}
												<Badge variant="outline" class="ml-2 text-xs">One-time</Badge>
											{/if}
										</span>
										<span class="font-medium">{formatCurrency(nre.cost)}</span>
									</div>
								{/each}
								<div class="flex justify-between rounded-lg bg-muted/50 p-3 font-medium">
									<span>Total NRE</span>
									<span>{formatCurrency(totalNre())}</span>
								</div>
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
				<!-- Quote Summary -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Receipt class="h-4 w-4" />
							Summary
						</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if unitPrice() !== null}
							<div class="rounded-lg bg-muted/50 p-4 text-center">
								<div class="text-sm text-muted-foreground">Unit Price (qty 1)</div>
								<div class="mt-1 text-2xl font-bold text-green-600 dark:text-green-400">
									{formatCurrency(unitPrice()!)}
								</div>
							</div>
						{/if}
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Quote Status</span>
							<Badge variant={getQuoteStatusVariant(quoteStatus)} class="capitalize">
								{quoteStatus}
							</Badge>
						</div>
						{#if priceBreaks.length > 1}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Price Breaks</span>
								<span class="text-sm font-medium">{priceBreaks.length}</span>
							</div>
						{/if}
						{#if moq}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">MOQ</span>
								<span class="text-sm font-medium">{moq}</span>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Supplier -->
				{#if supplierId}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Building2 class="h-4 w-4" />
								Supplier
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-2">
							{#if supplierName}
								<div class="font-medium">{supplierName}</div>
							{/if}
							<button
								class="font-mono text-xs text-primary hover:underline"
								onclick={() => goto(`/procurement/suppliers/${supplierId}`)}
							>
								{supplierId}
							</button>
						</CardContent>
					</Card>
				{/if}

				<!-- Linked Component/Assembly -->
				{#if componentId || assemblyId}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Package class="h-4 w-4" />
								{componentId ? 'Component' : 'Assembly'}
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-2">
							{#if linkedItemName}
								<div class="font-medium">{linkedItemName}</div>
							{/if}
							<button
								class="font-mono text-xs text-primary hover:underline"
								onclick={() => goto(componentId ? `/components/${componentId}` : `/assemblies/${assemblyId}`)}
							>
								{componentId ?? assemblyId}
							</button>
						</CardContent>
					</Card>
				{/if}

				<!-- Properties -->
				<Card>
					<CardHeader>
						<CardTitle>Properties</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if quoteRef}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Quote Ref</span>
								<span class="font-mono text-sm">{quoteRef}</span>
							</div>
						{/if}
						{#if quoteDate}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Quote Date</span>
								<span class="text-sm font-medium">{formatDate(quoteDate)}</span>
							</div>
						{/if}
						{#if validUntil}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Valid Until</span>
								<span class="text-sm font-medium">{formatDate(validUntil)}</span>
							</div>
						{/if}
						{#if leadTimeDays}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Default Lead Time</span>
								<div class="flex items-center gap-1">
									<Clock class="h-3 w-3 text-muted-foreground" />
									<span class="text-sm font-medium">{leadTimeDays} days</span>
								</div>
							</div>
						{/if}
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Currency</span>
							<span class="text-sm font-medium">{currency}</span>
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
