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
		Layers,
		User,
		Calendar,
		Tag,
		Box,
		FileText,
		Shield,
		Factory,
		Hash
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
	const partNumber = $derived((data.part_number as string) ?? '');
	const partRevision = $derived((data.revision as string) ?? null);
	const description = $derived((data.description as string) ?? null);
	const entityRevision = $derived((data.entity_revision as number) ?? 1);

	// Safety classifications
	const swClass = $derived((data.sw_class as string) ?? null);
	const asil = $derived((data.asil as string) ?? null);
	const dal = $derived((data.dal as string) ?? null);

	// BOM items
	interface BomItem {
		component_id: string;
		quantity: number;
		reference_designators?: string[];
		notes?: string;
	}
	const bom = $derived((data.bom as BomItem[]) ?? []);

	// Sub-assemblies
	const subassemblies = $derived((data.subassemblies as string[]) ?? []);

	// Documents
	interface Document {
		doc_type: string;
		path: string;
		revision?: string;
	}
	const documents = $derived((data.documents as Document[]) ?? []);

	// Manufacturing
	interface ManufacturingConfig {
		routing?: string[];
		work_cell?: string;
	}
	const manufacturing = $derived(data.manufacturing as ManufacturingConfig | null);

	// Calculate total component count
	const totalQuantity = $derived(bom.reduce((sum, item) => sum + item.quantity, 0));

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
			console.error('Failed to load assembly:', e);
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
			backHref="/assemblies"
			backLabel="Assemblies"
			onEdit={() => goto(`/assemblies/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Description -->
				{#if description}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Layers class="h-5 w-5" />
								Description
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap">{description}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Bill of Materials -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Box class="h-5 w-5" />
							Bill of Materials ({bom.length} line items, {totalQuantity} total)
						</CardTitle>
					</CardHeader>
					<CardContent>
						{#if bom.length === 0}
							<p class="py-4 text-center text-muted-foreground">No components in BOM</p>
						{:else}
							<div class="space-y-2">
								{#each bom as item, i}
									<button
										class="flex w-full items-center justify-between rounded-lg border p-3 text-left transition-colors hover:bg-muted/50"
										onclick={() => goto(`/components/${item.component_id}`)}
									>
										<div class="flex items-center gap-4">
											<span class="text-sm text-muted-foreground">#{i + 1}</span>
											<div>
												<p class="font-mono text-sm">{item.component_id}</p>
												{#if item.reference_designators && item.reference_designators.length > 0}
													<p class="mt-1 text-xs text-muted-foreground">
														Ref: {item.reference_designators.join(', ')}
													</p>
												{/if}
												{#if item.notes}
													<p class="mt-1 text-xs text-muted-foreground">{item.notes}</p>
												{/if}
											</div>
										</div>
										<Badge variant="outline">
											<Hash class="mr-1 h-3 w-3" />
											{item.quantity}
										</Badge>
									</button>
								{/each}
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Sub-assemblies -->
				{#if subassemblies.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Layers class="h-5 w-5" />
								Sub-assemblies ({subassemblies.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								{#each subassemblies as subId}
									<button
										class="flex w-full items-center rounded-lg border p-3 text-left transition-colors hover:bg-muted/50"
										onclick={() => goto(`/assemblies/${subId}`)}
									>
										<Layers class="mr-3 h-4 w-4 text-muted-foreground" />
										<span class="font-mono text-sm">{subId}</span>
									</button>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Manufacturing -->
				{#if manufacturing && (manufacturing.routing?.length || manufacturing.work_cell)}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Factory class="h-5 w-5" />
								Manufacturing
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if manufacturing.work_cell}
								<div>
									<h4 class="text-sm text-muted-foreground">Work Cell</h4>
									<p class="mt-1 font-medium">{manufacturing.work_cell}</p>
								</div>
							{/if}
							{#if manufacturing.routing && manufacturing.routing.length > 0}
								<div>
									<h4 class="text-sm text-muted-foreground">Process Routing</h4>
									<div class="mt-2 flex flex-wrap gap-2">
										{#each manufacturing.routing as procId}
											<Badge variant="outline">{procId}</Badge>
										{/each}
									</div>
								</div>
							{/if}
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
				<!-- Assembly Info -->
				<Card>
					<CardHeader>
						<CardTitle>Assembly Information</CardTitle>
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
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">BOM Items</span>
							<Badge variant="outline">{bom.length}</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Total Quantity</span>
							<Badge variant="outline">{totalQuantity}</Badge>
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
				<p class="text-muted-foreground">Assembly not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
