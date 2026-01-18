<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge, Button, Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter, Input, Label, Select, SelectTrigger, SelectValue, SelectContent, SelectItem, Textarea } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { ncrs, traceability } from '$lib/api/tauri';
	import type { LinkInfo } from '$lib/api/tauri';
	import EntityHistory from '$lib/components/EntityHistory.svelte';
	import {
		AlertOctagon,
		User,
		Calendar,
		Tag,
		FileText,
		ClipboardCheck,
		AlertTriangle,
		ChevronRight,
		X,
		DollarSign,
		History
	} from 'lucide-svelte';

	const id = $derived($page.params.id);

	let entity = $state<Record<string, unknown> | null>(null);
	let linksFrom = $state<LinkInfo[]>([]);
	let linksTo = $state<LinkInfo[]>([]);
	let loading = $state(true);
	let linksLoading = $state(true);
	let error = $state<string | null>(null);
	let actionInProgress = $state(false);

	// Modal states
	let showCloseModal = $state(false);
	let showCostModal = $state(false);

	// Close NCR form data
	let closeDecision = $state('');
	let closeDecisionMaker = $state('');
	let closeJustification = $state('');
	let closeMrbRequired = $state(false);

	// Cost form data
	let reworkCost = $state(0);
	let scrapCost = $state(0);

	// Track if we've loaded for this ID to prevent double-loads
	let loadedId = $state<string | null>(null);

	// Type-safe data access
	const ncrType = $derived((entity?.ncr_type as string) ?? 'internal');
	const ncrStatus = $derived((entity?.ncr_status as string) ?? 'open');
	const severity = $derived((entity?.severity as string) ?? 'minor');
	const category = $derived((entity?.category as string) ?? null);
	const description = $derived((entity?.description as string) ?? '');
	const disposition = $derived((entity?.disposition as string) ?? null);
	const dispositionRationale = $derived((entity?.disposition_rationale as string) ?? null);
	const affectedQuantity = $derived(entity?.affected_quantity as number | null);
	const revision = $derived((entity?.entity_revision as number) ?? 1);
	const entityStatus = $derived((entity?.status as string) ?? 'draft');
	const entityTitle = $derived((entity?.title as string) ?? '');
	const entityAuthor = $derived((entity?.author as string) ?? '');
	const entityCreated = $derived((entity?.created as string) ?? '');
	const entityTags = $derived((entity?.tags as string[]) ?? []);
	const containmentActions = $derived((entity?.containment_actions as string[]) ?? []);
	const detectionMethod = $derived((entity?.detection_method as string) ?? null);
	const reworkCostValue = $derived((entity?.rework_cost as number) ?? 0);
	const scrapCostValue = $derived((entity?.scrap_cost as number) ?? 0);
	const totalCost = $derived(reworkCostValue + scrapCostValue);

	async function loadData() {
		if (!id) return;

		loading = true;
		linksLoading = true;
		error = null;

		try {
			const [entityResult, fromLinks, toLinks] = await Promise.all([
				ncrs.get(id),
				traceability.getLinksFrom(id),
				traceability.getLinksTo(id)
			]);

			entity = entityResult as Record<string, unknown>;
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
			internal: 'Internal',
			supplier: 'Supplier',
			customer: 'Customer'
		};
		return types[type.toLowerCase()] ?? type;
	}

	function formatNcrStatus(status: string): string {
		const statuses: Record<string, string> = {
			open: 'Open',
			containment: 'Containment',
			investigation: 'Investigation',
			disposition: 'Disposition',
			closed: 'Closed'
		};
		return statuses[status.toLowerCase()] ?? status;
	}

	function getNcrStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			open: 'destructive',
			containment: 'secondary',
			investigation: 'secondary',
			disposition: 'outline',
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

	function getNextStatusLabel(status: string): string {
		const labels: Record<string, string> = {
			open: 'Start Containment',
			containment: 'Start Investigation',
			investigation: 'Proceed to Disposition',
			disposition: 'Close NCR'
		};
		return labels[status.toLowerCase()] ?? 'Advance Status';
	}

	// Workflow Actions
	async function handleAdvanceStatus() {
		if (ncrStatus === 'disposition') {
			showCloseModal = true;
			return;
		}

		actionInProgress = true;
		try {
			await ncrs.advanceStatus(id);
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			actionInProgress = false;
		}
	}

	async function handleClose() {
		if (!closeDecision || !closeDecisionMaker) return;

		actionInProgress = true;
		try {
			await ncrs.close(id, {
				decision: closeDecision,
				decision_maker: closeDecisionMaker,
				justification: closeJustification || undefined,
				mrb_required: closeMrbRequired
			});
			showCloseModal = false;
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			actionInProgress = false;
		}
	}

	async function handleSetCost() {
		actionInProgress = true;
		try {
			await ncrs.setCost(id, reworkCost, scrapCost);
			showCostModal = false;
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			actionInProgress = false;
		}
	}

	function openCostModal() {
		reworkCost = reworkCostValue;
		scrapCost = scrapCostValue;
		showCostModal = true;
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
			id={id}
			title={entityTitle}
			status={entityStatus}
			subtitle="Non-Conformance Report"
			backHref="/quality/ncrs"
			backLabel="NCRs"
			onEdit={() => goto(`/quality/ncrs/${id}/edit`)}
		/>

		<!-- Workflow Actions -->
		{#if ncrStatus !== 'closed'}
			<Card>
				<CardHeader>
					<CardTitle>Workflow Actions</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="flex flex-wrap gap-3">
						<Button
							variant="default"
							onclick={handleAdvanceStatus}
							disabled={actionInProgress}
						>
							<ChevronRight class="mr-2 h-4 w-4" />
							{getNextStatusLabel(ncrStatus)}
						</Button>

						<Button
							variant="outline"
							onclick={openCostModal}
							disabled={actionInProgress}
						>
							<DollarSign class="mr-2 h-4 w-4" />
							Set Costs
						</Button>
					</div>

					<!-- Workflow Status Display -->
					<div class="mt-4 flex items-center gap-2 text-sm text-muted-foreground">
						<span class={ncrStatus === 'open' ? 'font-bold text-foreground' : ''}>Open</span>
						<ChevronRight class="h-4 w-4" />
						<span class={ncrStatus === 'containment' ? 'font-bold text-foreground' : ''}>Containment</span>
						<ChevronRight class="h-4 w-4" />
						<span class={ncrStatus === 'investigation' ? 'font-bold text-foreground' : ''}>Investigation</span>
						<ChevronRight class="h-4 w-4" />
						<span class={ncrStatus === 'disposition' ? 'font-bold text-foreground' : ''}>Disposition</span>
						<ChevronRight class="h-4 w-4" />
						<span class={ncrStatus === 'closed' ? 'font-bold text-foreground' : ''}>Closed</span>
					</div>
				</CardContent>
			</Card>
		{/if}

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

				<!-- Containment Actions -->
				{#if containmentActions.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<AlertTriangle class="h-5 w-5" />
								Containment Actions
							</CardTitle>
						</CardHeader>
						<CardContent>
							<ul class="list-disc list-inside space-y-1">
								{#each containmentActions as action}
									<li>{action}</li>
								{/each}
							</ul>
						</CardContent>
					</Card>
				{/if}

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

				<!-- Cost Information -->
				{#if totalCost > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<DollarSign class="h-5 w-5" />
								Cost Impact
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="grid grid-cols-3 gap-4">
								<div>
									<p class="text-sm text-muted-foreground">Rework Cost</p>
									<p class="text-xl font-bold">${reworkCostValue.toLocaleString()}</p>
								</div>
								<div>
									<p class="text-sm text-muted-foreground">Scrap Cost</p>
									<p class="text-xl font-bold">${scrapCostValue.toLocaleString()}</p>
								</div>
								<div>
									<p class="text-sm text-muted-foreground">Total Cost</p>
									<p class="text-xl font-bold text-red-500">${totalCost.toLocaleString()}</p>
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
					entityId={id}
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
						<EntityHistory entityId={id} />
					</CardContent>
				</Card>
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
								<span class="text-sm font-medium capitalize">{category}</span>
							</div>
						{/if}
						{#if detectionMethod}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Detection</span>
								<span class="text-sm font-medium">{detectionMethod}</span>
							</div>
						{/if}
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entityStatus} />
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
							<span class="ml-auto text-sm font-medium">{entityAuthor}</span>
						</div>
						<div class="flex items-center gap-2">
							<Calendar class="h-4 w-4 text-muted-foreground" />
							<span class="text-sm text-muted-foreground">Created</span>
							<span class="ml-auto text-sm font-medium">{formatDate(entityCreated)}</span>
						</div>
					</CardContent>
				</Card>

				<!-- Tags -->
				{#if entityTags && entityTags.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Tag class="h-4 w-4" />
								Tags
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="flex flex-wrap gap-2">
								{#each entityTags as tag}
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

<!-- Close NCR Modal -->
<Dialog bind:open={showCloseModal}>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Close NCR</DialogTitle>
			<DialogDescription>
				Record the disposition decision and close this non-conformance report.
			</DialogDescription>
		</DialogHeader>
		<div class="space-y-4 py-4">
			<div class="space-y-2">
				<Label for="decision">Disposition Decision</Label>
				<Select bind:value={closeDecision}>
					<SelectTrigger>
						<SelectValue placeholder="Select disposition..." />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="use_as_is">Use As Is</SelectItem>
						<SelectItem value="rework">Rework</SelectItem>
						<SelectItem value="repair">Repair</SelectItem>
						<SelectItem value="scrap">Scrap</SelectItem>
						<SelectItem value="return_to_supplier">Return to Supplier</SelectItem>
					</SelectContent>
				</Select>
			</div>
			<div class="space-y-2">
				<Label for="decision_maker">Decision Maker</Label>
				<Input
					id="decision_maker"
					bind:value={closeDecisionMaker}
					placeholder="Enter name of decision maker"
				/>
			</div>
			<div class="space-y-2">
				<Label for="justification">Justification (optional)</Label>
				<Textarea
					id="justification"
					bind:value={closeJustification}
					placeholder="Enter justification for this disposition..."
					rows={3}
				/>
			</div>
			<div class="flex items-center gap-2">
				<input
					type="checkbox"
					id="mrb_required"
					bind:checked={closeMrbRequired}
					class="h-4 w-4 rounded border-gray-300"
				/>
				<Label for="mrb_required">MRB Review Required</Label>
			</div>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => { showCloseModal = false; }}>Cancel</Button>
			<Button
				onclick={handleClose}
				disabled={!closeDecision || !closeDecisionMaker || actionInProgress}
			>
				Close NCR
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>

<!-- Set Cost Modal -->
<Dialog bind:open={showCostModal}>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Set NCR Costs</DialogTitle>
			<DialogDescription>
				Record the cost impact of this non-conformance.
			</DialogDescription>
		</DialogHeader>
		<div class="space-y-4 py-4">
			<div class="space-y-2">
				<Label for="rework_cost">Rework Cost ($)</Label>
				<Input
					id="rework_cost"
					type="number"
					bind:value={reworkCost}
					min={0}
					step={0.01}
				/>
			</div>
			<div class="space-y-2">
				<Label for="scrap_cost">Scrap Cost ($)</Label>
				<Input
					id="scrap_cost"
					type="number"
					bind:value={scrapCost}
					min={0}
					step={0.01}
				/>
			</div>
			<div class="pt-2 border-t">
				<div class="flex justify-between text-sm">
					<span class="text-muted-foreground">Total Cost:</span>
					<span class="font-bold">${(reworkCost + scrapCost).toLocaleString()}</span>
				</div>
			</div>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => { showCostModal = false; }}>Cancel</Button>
			<Button onclick={handleSetCost} disabled={actionInProgress}>
				Save Costs
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
