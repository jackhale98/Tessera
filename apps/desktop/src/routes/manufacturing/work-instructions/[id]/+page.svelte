<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { entities, traceability, workInstructions } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import EntityHistory from '$lib/components/EntityHistory.svelte';
	import {
		ClipboardList,
		User,
		Calendar,
		Tag,
		FileText,
		ListOrdered,
		AlertCircle,
		Wrench,
		History,
		Plus,
		Trash2,
		Shield,
		CheckSquare
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
	const description = $derived((data.description as string) ?? '');
	const revision = $derived((data.revision as number) ?? 1);
	const equipment = $derived((data.equipment as string[]) ?? []);
	const materials = $derived((data.materials as string[]) ?? []);
	const safetyNotes = $derived((data.safety_notes as string[]) ?? []);

	interface Step {
		number: number;
		instruction: string;
		notes?: string;
		critical?: boolean;
	}
	const steps = $derived((data.steps as Step[]) ?? []);

	interface ToolItem {
		name: string;
		part_number?: string;
	}
	const tools = $derived((data.tools as ToolItem[]) ?? []);

	interface MaterialItem {
		name: string;
		specification?: string;
	}
	const structuredMaterials = $derived((data.structured_materials as MaterialItem[]) ?? (data.materials_list as MaterialItem[]) ?? []);

	interface QualityCheckItem {
		at_step: number;
		characteristic: string;
		specification?: string;
	}
	const qualityChecks = $derived((data.quality_checks as QualityCheckItem[]) ?? []);

	interface SafetyData {
		ppe_required?: Array<{ item: string; specification?: string }>;
		hazards?: Array<{ hazard: string; mitigation?: string }>;
	}
	const safety = $derived((data.safety as SafetyData) ?? null);

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
			console.error('Failed to load work instruction:', e);
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

	// Step management state
	let showAddStep = $state(false);
	let newStepAction = $state('');
	let newStepVerification = $state('');
	let newStepCaution = $state('');
	let actionInProgress = $state(false);

	// Tool management state
	let showAddTool = $state(false);
	let newToolName = $state('');
	let newToolPartNumber = $state('');

	// Material management state
	let showAddMaterial = $state(false);
	let newMaterialName = $state('');
	let newMaterialSpec = $state('');

	// Quality check state
	let showAddQC = $state(false);
	let newQCStep = $state(1);
	let newQCCharacteristic = $state('');
	let newQCSpec = $state('');

	// Safety state
	let showSetSafety = $state(false);
	let newPpeItem = $state('');
	let newHazard = $state('');

	async function addStep() {
		if (!id || !newStepAction.trim()) return;
		actionInProgress = true;
		try {
			await workInstructions.addStep(id, {
				action: newStepAction.trim(),
				verification: newStepVerification.trim() || undefined,
				caution: newStepCaution.trim() || undefined
			});
			newStepAction = '';
			newStepVerification = '';
			newStepCaution = '';
			showAddStep = false;
			loadedId = null; // Force reload
			loadData();
		} catch (e) {
			console.error('Failed to add step:', e);
		} finally {
			actionInProgress = false;
		}
	}

	async function removeStep(stepNumber: number) {
		if (!id) return;
		actionInProgress = true;
		try {
			await workInstructions.removeStep(id, stepNumber);
			loadedId = null;
			loadData();
		} catch (e) {
			console.error('Failed to remove step:', e);
		} finally {
			actionInProgress = false;
		}
	}

	async function addTool() {
		if (!id || !newToolName.trim()) return;
		actionInProgress = true;
		try {
			await workInstructions.addTool(id, {
				name: newToolName.trim(),
				part_number: newToolPartNumber.trim() || undefined
			});
			newToolName = '';
			newToolPartNumber = '';
			showAddTool = false;
			loadedId = null;
			loadData();
		} catch (e) {
			console.error('Failed to add tool:', e);
		} finally {
			actionInProgress = false;
		}
	}

	async function removeTool(toolName: string) {
		if (!id) return;
		actionInProgress = true;
		try {
			await workInstructions.removeTool(id, toolName);
			loadedId = null;
			loadData();
		} catch (e) {
			console.error('Failed to remove tool:', e);
		} finally {
			actionInProgress = false;
		}
	}

	async function addMaterial() {
		if (!id || !newMaterialName.trim()) return;
		actionInProgress = true;
		try {
			await workInstructions.addMaterial(id, {
				name: newMaterialName.trim(),
				specification: newMaterialSpec.trim() || undefined
			});
			newMaterialName = '';
			newMaterialSpec = '';
			showAddMaterial = false;
			loadedId = null;
			loadData();
		} catch (e) {
			console.error('Failed to add material:', e);
		} finally {
			actionInProgress = false;
		}
	}

	async function removeMaterial(materialName: string) {
		if (!id) return;
		actionInProgress = true;
		try {
			await workInstructions.removeMaterial(id, materialName);
			loadedId = null;
			loadData();
		} catch (e) {
			console.error('Failed to remove material:', e);
		} finally {
			actionInProgress = false;
		}
	}

	async function addQualityCheck() {
		if (!id || !newQCCharacteristic.trim()) return;
		actionInProgress = true;
		try {
			await workInstructions.addQualityCheck(id, {
				at_step: newQCStep,
				characteristic: newQCCharacteristic.trim(),
				specification: newQCSpec.trim() || undefined
			});
			newQCCharacteristic = '';
			newQCSpec = '';
			showAddQC = false;
			loadedId = null;
			loadData();
		} catch (e) {
			console.error('Failed to add quality check:', e);
		} finally {
			actionInProgress = false;
		}
	}

	async function removeQualityCheck(atStep: number) {
		if (!id) return;
		actionInProgress = true;
		try {
			await workInstructions.removeQualityCheck(id, atStep);
			loadedId = null;
			loadData();
		} catch (e) {
			console.error('Failed to remove quality check:', e);
		} finally {
			actionInProgress = false;
		}
	}

	async function clearSafety() {
		if (!id) return;
		actionInProgress = true;
		try {
			await workInstructions.clearSafety(id);
			loadedId = null;
			loadData();
		} catch (e) {
			console.error('Failed to clear safety:', e);
		} finally {
			actionInProgress = false;
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
			subtitle="Work Instruction"
			backHref="/manufacturing/work-instructions"
			backLabel="Work Instructions"
			onEdit={() => goto(`/manufacturing/work-instructions/${id}/edit`)}
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

				<!-- Safety Notes -->
				{#if safetyNotes.length > 0}
					<Card class="border-orange-500/50">
						<CardHeader>
							<CardTitle class="flex items-center gap-2 text-orange-500">
								<AlertCircle class="h-5 w-5" />
								Safety Notes
							</CardTitle>
						</CardHeader>
						<CardContent>
							<ul class="list-inside list-disc space-y-2">
								{#each safetyNotes as note}
									<li class="text-orange-600 dark:text-orange-400">{note}</li>
								{/each}
							</ul>
						</CardContent>
					</Card>
				{/if}

				<!-- Steps -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle class="flex items-center gap-2">
								<ListOrdered class="h-5 w-5" />
								Procedure Steps ({steps.length})
							</CardTitle>
							<button
								class="inline-flex items-center gap-1 rounded-md border px-3 py-1.5 text-sm hover:bg-accent"
								onclick={() => (showAddStep = !showAddStep)}
							>
								<Plus class="h-4 w-4" />
								Add Step
							</button>
						</div>
					</CardHeader>
						<CardContent>
							{#if showAddStep}
								<div class="mb-4 rounded-lg border border-dashed p-4 space-y-3">
									<input type="text" placeholder="Step action (required)" bind:value={newStepAction} class="w-full rounded-md border px-3 py-2 text-sm" />
									<input type="text" placeholder="Verification (optional)" bind:value={newStepVerification} class="w-full rounded-md border px-3 py-2 text-sm" />
									<input type="text" placeholder="Caution/warning (optional)" bind:value={newStepCaution} class="w-full rounded-md border px-3 py-2 text-sm" />
									<div class="flex gap-2">
										<button class="rounded-md bg-primary px-3 py-1.5 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50" disabled={!newStepAction.trim() || actionInProgress} onclick={addStep}>{actionInProgress ? 'Adding...' : 'Add Step'}</button>
										<button class="rounded-md border px-3 py-1.5 text-sm hover:bg-accent" onclick={() => (showAddStep = false)}>Cancel</button>
									</div>
								</div>
							{/if}
							<div class="space-y-4">
								{#each steps as step}
									<div class="rounded-lg border p-4 {step.critical ? 'border-orange-500/50 bg-orange-500/5' : ''}">
										<div class="flex items-start gap-4">
											<div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-sm font-bold text-primary">
												{step.number}
											</div>
											<div class="flex-1">
												<div class="flex items-center gap-2">
													<p class="font-medium">{step.instruction}</p>
													{#if step.critical}
														<Badge variant="destructive" class="text-xs">Critical</Badge>
													{/if}
												</div>
												{#if step.notes}
													<p class="mt-2 text-sm text-muted-foreground">{step.notes}</p>
												{/if}
											</div>
											<button class="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive" title="Remove step" onclick={() => removeStep(step.number)}>
												<Trash2 class="h-4 w-4" />
											</button>
										</div>
									</div>
								{/each}
							</div>
							{#if steps.length === 0 && !showAddStep}
								<p class="text-sm text-muted-foreground">No procedure steps defined. Click "Add Step" to begin.</p>
							{/if}
						</CardContent>
					</Card>

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
				<!-- Properties -->
				<Card>
					<CardHeader>
						<CardTitle>Properties</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Revision</span>
							<span class="text-sm font-medium">{revision}</span>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Steps</span>
							<span class="text-sm font-medium">{steps.length}</span>
						</div>
					</CardContent>
				</Card>

				<!-- Equipment -->
				{#if equipment.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Wrench class="h-4 w-4" />
								Equipment
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="flex flex-wrap gap-2">
								{#each equipment as item}
									<Badge variant="outline">{item}</Badge>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Materials -->
				{#if materials.length > 0}
					<Card>
						<CardHeader>
							<CardTitle>Materials</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="flex flex-wrap gap-2">
								{#each materials as material}
									<Badge variant="secondary">{material}</Badge>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Tools Management -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle class="flex items-center gap-2">
								<Wrench class="h-4 w-4" />
								Tools ({tools.length})
							</CardTitle>
							<button
								class="inline-flex items-center gap-1 rounded-md border px-2 py-1 text-xs hover:bg-accent"
								onclick={() => (showAddTool = !showAddTool)}
							>
								<Plus class="h-3 w-3" />
								Add
							</button>
						</div>
					</CardHeader>
					<CardContent>
						{#if showAddTool}
							<div class="mb-3 space-y-2 rounded-lg border border-dashed p-3">
								<input type="text" placeholder="Tool name" bind:value={newToolName} class="w-full rounded-md border px-2 py-1.5 text-sm" />
								<input type="text" placeholder="Part number (optional)" bind:value={newToolPartNumber} class="w-full rounded-md border px-2 py-1.5 text-sm" />
								<div class="flex gap-2">
									<button class="rounded-md bg-primary px-2 py-1 text-xs text-primary-foreground hover:bg-primary/90 disabled:opacity-50" disabled={!newToolName.trim() || actionInProgress} onclick={addTool}>Add</button>
									<button class="rounded-md border px-2 py-1 text-xs hover:bg-accent" onclick={() => (showAddTool = false)}>Cancel</button>
								</div>
							</div>
						{/if}
						{#if tools.length > 0}
							<div class="space-y-2">
								{#each tools as tool}
									<div class="flex items-center justify-between rounded border p-2">
										<div>
											<p class="text-sm font-medium">{tool.name}</p>
											{#if tool.part_number}
												<p class="text-xs text-muted-foreground">{tool.part_number}</p>
											{/if}
										</div>
										<button class="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive" onclick={() => removeTool(tool.name)}>
											<Trash2 class="h-3 w-3" />
										</button>
									</div>
								{/each}
							</div>
						{:else if !showAddTool}
							<p class="text-xs text-muted-foreground">No tools defined.</p>
						{/if}
					</CardContent>
				</Card>

				<!-- Structured Materials Management -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle class="flex items-center gap-2">
								<ClipboardList class="h-4 w-4" />
								Bill of Materials ({structuredMaterials.length})
							</CardTitle>
							<button
								class="inline-flex items-center gap-1 rounded-md border px-2 py-1 text-xs hover:bg-accent"
								onclick={() => (showAddMaterial = !showAddMaterial)}
							>
								<Plus class="h-3 w-3" />
								Add
							</button>
						</div>
					</CardHeader>
					<CardContent>
						{#if showAddMaterial}
							<div class="mb-3 space-y-2 rounded-lg border border-dashed p-3">
								<input type="text" placeholder="Material name" bind:value={newMaterialName} class="w-full rounded-md border px-2 py-1.5 text-sm" />
								<input type="text" placeholder="Specification (optional)" bind:value={newMaterialSpec} class="w-full rounded-md border px-2 py-1.5 text-sm" />
								<div class="flex gap-2">
									<button class="rounded-md bg-primary px-2 py-1 text-xs text-primary-foreground hover:bg-primary/90 disabled:opacity-50" disabled={!newMaterialName.trim() || actionInProgress} onclick={addMaterial}>Add</button>
									<button class="rounded-md border px-2 py-1 text-xs hover:bg-accent" onclick={() => (showAddMaterial = false)}>Cancel</button>
								</div>
							</div>
						{/if}
						{#if structuredMaterials.length > 0}
							<div class="space-y-2">
								{#each structuredMaterials as mat}
									<div class="flex items-center justify-between rounded border p-2">
										<div>
											<p class="text-sm font-medium">{mat.name}</p>
											{#if mat.specification}
												<p class="text-xs text-muted-foreground">{mat.specification}</p>
											{/if}
										</div>
										<button class="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive" onclick={() => removeMaterial(mat.name)}>
											<Trash2 class="h-3 w-3" />
										</button>
									</div>
								{/each}
							</div>
						{:else if !showAddMaterial}
							<p class="text-xs text-muted-foreground">No materials defined.</p>
						{/if}
					</CardContent>
				</Card>

				<!-- Quality Checks -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle class="flex items-center gap-2">
								<CheckSquare class="h-4 w-4" />
								Quality Checks ({qualityChecks.length})
							</CardTitle>
							<button
								class="inline-flex items-center gap-1 rounded-md border px-2 py-1 text-xs hover:bg-accent"
								onclick={() => (showAddQC = !showAddQC)}
							>
								<Plus class="h-3 w-3" />
								Add
							</button>
						</div>
					</CardHeader>
					<CardContent>
						{#if showAddQC}
							<div class="mb-3 space-y-2 rounded-lg border border-dashed p-3">
								<input type="number" placeholder="At step #" min={1} bind:value={newQCStep} class="w-full rounded-md border px-2 py-1.5 text-sm" />
								<input type="text" placeholder="Characteristic" bind:value={newQCCharacteristic} class="w-full rounded-md border px-2 py-1.5 text-sm" />
								<input type="text" placeholder="Specification (optional)" bind:value={newQCSpec} class="w-full rounded-md border px-2 py-1.5 text-sm" />
								<div class="flex gap-2">
									<button class="rounded-md bg-primary px-2 py-1 text-xs text-primary-foreground hover:bg-primary/90 disabled:opacity-50" disabled={!newQCCharacteristic.trim() || actionInProgress} onclick={addQualityCheck}>Add</button>
									<button class="rounded-md border px-2 py-1 text-xs hover:bg-accent" onclick={() => (showAddQC = false)}>Cancel</button>
								</div>
							</div>
						{/if}
						{#if qualityChecks.length > 0}
							<div class="space-y-2">
								{#each qualityChecks as qc}
									<div class="flex items-center justify-between rounded border p-2">
										<div>
											<p class="text-sm font-medium">Step {qc.at_step}: {qc.characteristic}</p>
											{#if qc.specification}
												<p class="text-xs text-muted-foreground">{qc.specification}</p>
											{/if}
										</div>
										<button class="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive" onclick={() => removeQualityCheck(qc.at_step)}>
											<Trash2 class="h-3 w-3" />
										</button>
									</div>
								{/each}
							</div>
						{:else if !showAddQC}
							<p class="text-xs text-muted-foreground">No quality checks defined.</p>
						{/if}
					</CardContent>
				</Card>

				<!-- Safety -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle class="flex items-center gap-2">
								<Shield class="h-4 w-4" />
								Safety
							</CardTitle>
							{#if safety}
								<button
									class="rounded-md border border-destructive/50 px-2 py-1 text-xs text-destructive hover:bg-destructive/10"
									onclick={clearSafety}
								>
									Clear
								</button>
							{/if}
						</div>
					</CardHeader>
					<CardContent>
						{#if safety}
							{#if safety.ppe_required && safety.ppe_required.length > 0}
								<div class="mb-3">
									<p class="mb-1 text-xs font-medium text-muted-foreground uppercase">PPE Required</p>
									<div class="space-y-1">
										{#each safety.ppe_required as ppe}
											<Badge variant="outline" class="mr-1">{ppe.item}{#if ppe.specification} ({ppe.specification}){/if}</Badge>
										{/each}
									</div>
								</div>
							{/if}
							{#if safety.hazards && safety.hazards.length > 0}
								<div>
									<p class="mb-1 text-xs font-medium text-muted-foreground uppercase">Hazards</p>
									<div class="space-y-1">
										{#each safety.hazards as h}
											<div class="rounded border border-orange-500/30 bg-orange-500/5 p-2 text-sm">
												<p class="font-medium text-orange-600 dark:text-orange-400">{h.hazard}</p>
												{#if h.mitigation}
													<p class="text-xs text-muted-foreground">{h.mitigation}</p>
												{/if}
											</div>
										{/each}
									</div>
								</div>
							{/if}
						{:else}
							<p class="text-xs text-muted-foreground">No safety information defined.</p>
						{/if}
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
				<p class="text-muted-foreground">Work instruction not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
