<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge, Button, Input } from '$lib/components/ui';
	import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { entities, traceability, assemblies } from '$lib/api/tauri';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo, BomNode, BomCostResult, BomMassResult } from '$lib/api/tauri';
	import {
		Layers,
		User,
		Calendar,
		Tag,
		Box,
		FileText,
		Shield,
		Factory,
		Hash,
		DollarSign,
		Scale,
		ChevronRight,
		ChevronDown,
		Calculator,
		AlertTriangle
	} from 'lucide-svelte';

	const id = $derived($page.params.id);

	let entity = $state<EntityData | null>(null);
	let linksFrom = $state<LinkInfo[]>([]);
	let linksTo = $state<LinkInfo[]>([]);
	let bomTree = $state<BomNode | null>(null);
	let costResult = $state<BomCostResult | null>(null);
	let massResult = $state<BomMassResult | null>(null);
	let loading = $state(true);
	let linksLoading = $state(true);
	let bomLoading = $state(false);
	let error = $state<string | null>(null);

	// Quantity input for cost extrapolation
	let quantity = $state(1);
	let expandedNodes = $state<Set<string>>(new Set());

	// Track if we've loaded for this ID to prevent double-loads
	let loadedId = $state<string | null>(null);

	// Debounce timer for quantity changes
	let quantityDebounce: ReturnType<typeof setTimeout> | null = null;

	// Editing state for component quantities
	let editingComponentId = $state<string | null>(null);
	let editingQuantityValue = $state<number>(0);
	let savingQuantity = $state(false);

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

	// BOM items (raw from entity)
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

	// Unit cost (cost for one assembly, calculated from total / quantity)
	// The API returns total cost for the quantity, so we divide to get unit cost
	const unitCost = $derived(costResult && quantity > 0 ? costResult.total_cost / quantity : null);
	const unitMass = $derived(massResult && quantity > 0 ? massResult.total_mass_kg / quantity : null);

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

			// Load BOM tree and calculations after entity loads
			await loadBomData(quantity);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load assembly:', e);
		} finally {
			loading = false;
			linksLoading = false;
		}
	}

	async function loadBomData(qty: number = 1) {
		if (!id) return;

		bomLoading = true;
		try {
			const [tree, cost, mass] = await Promise.all([
				assemblies.getBomTree(id, qty),
				assemblies.calculateCost(id, qty),
				assemblies.calculateMass(id, qty)
			]);

			bomTree = tree;
			costResult = cost;
			massResult = mass;

			// Auto-expand root
			if (tree) {
				expandedNodes.add(tree.id);
			}
		} catch (e) {
			console.error('Failed to load BOM data:', e);
		} finally {
			bomLoading = false;
		}
	}

	function formatCurrency(value: number | null | undefined): string {
		if (value == null) return '—';
		return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(value);
	}

	function formatMass(value: number | null | undefined): string {
		if (value == null) return '—';
		if (value < 1) return `${(value * 1000).toFixed(1)} g`;
		return `${value.toFixed(3)} kg`;
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

	function toggleNode(nodeId: string) {
		if (expandedNodes.has(nodeId)) {
			expandedNodes.delete(nodeId);
		} else {
			expandedNodes.add(nodeId);
		}
		expandedNodes = new Set(expandedNodes); // Trigger reactivity
	}

	function handleQuantityChange(e: Event) {
		const input = e.target as HTMLInputElement;
		const val = parseInt(input.value, 10);
		if (!isNaN(val) && val > 0) {
			quantity = val;

			// Debounce BOM data reload to avoid excessive API calls
			if (quantityDebounce) {
				clearTimeout(quantityDebounce);
			}
			quantityDebounce = setTimeout(() => {
				loadBomData(quantity);
			}, 500);
		}
	}

	function startEditingQuantity(componentId: string, currentQuantity: number) {
		editingComponentId = componentId;
		editingQuantityValue = currentQuantity;
	}

	function cancelEditingQuantity() {
		editingComponentId = null;
		editingQuantityValue = 0;
	}

	async function saveComponentQuantity(componentId: string) {
		if (!id || editingQuantityValue <= 0) return;

		savingQuantity = true;
		try {
			await assemblies.updateComponentQuantity(id, componentId, editingQuantityValue);
			editingComponentId = null;
			// Reload BOM data to reflect the change
			await loadBomData(quantity);
		} catch (e) {
			console.error('Failed to update component quantity:', e);
		} finally {
			savingQuantity = false;
		}
	}

	function handleEditQuantityKeydown(e: KeyboardEvent, componentId: string) {
		if (e.key === 'Enter') {
			saveComponentQuantity(componentId);
		} else if (e.key === 'Escape') {
			cancelEditingQuantity();
		}
	}

	// Flatten BOM tree for table display with indentation levels
	interface FlatBomRow {
		node: BomNode;
		level: number;
		hasChildren: boolean;
		isExpanded: boolean;
	}

	function flattenBomTree(node: BomNode, level: number = 0): FlatBomRow[] {
		const rows: FlatBomRow[] = [];
		const hasChildren = node.children && node.children.length > 0;
		const isExpanded = expandedNodes.has(node.id);

		rows.push({ node, level, hasChildren, isExpanded });

		if (hasChildren && isExpanded) {
			for (const child of node.children) {
				rows.push(...flattenBomTree(child, level + 1));
			}
		}

		return rows;
	}

	const flatBomRows = $derived(bomTree ? flattenBomTree(bomTree).slice(1) : []); // Skip root assembly

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

				<!-- BOM Rollup with Cost/Mass -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle class="flex items-center gap-2">
								<Calculator class="h-5 w-5" />
								BOM Cost Rollup
							</CardTitle>
							<div class="flex items-center gap-2">
								<span class="text-sm text-muted-foreground">Quantity:</span>
								<Input
									type="number"
									min="1"
									value={quantity}
									onchange={handleQuantityChange}
									class="w-24"
								/>
							</div>
						</div>
					</CardHeader>
					<CardContent>
						{#if bomLoading}
							<div class="flex h-32 items-center justify-center">
								<div class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
							</div>
						{:else if flatBomRows.length === 0}
							<p class="py-4 text-center text-muted-foreground">No components in BOM</p>
						{:else}
							<!-- Summary Cards -->
							<div class="mb-4 grid gap-4 md:grid-cols-3">
								<div class="rounded-lg border bg-muted/30 p-4">
									<div class="flex items-center gap-2 text-sm text-muted-foreground">
										<DollarSign class="h-4 w-4" />
										Unit Cost
									</div>
									<div class="mt-1 text-2xl font-bold">
										{formatCurrency(unitCost)}
									</div>
									{#if quantity > 1}
										<p class="mt-1 text-xs text-muted-foreground">
											Using price breaks for qty {quantity}
										</p>
									{/if}
								</div>
								<div class="rounded-lg border bg-muted/30 p-4">
									<div class="flex items-center gap-2 text-sm text-muted-foreground">
										<DollarSign class="h-4 w-4" />
										Total Cost (x{quantity})
									</div>
									<div class="mt-1 text-2xl font-bold text-green-500">
										{formatCurrency(costResult?.total_cost)}
									</div>
								</div>
								<div class="rounded-lg border bg-muted/30 p-4">
									<div class="flex items-center gap-2 text-sm text-muted-foreground">
										<Scale class="h-4 w-4" />
										Unit Mass
									</div>
									<div class="mt-1 text-2xl font-bold">
										{formatMass(unitMass)}
									</div>
								</div>
							</div>

							<!-- Warnings -->
							{#if costResult && costResult.missing_cost.length > 0}
								<div class="mb-4 flex items-start gap-2 rounded-lg border border-yellow-500/50 bg-yellow-500/10 p-3">
									<AlertTriangle class="mt-0.5 h-4 w-4 text-yellow-500" />
									<div class="text-sm">
										<span class="font-medium text-yellow-500">
											{costResult.missing_cost.length} component{costResult.missing_cost.length > 1 ? 's' : ''} missing cost data
										</span>
										<p class="mt-1 text-muted-foreground">
											{costResult.missing_cost.slice(0, 5).join(', ')}
											{#if costResult.missing_cost.length > 5}
												<span> and {costResult.missing_cost.length - 5} more</span>
											{/if}
										</p>
									</div>
								</div>
							{/if}

							<!-- BOM Tree Table -->
							<div class="rounded-md border">
								<Table>
									<TableHeader>
										<TableRow>
											<TableHead class="w-[40%]">Item</TableHead>
											<TableHead class="w-20 text-center">Qty</TableHead>
											<TableHead class="w-28 text-right">Unit Cost</TableHead>
											<TableHead class="w-28 text-right">Ext. Cost</TableHead>
											<TableHead class="w-24 text-right">Mass</TableHead>
										</TableRow>
									</TableHeader>
									<TableBody>
										{#each flatBomRows as row (row.node.id + '-' + row.level)}
											<TableRow
												class="cursor-pointer hover:bg-muted/50"
												onclick={(e) => {
													// Don't navigate if we're editing or if click was on an interactive element
													if (editingComponentId) return;

													if (row.hasChildren) {
														toggleNode(row.node.id);
													} else if (row.node.is_assembly) {
														goto(`/assemblies/${row.node.id}`);
													} else {
														goto(`/components/${row.node.id}`);
													}
												}}
											>
												<TableCell>
													<div class="flex items-center" style="padding-left: {row.level * 20}px">
														{#if row.hasChildren}
															<button
																class="mr-2 rounded p-0.5 hover:bg-muted"
																onclick={(e) => {
																	e.stopPropagation();
																	toggleNode(row.node.id);
																}}
															>
																{#if row.isExpanded}
																	<ChevronDown class="h-4 w-4" />
																{:else}
																	<ChevronRight class="h-4 w-4" />
																{/if}
															</button>
														{:else}
															<span class="mr-2 w-5"></span>
														{/if}
														<div>
															<div class="flex items-center gap-2">
																{#if row.node.is_assembly}
																	<Layers class="h-4 w-4 text-blue-500" />
																{:else}
																	<Box class="h-4 w-4 text-muted-foreground" />
																{/if}
																<span class="font-medium">{row.node.title}</span>
															</div>
															<span class="ml-6 text-xs text-muted-foreground font-mono">
																{row.node.part_number}
															</span>
														</div>
													</div>
												</TableCell>
												<TableCell class="text-center" onclick={(e) => e.stopPropagation()}>
													{#if editingComponentId === row.node.id && !row.node.is_assembly}
														<div class="flex items-center justify-center gap-1">
															<Input
																type="number"
																min="1"
																bind:value={editingQuantityValue}
																onkeydown={(e) => handleEditQuantityKeydown(e, row.node.id)}
																onclick={(e) => e.stopPropagation()}
																class="w-16 h-7 text-center text-xs"
																disabled={savingQuantity}
															/>
															<Button
																variant="ghost"
																size="sm"
																class="h-6 w-6 p-0"
																onclick={(e) => { e.stopPropagation(); saveComponentQuantity(row.node.id); }}
																disabled={savingQuantity}
															>
																<span class="text-green-500">&#10003;</span>
															</Button>
															<Button
																variant="ghost"
																size="sm"
																class="h-6 w-6 p-0"
																onclick={(e) => { e.stopPropagation(); cancelEditingQuantity(); }}
																disabled={savingQuantity}
															>
																<span class="text-red-500">&times;</span>
															</Button>
														</div>
													{:else}
														<button
															class="rounded px-2 py-0.5 text-sm border hover:bg-muted/50 transition-colors {row.node.is_assembly ? 'cursor-default' : 'cursor-pointer'}"
															onclick={(e) => {
																e.stopPropagation();
																if (!row.node.is_assembly) {
																	startEditingQuantity(row.node.id, row.node.quantity);
																}
															}}
															disabled={row.node.is_assembly}
															title={row.node.is_assembly ? 'Subassembly quantity' : 'Click to edit quantity'}
														>
															{row.node.quantity}
														</button>
													{/if}
												</TableCell>
												<TableCell class="text-right font-mono text-sm">
													{formatCurrency(row.node.unit_cost)}
												</TableCell>
												<TableCell class="text-right font-mono text-sm font-medium">
													{formatCurrency(row.node.extended_cost)}
												</TableCell>
												<TableCell class="text-right font-mono text-sm">
													{formatMass(row.node.extended_mass)}
												</TableCell>
											</TableRow>
										{/each}
									</TableBody>
								</Table>
							</div>

							<!-- NRE Costs -->
							{#if costResult && costResult.total_nre > 0}
								<div class="mt-4 flex items-center justify-between rounded-lg border bg-muted/30 p-3">
									<span class="text-sm text-muted-foreground">NRE / Tooling Costs</span>
									<span class="font-mono font-medium">{formatCurrency(costResult.total_nre)}</span>
								</div>
							{/if}
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

				<!-- Cost Summary -->
				{#if costResult}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<DollarSign class="h-4 w-4" />
								Cost Summary
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Unit Cost</span>
								<span class="font-mono font-medium">{formatCurrency(unitCost)}</span>
							</div>
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Total (x{quantity})</span>
								<span class="font-mono font-medium text-green-500">{formatCurrency(costResult.total_cost)}</span>
							</div>
							{#if costResult.total_nre > 0}
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">NRE Costs</span>
									<span class="font-mono font-medium">{formatCurrency(costResult.total_nre)}</span>
								</div>
							{/if}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Components w/ Cost</span>
								<Badge variant="default">{costResult.components_with_cost}</Badge>
							</div>
							{#if costResult.components_without_cost > 0}
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">Missing Cost</span>
									<Badge variant="destructive">{costResult.components_without_cost}</Badge>
								</div>
							{/if}
						</CardContent>
					</Card>
				{/if}

				<!-- Mass Summary -->
				{#if massResult}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Scale class="h-4 w-4" />
								Mass Summary
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Unit Mass</span>
								<span class="font-mono font-medium">{formatMass(unitMass)}</span>
							</div>
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Total (x{quantity})</span>
								<span class="font-mono font-medium">{formatMass(massResult.total_mass_kg)}</span>
							</div>
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Components w/ Mass</span>
								<Badge variant="default">{massResult.components_with_mass}</Badge>
							</div>
							{#if massResult.components_without_mass > 0}
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">Missing Mass</span>
									<Badge variant="destructive">{massResult.components_without_mass}</Badge>
								</div>
							{/if}
						</CardContent>
					</Card>
				{/if}

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
