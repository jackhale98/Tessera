<script lang="ts">
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Button, Dialog } from '$lib/components/ui';
	import { StatusBadge, EntityPicker } from '$lib/components/common';
	import { Link2, ArrowRight, ArrowLeft, Plus, Loader2, X } from 'lucide-svelte';
	import { traceability } from '$lib/api';
	import type { LinkInfo } from '$lib/api/tauri';

	interface EntitySearchResult {
		id: string;
		title: string;
		status: string;
		prefix: string;
	}

	interface Props {
		linksFrom: LinkInfo[];
		linksTo: LinkInfo[];
		loading?: boolean;
		entityId?: string;
		onLinksChanged?: () => void;
	}

	let { linksFrom = [], linksTo = [], loading = false, entityId, onLinksChanged }: Props = $props();

	let showAddDialog = $state(false);
	let selectedEntity = $state<EntitySearchResult | null>(null);
	let newLinkType = $state('');
	let adding = $state(false);
	let addError = $state<string | null>(null);
	let selectedTypeFilter = $state('');

	// Remove link state
	let showRemoveDialog = $state(false);
	let linkToRemove = $state<{ sourceId: string; targetId: string; linkType: string; title: string } | null>(null);
	let removing = $state(false);
	let removeError = $state<string | null>(null);

	// REQ→REQ link type options (only shown when both source and target are REQ)
	const reqToReqLinkTypes = [
		{ value: 'derives_from', label: 'Derives From', description: 'This requirement derives from the target' },
		{ value: 'allocated_to', label: 'Allocated To', description: 'This requirement is allocated to the target' },
		{ value: 'related_to', label: 'Related To', description: 'General relationship' }
	];

	// All entity types for the picker — auto-inference means we search everything
	const allEntityTypes = ['REQ', 'RISK', 'HAZ', 'TEST', 'RSLT', 'CMP', 'ASM', 'FEAT', 'MATE', 'TOL', 'PROC', 'CTRL', 'WORK', 'LOT', 'DEV', 'NCR', 'CAPA', 'QUOT', 'SUP'];

	const entityTypeLabels: Record<string, string> = {
		REQ: 'Requirements', RISK: 'Risks', HAZ: 'Hazards', TEST: 'Tests', RSLT: 'Results',
		CMP: 'Components', ASM: 'Assemblies', FEAT: 'Features', MATE: 'Mates', TOL: 'Stackups',
		PROC: 'Processes', CTRL: 'Controls', WORK: 'Work Instructions', LOT: 'Lots',
		DEV: 'Deviations', NCR: 'NCRs', CAPA: 'CAPAs', QUOT: 'Quotes', SUP: 'Suppliers'
	};

	const filteredEntityTypes = $derived(
		selectedTypeFilter ? [selectedTypeFilter] : allEntityTypes
	);
	const pickerPlaceholder = $derived(
		selectedTypeFilter ? `Search ${entityTypeLabels[selectedTypeFilter] ?? selectedTypeFilter}...` : 'Search all entities...'
	);

	// Detect REQ→REQ case: source is REQ and selected target is REQ
	const sourcePrefix = $derived(entityId?.split('-')[0]?.toUpperCase() ?? '');
	const targetPrefix = $derived(selectedEntity?.prefix?.toUpperCase() ?? '');
	const isReqToReq = $derived(sourcePrefix === 'REQ' && targetPrefix === 'REQ');
	const needsLinkTypeChoice = $derived(isReqToReq && selectedEntity !== null);

	async function handleAddLink() {
		if (!entityId || !selectedEntity) return;

		// For REQ→REQ, require explicit link type selection
		if (isReqToReq && !newLinkType) {
			addError = 'Please select a link type for REQ→REQ links';
			return;
		}

		adding = true;
		addError = null;

		try {
			// Pass undefined for auto-inference, or explicit type for REQ→REQ
			const linkType = isReqToReq ? newLinkType : undefined;
			await traceability.addLink(entityId, selectedEntity.id, linkType);
			showAddDialog = false;
			selectedEntity = null;
			newLinkType = '';
			onLinksChanged?.();
		} catch (e) {
			addError = e instanceof Error ? e.message : 'Failed to add link';
		} finally {
			adding = false;
		}
	}

	function handleEntitySelect(entity: EntitySearchResult) {
		selectedEntity = entity;
		// Reset link type when entity changes
		newLinkType = '';
	}

	function handleEntityClear() {
		selectedEntity = null;
		newLinkType = '';
	}

	function handleTypeFilterChange(e: Event) {
		selectedTypeFilter = (e.target as HTMLSelectElement).value;
		// Clear selected entity when type filter changes
		selectedEntity = null;
		newLinkType = '';
	}

	function initiateRemoveLink(sourceId: string, targetId: string, linkType: string, title: string) {
		linkToRemove = { sourceId, targetId, linkType, title };
		removeError = null;
		showRemoveDialog = true;
	}

	async function handleRemoveLink() {
		if (!linkToRemove) return;

		removing = true;
		removeError = null;

		try {
			await traceability.removeLink(linkToRemove.sourceId, linkToRemove.targetId, linkToRemove.linkType);
			showRemoveDialog = false;
			linkToRemove = null;
			onLinksChanged?.();
		} catch (e) {
			removeError = e instanceof Error ? e.message : 'Failed to remove link';
		} finally {
			removing = false;
		}
	}

	function cancelRemoveLink() {
		showRemoveDialog = false;
		linkToRemove = null;
		removeError = null;
	}

	function getEntityRoute(id: string): string {
		const prefix = id.split('-')[0]?.toUpperCase();
		const routeMap: Record<string, string> = {
			REQ: '/requirements',
			RISK: '/risks',
			HAZ: '/hazards',
			TEST: '/verification/tests',
			RSLT: '/verification/results',
			CMP: '/components',
			ASM: '/assemblies',
			FEAT: '/features',
			MATE: '/mates',
			TOL: '/tolerances',
			PROC: '/manufacturing/processes',
			CTRL: '/controls',
			WORK: '/manufacturing/work-instructions',
			LOT: '/manufacturing/lots',
			DEV: '/manufacturing/deviations',
			NCR: '/quality/ncrs',
			CAPA: '/quality/capas',
			QUOT: '/procurement/quotes',
			SUP: '/procurement/suppliers'
		};
		return `${routeMap[prefix] ?? '/entities'}/${id}`;
	}

	function formatLinkType(linkType: string): string {
		return linkType.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase());
	}

	// Group links by type
	const groupedLinksFrom = $derived(
		linksFrom.reduce(
			(acc, link) => {
				const type = link.link_type || 'related';
				if (!acc[type]) acc[type] = [];
				acc[type].push(link);
				return acc;
			},
			{} as Record<string, LinkInfo[]>
		)
	);

	const groupedLinksTo = $derived(
		linksTo.reduce(
			(acc, link) => {
				const type = link.link_type || 'related';
				if (!acc[type]) acc[type] = [];
				acc[type].push(link);
				return acc;
			},
			{} as Record<string, LinkInfo[]>
		)
	);

	// Deduplicate: filter out incoming links that are reciprocals of outgoing links
	// If entityId→otherId exists as outgoing, otherId→entityId incoming is redundant
	const outgoingTargets = $derived(new Set(linksFrom.map((l) => l.target_id)));
	const dedupedLinksTo = $derived(linksTo.filter((l) => !outgoingTargets.has(l.source_id)));

	const groupedDedupedLinksTo = $derived(
		dedupedLinksTo.reduce(
			(acc, link) => {
				const type = link.link_type || 'related';
				if (!acc[type]) acc[type] = [];
				acc[type].push(link);
				return acc;
			},
			{} as Record<string, LinkInfo[]>
		)
	);

	const hasLinks = $derived(linksFrom.length > 0 || dedupedLinksTo.length > 0);
