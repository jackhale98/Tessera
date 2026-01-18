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
	let newLinkType = $state('verified_by');
	let adding = $state(false);
	let addError = $state<string | null>(null);

	// Remove link state
	let showRemoveDialog = $state(false);
	let linkToRemove = $state<{ sourceId: string; targetId: string; linkType: string; title: string } | null>(null);
	let removing = $state(false);
	let removeError = $state<string | null>(null);

	// Link type options with their target entity types
	const linkTypeOptions = [
		{ value: 'verified_by', label: 'Verified By', entityTypes: ['TEST', 'RSLT'], description: 'Tests that verify this entity' },
		{ value: 'mitigated_by', label: 'Mitigated By', entityTypes: ['CTRL', 'CMP'], description: 'Controls or components that mitigate risk' },
		{ value: 'related_to', label: 'Related To', entityTypes: ['REQ', 'RISK', 'CMP', 'ASM', 'TEST', 'HAZ', 'FEAT', 'PROC', 'CTRL'], description: 'General relationship' },
		{ value: 'affects', label: 'Affects', entityTypes: ['FEAT', 'CMP', 'ASM', 'REQ'], description: 'Entities affected by this one' },
		{ value: 'satisfied_by', label: 'Satisfied By', entityTypes: ['CMP', 'ASM', 'FEAT'], description: 'Components that satisfy requirements' },
		{ value: 'allocated_to', label: 'Allocated To', entityTypes: ['CMP', 'ASM'], description: 'Components this is allocated to' }
	];

	// Get entity types for the currently selected link type
	const currentEntityTypes = $derived(
		linkTypeOptions.find((opt) => opt.value === newLinkType)?.entityTypes ?? ['REQ', 'RISK', 'TEST', 'CMP']
	);

	async function handleAddLink() {
		if (!entityId || !selectedEntity) return;

		adding = true;
		addError = null;

		try {
			await traceability.addLink(entityId, selectedEntity.id, newLinkType);
			showAddDialog = false;
			selectedEntity = null;
			newLinkType = 'verified_by';
			onLinksChanged?.();
		} catch (e) {
			addError = e instanceof Error ? e.message : 'Failed to add link';
		} finally {
			adding = false;
		}
	}

	function handleEntitySelect(entity: EntitySearchResult) {
		selectedEntity = entity;
	}

	function handleEntityClear() {
		selectedEntity = null;
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

	function handleLinkTypeChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		newLinkType = target.value;
		// Clear selected entity when link type changes since entity types change
		selectedEntity = null;
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

	const hasLinks = $derived(linksFrom.length > 0 || linksTo.length > 0);
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

				<!-- Incoming links (to this entity) -->
				{#if linksTo.length > 0}
					<div class="space-y-3">
						<h4 class="flex items-center gap-2 text-sm font-medium text-muted-foreground">
							<ArrowLeft class="h-4 w-4" />
							Incoming Links ({linksTo.length})
						</h4>
						{#each Object.entries(groupedLinksTo) as [linkType, links]}
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
					Connect this entity to another entity in the project
				</p>
			</div>
			<div class="space-y-4">
				<div class="space-y-2">
					<label for="link-type" class="text-sm font-medium">Link Type</label>
					<select
						id="link-type"
						value={newLinkType}
						onchange={handleLinkTypeChange}
						class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
					>
						{#each linkTypeOptions as option}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>
					<p class="text-xs text-muted-foreground">
						{linkTypeOptions.find((opt) => opt.value === newLinkType)?.description ?? ''}
					</p>
				</div>
				<div class="space-y-2">
					<EntityPicker
						label="Target Entity"
						entityTypes={currentEntityTypes}
						placeholder="Search {currentEntityTypes.join(', ')} entities..."
						onSelect={handleEntitySelect}
						onClear={handleEntityClear}
					/>
				</div>
				{#if addError}
					<p class="text-sm text-destructive">{addError}</p>
				{/if}
			</div>
			<div class="mt-6 flex justify-end gap-2">
				<Button variant="outline" onclick={() => { showAddDialog = false; selectedEntity = null; }}>Cancel</Button>
				<Button onclick={handleAddLink} disabled={adding || !selectedEntity}>
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
