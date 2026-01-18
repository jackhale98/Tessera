<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge, Button } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection, AddFeatureDialog, AddQuoteDialog } from '$lib/components/entities';
	import EntityHistory from '$lib/components/EntityHistory.svelte';
	import { StatusBadge } from '$lib/components/common';
	import { entities, traceability } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import {
		Box,
		User,
		Calendar,
		Tag,
		Package,
		DollarSign,
		Scale,
		Truck,
		FileText,
		Shield,
		Factory,
		CircleDot,
		Plus,
		Receipt,
		Layers,
		History
	} from 'lucide-svelte';

	const id = $derived($page.params.id);

	let entity = $state<EntityData | null>(null);
	let linksFrom = $state<LinkInfo[]>([]);
	let linksTo = $state<LinkInfo[]>([]);
	let loading = $state(true);
	let linksLoading = $state(true);
	let error = $state<string | null>(null);

	// Features for this component
	let features = $state<Array<{ id: string; title: string; feature_type: string; status: string }>>([]);
	let featuresLoading = $state(false);
	let addFeatureDialogOpen = $state(false);

	// Quotes for this component
	interface QuoteInfo {
		id: string;
		title: string;
		supplier: string;
		supplier_name?: string;
		unit_price?: number;
		currency?: string;
		moq?: number;
		lead_time_days?: number;
		quote_status: string;
		status: string;
	}
	let quotes = $state<QuoteInfo[]>([]);
	let quotesLoading = $state(false);
	let addQuoteDialogOpen = $state(false);

	// Type-safe data access
	const data = $derived(entity?.data ?? {});
	const partNumber = $derived((data.part_number as string) ?? '');
	const partRevision = $derived((data.revision as string) ?? null);
	const description = $derived((data.description as string) ?? null);
	const category = $derived((data.category as string) ?? 'mechanical');
	const makeBuy = $derived((data.make_buy as string) ?? 'make');
	const material = $derived((data.material as string) ?? null);
	const massKg = $derived(data.mass_kg as number | null);
	const rawUnitCost = $derived(data.unit_cost as number | null);
	const selectedQuoteId = $derived((data.selected_quote as string) ?? null);
	const entityRevision = $derived((data.entity_revision as number) ?? 1);

	// Computed unit cost: prioritize selected quote, then any quote, then raw unit_cost
	const unitCost = $derived.by(() => {
		// Priority 1: Use selected quote if set
		if (selectedQuoteId) {
			const selectedQuote = quotes.find((q) => q.id === selectedQuoteId);
			if (selectedQuote?.unit_price != null) {
				return selectedQuote.unit_price;
			}
		}
		// Priority 2: Fall back to raw unit_cost from component
		return rawUnitCost;
	});

	// Track whether cost comes from quote for display purposes
	const costSource = $derived.by(() => {
		if (selectedQuoteId) {
			const selectedQuote = quotes.find((q) => q.id === selectedQuoteId);
			if (selectedQuote?.unit_price != null) {
				return { type: 'quote' as const, quote: selectedQuote };
			}
		}
		return rawUnitCost != null ? { type: 'manual' as const } : null;
	});

	// Safety classifications
	const swClass = $derived((data.sw_class as string) ?? null);
	const asil = $derived((data.asil as string) ?? null);
	const dal = $derived((data.dal as string) ?? null);

	// Suppliers
	interface Supplier {
		supplier_id?: string;
		name: string;
		supplier_pn?: string;
		lead_time_days?: number;
		moq?: number;
		unit_cost?: number;
	}
	const suppliers = $derived((data.suppliers as Supplier[]) ?? []);

	// Documents
	interface Document {
		doc_type: string;
		path: string;
		revision?: string;
	}
	const documents = $derived((data.documents as Document[]) ?? []);

	// Assemblies that use this component (from incoming "contains" links)
	const usedInAssemblies = $derived(
		linksTo.filter(
			(link) => link.link_type === 'contains' || link.source_id.startsWith('ASM-')
		)
	);

	async function loadData() {
		if (!id) return;

		console.log('[Component Detail] Loading data for:', id);
		loading = true;
		linksLoading = true;
		error = null;

		try {
			console.log('[Component Detail] Starting API calls...');

			// Make calls separately to identify which one hangs
			console.log('[Component Detail] Fetching entity...');
			const entityResult = await entities.get(id);
			console.log('[Component Detail] Entity fetched:', entityResult?.title);

			console.log('[Component Detail] Fetching links from...');
			const fromLinks = await traceability.getLinksFrom(id);
			console.log('[Component Detail] Links from fetched:', fromLinks.length);

			console.log('[Component Detail] Fetching links to...');
			const toLinks = await traceability.getLinksTo(id);
			console.log('[Component Detail] Links to fetched:', toLinks.length);

			entity = entityResult;
			linksFrom = fromLinks;
			linksTo = toLinks;
			console.log('[Component Detail] All data loaded successfully');
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('[Component Detail] Failed to load component:', e);
		} finally {
			loading = false;
			linksLoading = false;
			console.log('[Component Detail] Loading complete, loading=', loading);
		}

		// Load features and quotes for this component
		loadFeatures();
		loadQuotes();
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

	async function loadFeatures() {
		if (!id) return;

		featuresLoading = true;
		try {
			// Get all features and filter by component ID
			const result = await entities.list('FEAT', { limit: 100 });
			features = result.items
				.filter((f) => f.data?.component === id)
				.map((f) => ({
					id: f.id,
					title: f.title,
					feature_type: (f.data?.feature_type as string) ?? 'external',
					status: f.status
				}));
		} catch (e) {
			console.error('Failed to load features:', e);
		} finally {
			featuresLoading = false;
		}
	}

	async function loadQuotes() {
		if (!id) return;

		quotesLoading = true;
		try {
			// Get all quotes and filter by component ID
			const result = await entities.list('QUOT', { limit: 100 });
			const componentQuotes = result.items.filter((q) => q.data?.component === id);

			// Load supplier names for each quote
			const quotesWithSuppliers = await Promise.all(
				componentQuotes.map(async (q) => {
					const supplierId = q.data?.supplier as string;
					let supplierName = supplierId;

					// Try to get supplier name
					if (supplierId) {
						try {
							const supplier = await entities.get(supplierId);
							if (supplier) {
								supplierName = supplier.title;
							}
						} catch {
							// Keep ID as fallback
						}
					}

					// Get first price break's unit price if available
					const priceBreaks = (q.data?.price_breaks as Array<{ min_qty: number; unit_price: number }>) ?? [];
					const firstPrice = priceBreaks.length > 0 ? priceBreaks[0]?.unit_price : undefined;

					return {
						id: q.id,
						title: q.title,
						supplier: supplierId,
						supplier_name: supplierName,
						unit_price: firstPrice,
						currency: (q.data?.currency as string) ?? 'USD',
						moq: q.data?.moq as number | undefined,
						lead_time_days: q.data?.lead_time_days as number | undefined,
						quote_status: (q.data?.quote_status as string) ?? 'pending',
						status: q.status
					};
				})
			);

			quotes = quotesWithSuppliers;
		} catch (e) {
			console.error('Failed to load quotes:', e);
		} finally {
			quotesLoading = false;
		}
	}

	function handleFeatureCreated(featureId: string) {
		// Reload features to show the new one
		loadFeatures();
		// Also reload links since we added a link
		loadData();
	}

	function handleQuoteCreated(quoteId: string) {
		// Reload quotes to show the new one
		loadQuotes();
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

	function formatCurrency(value: number | null | undefined): string {
		if (value == null) return '—';
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD'
		}).format(value);
	}

	function formatMass(kg: number | null | undefined): string {
		if (kg == null) return '—';
		if (kg < 1) return `${(kg * 1000).toFixed(1)} g`;
		return `${kg.toFixed(3)} kg`;
	}

	function getCategoryLabel(cat: string): string {
		const labels: Record<string, string> = {
			mechanical: 'Mechanical',
			electrical: 'Electrical',
			software: 'Software',
			fastener: 'Fastener',
			consumable: 'Consumable'
		};
		return labels[cat.toLowerCase()] ?? cat;
	}

	function getMakeBuyVariant(mb: string): 'default' | 'secondary' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'outline'> = {
			make: 'default',
			buy: 'secondary',
			modify: 'outline'
		};
		return variants[mb.toLowerCase()] ?? 'outline';
	}

	// Track loaded ID to prevent double-loads
	let loadedId = $state<string | null>(null);

	// Load data when ID changes
	$effect(() => {
		const currentId = id;
		if (currentId && currentId !== loadedId) {
			console.log('[Component Detail] Effect triggered for ID:', currentId);
			loadedId = currentId;
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
			subtitle={partNumber ? `P/N: ${partNumber}${partRevision ? ` Rev ${partRevision}` : ''}` : undefined}
			backHref="/components"
			backLabel="Components"
			onEdit={() => goto(`/components/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Description -->
				{#if description}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Box class="h-5 w-5" />
								Description
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap">{description}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Key Specifications -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Package class="h-5 w-5" />
							Specifications
						</CardTitle>
					</CardHeader>
					<CardContent>
						<dl class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
							<div class="rounded-lg border p-4">
								<dt class="flex items-center gap-2 text-sm text-muted-foreground">
									<DollarSign class="h-4 w-4" />
									Unit Cost
									{#if costSource?.type === 'quote'}
										<Badge variant="secondary" class="ml-auto text-xs">Quote</Badge>
									{/if}
								</dt>
								<dd class="mt-1 text-xl font-bold">{formatCurrency(unitCost)}</dd>
								{#if costSource?.type === 'quote' && costSource.quote}
									<dd class="mt-1 text-xs text-muted-foreground">
										From: {costSource.quote.supplier_name ?? costSource.quote.supplier}
									</dd>
								{/if}
							</div>
							<div class="rounded-lg border p-4">
								<dt class="flex items-center gap-2 text-sm text-muted-foreground">
									<Scale class="h-4 w-4" />
									Mass
								</dt>
								<dd class="mt-1 text-xl font-bold">{formatMass(massKg)}</dd>
							</div>
							{#if material}
								<div class="rounded-lg border p-4">
									<dt class="text-sm text-muted-foreground">Material</dt>
									<dd class="mt-1 text-xl font-bold">{material}</dd>
								</div>
							{/if}
						</dl>
					</CardContent>
				</Card>

				<!-- Used In Assemblies -->
				{#if usedInAssemblies.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Layers class="h-5 w-5" />
								Used In Assemblies ({usedInAssemblies.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								{#each usedInAssemblies as assembly}
									<button
										class="flex w-full items-center justify-between rounded-lg border p-3 text-left transition-colors hover:bg-muted/50"
										onclick={() => goto(`/assemblies/${assembly.source_id}`)}
									>
										<div class="flex items-center gap-3">
											<Layers class="h-4 w-4 text-blue-500" />
											<div>
												<p class="font-medium">{assembly.target_title ?? assembly.source_id}</p>
												<p class="text-xs text-muted-foreground font-mono">
													{assembly.source_id}
												</p>
											</div>
										</div>
									</button>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Features -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle class="flex items-center gap-2">
								<CircleDot class="h-5 w-5" />
								Features ({features.length})
							</CardTitle>
							<Button size="sm" onclick={() => (addFeatureDialogOpen = true)}>
								<Plus class="mr-1 h-4 w-4" />
								Add Feature
							</Button>
						</div>
					</CardHeader>
					<CardContent>
						{#if featuresLoading}
							<div class="flex justify-center py-4">
								<div class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
							</div>
						{:else if features.length === 0}
							<div class="rounded-lg border border-dashed p-6 text-center">
								<CircleDot class="mx-auto h-8 w-8 text-muted-foreground/50" />
								<p class="mt-2 text-sm text-muted-foreground">
									No features defined for this component
								</p>
								<Button
									size="sm"
									variant="outline"
									class="mt-3"
									onclick={() => (addFeatureDialogOpen = true)}
								>
									<Plus class="mr-1 h-4 w-4" />
									Add First Feature
								</Button>
							</div>
						{:else}
							<div class="space-y-2">
								{#each features as feature}
									<button
										class="flex w-full items-center justify-between rounded-lg border p-3 text-left transition-colors hover:bg-muted/50"
										onclick={() => goto(`/features/${feature.id}`)}
									>
										<div class="flex items-center gap-3">
											<CircleDot class="h-4 w-4 text-muted-foreground" />
											<div>
												<p class="font-medium">{feature.title}</p>
												<p class="text-xs text-muted-foreground capitalize">
													{feature.feature_type}
												</p>
											</div>
										</div>
										<Badge variant="outline" class="capitalize">{feature.status}</Badge>
									</button>
								{/each}
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Quotes -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle class="flex items-center gap-2">
								<Receipt class="h-5 w-5" />
								Quotes ({quotes.length})
							</CardTitle>
							<Button size="sm" onclick={() => (addQuoteDialogOpen = true)}>
								<Plus class="mr-1 h-4 w-4" />
								Add Quote
							</Button>
						</div>
					</CardHeader>
					<CardContent>
						{#if quotesLoading}
							<div class="flex justify-center py-4">
								<div class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
							</div>
						{:else if quotes.length === 0}
							<div class="rounded-lg border border-dashed p-6 text-center">
								<Receipt class="mx-auto h-8 w-8 text-muted-foreground/50" />
								<p class="mt-2 text-sm text-muted-foreground">
									No quotes for this component yet
								</p>
								<Button
									size="sm"
									variant="outline"
									class="mt-3"
									onclick={() => (addQuoteDialogOpen = true)}
								>
									<Plus class="mr-1 h-4 w-4" />
									Add First Quote
								</Button>
							</div>
						{:else}
							<div class="space-y-2">
								{#each quotes as quote}
									{@const isSelected = quote.id === selectedQuoteId}
									<button
										class="flex w-full items-center justify-between rounded-lg border p-3 text-left transition-colors hover:bg-muted/50 {isSelected ? 'border-primary bg-primary/5' : ''}"
										onclick={() => goto(`/procurement/quotes/${quote.id}`)}
									>
										<div class="flex items-center gap-3">
											<Receipt class="h-4 w-4 {isSelected ? 'text-primary' : 'text-muted-foreground'}" />
											<div>
												<p class="font-medium">
													{quote.title}
													{#if isSelected}
														<Badge variant="default" class="ml-2 text-xs">Selected</Badge>
													{/if}
												</p>
												<p class="text-xs text-muted-foreground">
													{quote.supplier_name ?? quote.supplier}
													{#if quote.lead_time_days}
														<span class="ml-2">| {quote.lead_time_days} days</span>
													{/if}
												</p>
											</div>
										</div>
										<div class="flex items-center gap-2">
											{#if quote.unit_price}
												<span class="font-bold {isSelected ? 'text-primary' : 'text-green-600'}">
													{formatCurrency(quote.unit_price)}
												</span>
											{/if}
											<Badge variant="outline" class="capitalize">{quote.quote_status}</Badge>
										</div>
									</button>
								{/each}
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Suppliers -->
				{#if suppliers.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Truck class="h-5 w-5" />
								Suppliers ({suppliers.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-3">
								{#each suppliers as supplier}
									<div class="rounded-lg border p-4">
										<div class="flex items-start justify-between">
											<div>
												<p class="font-medium">{supplier.name}</p>
												{#if supplier.supplier_pn}
													<p class="mt-1 font-mono text-sm text-muted-foreground">
														P/N: {supplier.supplier_pn}
													</p>
												{/if}
											</div>
											{#if supplier.unit_cost}
												<span class="font-bold text-green-600">
													{formatCurrency(supplier.unit_cost)}
												</span>
											{/if}
										</div>
										{#if supplier.lead_time_days || supplier.moq}
											<div class="mt-3 flex gap-4 text-sm text-muted-foreground">
												{#if supplier.lead_time_days}
													<span>Lead time: {supplier.lead_time_days} days</span>
												{/if}
												{#if supplier.moq}
													<span>MOQ: {supplier.moq}</span>
												{/if}
											</div>
										{/if}
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Documents -->
				{#if documents.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<FileText class="h-5 w-5" />
								Documents ({documents.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								{#each documents as doc}
									<div class="flex items-center justify-between rounded-lg border p-3">
										<div class="flex items-center gap-3">
											<FileText class="h-4 w-4 text-muted-foreground" />
											<div>
												<p class="font-medium">{doc.path}</p>
												<p class="text-xs text-muted-foreground capitalize">
													{doc.doc_type}
													{#if doc.revision}
														<span class="ml-2">Rev {doc.revision}</span>
													{/if}
												</p>
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
				<!-- Part Info -->
				<Card>
					<CardHeader>
						<CardTitle>Part Information</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Part Number</span>
							<span class="font-mono font-medium">{partNumber || '—'}</span>
						</div>
						{#if partRevision}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Revision</span>
								<span class="font-medium">{partRevision}</span>
							</div>
						{/if}
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Category</span>
							<Badge variant="outline">{getCategoryLabel(category)}</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Make/Buy</span>
							<Badge variant={getMakeBuyVariant(makeBuy)} class="uppercase">
								{makeBuy}
							</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
					</CardContent>
				</Card>

				<!-- Safety Classifications -->
				{#if swClass || asil || dal}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Shield class="h-4 w-4" />
								Safety Classifications
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if swClass}
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">IEC 62304</span>
									<Badge variant="destructive">{swClass}</Badge>
								</div>
							{/if}
							{#if asil}
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">ISO 26262 ASIL</span>
									<Badge variant="destructive">{asil}</Badge>
								</div>
							{/if}
							{#if dal}
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">DO-178C DAL</span>
									<Badge variant="destructive">{dal}</Badge>
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
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Entity Revision</span>
							<span class="text-sm font-medium">{entityRevision}</span>
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
				<p class="text-muted-foreground">Component not found</p>
			</CardContent>
		</Card>
	{/if}
</div>

<!-- Add Feature Dialog -->
{#if entity}
	<AddFeatureDialog
		bind:open={addFeatureDialogOpen}
		componentId={entity.id}
		componentTitle={entity.title}
		onClose={() => (addFeatureDialogOpen = false)}
		onCreated={handleFeatureCreated}
	/>
{/if}

<!-- Add Quote Dialog -->
{#if entity}
	<AddQuoteDialog
		bind:open={addQuoteDialogOpen}
		componentId={entity.id}
		componentTitle={entity.title}
		onClose={() => (addQuoteDialogOpen = false)}
		onCreated={handleQuoteCreated}
	/>
{/if}