</script>

<Card>
	<CardHeader class="flex flex-row items-center justify-between">
		<CardTitle class="flex items-center gap-2">
			<Link2 class="h-5 w-5" />
			Links & Traceability
		</CardTitle>
		{#if entityId}
			<Button variant="outline" size="sm" onclick={() => showAddDialog = true}>
				<Plus class="h-4 w-4 mr-1" />
				Add Link
			</Button>
		{/if}
	</CardHeader>
	<CardContent>
		{#if loading}
			<div class="flex items-center justify-center py-8">
				<div class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
			</div>
		{:else if !hasLinks}
			<div class="flex flex-col items-center justify-center py-8 text-center">
				<Link2 class="h-12 w-12 text-muted-foreground/30 mb-3" />
				<p class="text-sm text-muted-foreground">No links defined</p>
				{#if entityId}
					<p class="text-xs text-muted-foreground/70 mt-1">
						Click "Add Link" to connect this entity to tests, controls, or other entities
					</p>
				{/if}
			</div>
		{:else}
			<div class="space-y-6">
				<!-- Outgoing links (from this entity) -->
				{#if linksFrom.length > 0}
					<div class="space-y-3">
						<h4 class="flex items-center gap-2 text-sm font-medium text-muted-foreground">
							<ArrowRight class="h-4 w-4" />
							Outgoing Links ({linksFrom.length})
						</h4>
						{#each Object.entries(groupedLinksFrom) as [linkType, links]}
							<div class="space-y-2">
								<p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
									{formatLinkType(linkType)}
								</p>
								<div class="space-y-1">
									{#each links as link}
										<div class="flex items-center gap-1">
											<button
												class="flex flex-1 items-center justify-between rounded-lg border p-3 text-left transition-colors hover:bg-muted/50"
												onclick={() => goto(getEntityRoute(link.target_id))}
											>
												<div class="min-w-0 flex-1">
													<p class="truncate font-medium">
														{link.target_title || link.target_id}
													</p>
													<p class="font-mono text-xs text-muted-foreground">
														{link.target_id}
													</p>
												</div>
												{#if link.target_status}
													<StatusBadge status={link.target_status} class="ml-2" />
												{/if}
											</button>
											{#if entityId}
												<button
													class="p-2 rounded-lg border text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
													title="Remove link"
													onclick={() => initiateRemoveLink(link.source_id, link.target_id, link.link_type, link.target_title || link.target_id)}
												>
													<X class="h-4 w-4" />
												</button>
											{/if}
										</div>
									{/each}
								</div>
							</div>
						{/each}
					</div>
				{/if}

				<!-- Incoming links (to this entity), excluding reciprocals of outgoing -->
				{#if dedupedLinksTo.length > 0}
					<div class="space-y-3">
						<h4 class="flex items-center gap-2 text-sm font-medium text-muted-foreground">
							<ArrowLeft class="h-4 w-4" />
							Incoming Links ({dedupedLinksTo.length})
						</h4>
						{#each Object.entries(groupedDedupedLinksTo) as [linkType, links]}
							<div class="space-y-2">
								<p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
									{formatLinkType(linkType)}
								</p>
								<div class="space-y-1">
									{#each links as link}
										<div class="flex items-center gap-1">
											<button
												class="flex flex-1 items-center justify-between rounded-lg border p-3 text-left transition-colors hover:bg-muted/50"
												onclick={() => goto(getEntityRoute(link.source_id))}
											>
												<div class="min-w-0 flex-1">
													<p class="truncate font-medium">
														{link.target_title || link.source_id}
													</p>
													<p class="font-mono text-xs text-muted-foreground">
														{link.source_id}
													</p>
												</div>
												{#if link.target_status}
													<StatusBadge status={link.target_status} class="ml-2" />
												{/if}
											</button>
											{#if entityId}
												<button
													class="p-2 rounded-lg border text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
													title="Remove link"
													onclick={() => initiateRemoveLink(link.source_id, link.target_id, link.link_type, link.target_title || link.source_id)}
												>
													<X class="h-4 w-4" />
												</button>
											{/if}
										</div>
									{/each}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	</CardContent>
</Card>

<!-- Add Link Dialog -->
{#if entityId}
	<Dialog bind:open={showAddDialog}>
		<div class="p-6">
			<div class="mb-4">
				<h2 class="text-lg font-semibold">Add Link</h2>
				<p class="text-sm text-muted-foreground mt-1">
					Search for an entity to link. The link type will be automatically determined.
				</p>
			</div>
			<div class="space-y-4">
				<div class="space-y-2">
					<label for="entity-type-filter" class="text-sm font-medium">Entity Type</label>
					<select
						id="entity-type-filter"
						value={selectedTypeFilter}
						onchange={handleTypeFilterChange}
						class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
					>
						<option value="">All Types</option>
						{#each allEntityTypes as type}
							<option value={type}>{entityTypeLabels[type] ?? type} ({type})</option>
						{/each}
					</select>
				</div>
				<div class="space-y-2">
					{#key selectedTypeFilter}
						<EntityPicker
							label="Target Entity"
							entityTypes={filteredEntityTypes}
							placeholder={pickerPlaceholder}
							onSelect={handleEntitySelect}
							onClear={handleEntityClear}
						/>
					{/key}
				</div>
				{#if needsLinkTypeChoice}
					<div class="space-y-2">
						<label for="req-link-type" class="text-sm font-medium">Link Type (REQ → REQ)</label>
						<select
							id="req-link-type"
							value={newLinkType}
							onchange={(e) => { newLinkType = (e.target as HTMLSelectElement).value; }}
							class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
						>
							<option value="">Select link type...</option>
							{#each reqToReqLinkTypes as option}
								<option value={option.value}>{option.label}</option>
							{/each}
						</select>
						<p class="text-xs text-muted-foreground">
							{reqToReqLinkTypes.find((opt) => opt.value === newLinkType)?.description ?? 'Requirement-to-requirement links require an explicit type'}
						</p>
					</div>
				{/if}
				{#if addError}
					<p class="text-sm text-destructive">{addError}</p>
				{/if}
			</div>
			<div class="mt-6 flex justify-end gap-2">
				<Button variant="outline" onclick={() => { showAddDialog = false; selectedEntity = null; newLinkType = ''; selectedTypeFilter = ''; }}>Cancel</Button>
				<Button onclick={handleAddLink} disabled={adding || !selectedEntity || (isReqToReq && !newLinkType)}>
					{#if adding}
						<Loader2 class="h-4 w-4 mr-2 animate-spin" />
					{/if}
					Add Link
				</Button>
			</div>
		</div>
	</Dialog>
{/if}

<!-- Remove Link Confirmation Dialog -->
<Dialog bind:open={showRemoveDialog}>
	<div class="p-6">
		<div class="mb-4">
			<h2 class="text-lg font-semibold">Remove Link</h2>
			<p class="text-sm text-muted-foreground mt-1">
				Are you sure you want to remove this link?
			</p>
		</div>
		{#if linkToRemove}
			<div class="rounded-lg border p-4 mb-4 bg-muted/50">
				<p class="font-medium">{linkToRemove.title}</p>
				<p class="text-sm text-muted-foreground mt-1">
					Link type: <span class="font-medium">{formatLinkType(linkToRemove.linkType)}</span>
				</p>
			</div>
			<p class="text-sm text-muted-foreground mb-4">
				This will also remove the reciprocal link from the target entity.
			</p>
		{/if}
		{#if removeError}
			<p class="text-sm text-destructive mb-4">{removeError}</p>
		{/if}
		<div class="flex justify-end gap-2">
			<Button variant="outline" onclick={cancelRemoveLink}>Cancel</Button>
			<Button variant="destructive" onclick={handleRemoveLink} disabled={removing}>
				{#if removing}
					<Loader2 class="h-4 w-4 mr-2 animate-spin" />
				{/if}
				Remove Link
			</Button>
		</div>
	</div>
</Dialog>
