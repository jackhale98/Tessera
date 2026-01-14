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
		Factory
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
	const partNumber = $derived((data.part_number as string) ?? '');
	const partRevision = $derived((data.revision as string) ?? null);
	const description = $derived((data.description as string) ?? null);
	const category = $derived((data.category as string) ?? 'mechanical');
	const makeBuy = $derived((data.make_buy as string) ?? 'make');
	const material = $derived((data.material as string) ?? null);
	const massKg = $derived(data.mass_kg as number | null);
	const unitCost = $derived(data.unit_cost as number | null);
	const selectedQuote = $derived((data.selected_quote as string) ?? null);
	const entityRevision = $derived((data.entity_revision as number) ?? 1);

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

	function formatCurrency(value: number | null): string {
		if (value === null) return '—';
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD'
		}).format(value);
	}

	function formatMass(kg: number | null): string {
		if (kg === null) return '—';
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
								</dt>
								<dd class="mt-1 text-xl font-bold">{formatCurrency(unitCost)}</dd>
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
				<LinksSection {linksFrom} {linksTo} loading={linksLoading} />
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
