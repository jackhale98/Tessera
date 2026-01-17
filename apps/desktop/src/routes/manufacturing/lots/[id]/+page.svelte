<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge, Button, Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter, Input, Label, Textarea, Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { lots, traceability } from '$lib/api/tauri';
	import type { LinkInfo } from '$lib/api/tauri';
	import {
		Package,
		User,
		Calendar,
		Tag,
		FileText,
		Hash,
		Clock,
		CheckCircle2,
		Box,
		Pause,
		Play,
		Trash2,
		ChevronRight,
		XCircle,
		AlertTriangle
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
	let showStepModal = $state(false);
	let selectedStepIndex = $state<number | null>(null);

	// Step update form data
	let stepStatus = $state('completed');
	let stepOperator = $state('');
	let stepNotes = $state('');

	// Track if we've loaded for this ID to prevent double-loads
	let loadedId = $state<string | null>(null);

	// Type-safe data access
	const lotNumber = $derived((entity?.lot_number as string) ?? null);
	const lotStatus = $derived((entity?.lot_status as string) ?? 'in_progress');
	const quantity = $derived(entity?.quantity as number | null);
	const startDate = $derived((entity?.start_date as string) ?? null);
	const completionDate = $derived((entity?.completion_date as string) ?? null);
	const description = $derived((entity?.notes as string) ?? '');
	const revision = $derived((entity?.entity_revision as number) ?? 1);
	const entityStatus = $derived((entity?.status as string) ?? 'draft');
	const entityTitle = $derived((entity?.title as string) ?? '');
	const entityAuthor = $derived((entity?.author as string) ?? '');
	const entityCreated = $derived((entity?.created as string) ?? '');
	const entityTags = $derived((entity?.tags as string[]) ?? []);
	const product = $derived((entity?.product as string) ?? null);

	interface Material {
		material_id?: string;
		material_lot?: string;
		quantity_used?: number;
		unit?: string;
	}
	const materialsUsed = $derived((entity?.materials_used as Material[]) ?? []);

	interface ExecutionStep {
		process_id: string;
		process_name?: string;
		status: string;
		operator?: string;
		started_at?: string;
		completed_at?: string;
		notes?: string;
		work_instructions_used?: string[];
	}
	const executionSteps = $derived((entity?.execution_steps as ExecutionStep[]) ?? []);

	// Calculate progress
	const completedSteps = $derived(executionSteps.filter(s => s.status === 'completed').length);
	const totalSteps = $derived(executionSteps.length);
	const progressPercent = $derived(totalSteps > 0 ? Math.round((completedSteps / totalSteps) * 100) : 0);

	async function loadData() {
		if (!id) return;

		loading = true;
		linksLoading = true;
		error = null;

		try {
			const [entityResult, fromLinks, toLinks] = await Promise.all([
				lots.get(id),
				traceability.getLinksFrom(id),
				traceability.getLinksTo(id)
			]);

			entity = entityResult as Record<string, unknown>;
			linksFrom = fromLinks;
			linksTo = toLinks;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load lot:', e);
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

	function formatLotStatus(status: string): string {
		const statuses: Record<string, string> = {
			in_progress: 'In Progress',
			on_hold: 'On Hold',
			completed: 'Completed',
			scrapped: 'Scrapped'
		};
		return statuses[status.toLowerCase()] ?? status;
	}

	function getLotStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			in_progress: 'secondary',
			on_hold: 'outline',
			completed: 'default',
			scrapped: 'destructive'
		};
		return variants[status.toLowerCase()] ?? 'outline';
	}

	function getStepStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			pending: 'outline',
			in_progress: 'secondary',
			completed: 'default',
			skipped: 'destructive'
		};
		return variants[status.toLowerCase()] ?? 'outline';
	}

	function formatStepStatus(status: string): string {
		const statuses: Record<string, string> = {
			pending: 'Pending',
			in_progress: 'In Progress',
			completed: 'Completed',
			skipped: 'Skipped'
		};
		return statuses[status.toLowerCase()] ?? status;
	}

	// Workflow Actions
	async function handlePutOnHold() {
		actionInProgress = true;
		try {
			await lots.putOnHold(id);
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			actionInProgress = false;
		}
	}

	async function handleResume() {
		actionInProgress = true;
		try {
			await lots.resume(id);
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			actionInProgress = false;
		}
	}

	async function handleComplete() {
		actionInProgress = true;
		try {
			await lots.complete(id);
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			actionInProgress = false;
		}
	}

	async function handleScrap() {
		if (!confirm('Are you sure you want to scrap this lot? This action cannot be undone.')) {
			return;
		}
		actionInProgress = true;
		try {
			await lots.scrap(id);
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			actionInProgress = false;
		}
	}

	function openStepModal(index: number) {
		selectedStepIndex = index;
		const step = executionSteps[index];
		stepStatus = step.status === 'pending' ? 'in_progress' : 'completed';
		stepOperator = step.operator ?? '';
		stepNotes = step.notes ?? '';
		showStepModal = true;
	}

	async function handleUpdateStep() {
		if (selectedStepIndex === null) return;

		actionInProgress = true;
		try {
			await lots.updateStep(
				id,
				selectedStepIndex,
				stepStatus,
				stepOperator || undefined,
				stepNotes || undefined
			);
			showStepModal = false;
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
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
			id={id}
			title={entityTitle}
			status={entityStatus}
			subtitle={lotNumber ? `Lot: ${lotNumber}` : 'Production Lot'}
			backHref="/manufacturing/lots"
			backLabel="Lots"
			onEdit={() => goto(`/manufacturing/lots/${id}/edit`)}
		/>

		<!-- Workflow Actions -->
		{#if lotStatus !== 'completed' && lotStatus !== 'scrapped'}
			<Card>
				<CardHeader>
					<CardTitle>Workflow Actions</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="flex flex-wrap gap-3">
						{#if lotStatus === 'in_progress'}
							<Button
								variant="outline"
								onclick={handlePutOnHold}
								disabled={actionInProgress}
							>
								<Pause class="mr-2 h-4 w-4" />
								Put On Hold
							</Button>
							<Button
								variant="default"
								onclick={handleComplete}
								disabled={actionInProgress || completedSteps < totalSteps}
							>
								<CheckCircle2 class="mr-2 h-4 w-4" />
								Complete Lot
							</Button>
						{:else if lotStatus === 'on_hold'}
							<Button
								variant="default"
								onclick={handleResume}
								disabled={actionInProgress}
							>
								<Play class="mr-2 h-4 w-4" />
								Resume
							</Button>
						{/if}

						<Button
							variant="destructive"
							onclick={handleScrap}
							disabled={actionInProgress}
						>
							<Trash2 class="mr-2 h-4 w-4" />
							Scrap Lot
						</Button>
					</div>

					{#if completedSteps < totalSteps && lotStatus === 'in_progress'}
						<p class="mt-3 text-sm text-muted-foreground">
							Complete all {totalSteps - completedSteps} remaining steps before marking the lot as complete.
						</p>
					{/if}
				</CardContent>
			</Card>
		{/if}

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Progress Overview -->
				{#if totalSteps > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center justify-between">
								<span>Production Progress</span>
								<Badge variant={progressPercent === 100 ? 'default' : 'secondary'}>
									{progressPercent}%
								</Badge>
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								<div class="flex justify-between text-sm">
									<span>{completedSteps} of {totalSteps} steps completed</span>
								</div>
								<div class="h-2 w-full rounded-full bg-muted">
									<div
										class="h-full rounded-full bg-primary transition-all"
										style="width: {progressPercent}%"
									></div>
								</div>
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Description -->
				{#if description}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<FileText class="h-5 w-5" />
								Notes
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap">{description}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Execution Steps -->
				{#if executionSteps.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<CheckCircle2 class="h-5 w-5" />
								Execution Steps ({executionSteps.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-3">
								{#each executionSteps as step, i}
									<div class="flex items-center gap-4 rounded-lg border p-3">
										<div class="flex h-8 w-8 items-center justify-center rounded-full {step.status === 'completed' ? 'bg-green-500/10 text-green-500' : step.status === 'in_progress' ? 'bg-blue-500/10 text-blue-500' : 'bg-muted text-muted-foreground'}">
											{#if step.status === 'completed'}
												<CheckCircle2 class="h-4 w-4" />
											{:else if step.status === 'skipped'}
												<XCircle class="h-4 w-4" />
											{:else}
												<span class="text-sm font-medium">{i + 1}</span>
											{/if}
										</div>
										<div class="flex-1">
											<p class="font-medium">{step.process_name ?? step.process_id}</p>
											{#if step.operator || step.completed_at}
												<p class="text-sm text-muted-foreground">
													{#if step.operator}by {step.operator}{/if}
													{#if step.completed_at} on {formatDate(step.completed_at)}{/if}
												</p>
											{/if}
											{#if step.notes}
												<p class="text-sm text-muted-foreground mt-1">{step.notes}</p>
											{/if}
										</div>
										<div class="flex items-center gap-2">
											<Badge variant={getStepStatusVariant(step.status)}>
												{formatStepStatus(step.status)}
											</Badge>
											{#if lotStatus === 'in_progress' && step.status !== 'completed' && step.status !== 'skipped'}
												<Button
													variant="ghost"
													size="sm"
													onclick={() => openStepModal(i)}
												>
													Update
												</Button>
											{/if}
										</div>
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Materials Used -->
				{#if materialsUsed.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Box class="h-5 w-5" />
								Materials Used ({materialsUsed.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="overflow-x-auto">
								<table class="w-full text-sm">
									<thead>
										<tr class="border-b">
											<th class="px-4 py-2 text-left font-medium">Material ID</th>
											<th class="px-4 py-2 text-left font-medium">Lot</th>
											<th class="px-4 py-2 text-left font-medium">Quantity</th>
										</tr>
									</thead>
									<tbody>
										{#each materialsUsed as material}
											<tr class="border-b">
												<td class="px-4 py-2 font-mono text-xs">{material.material_id ?? '-'}</td>
												<td class="px-4 py-2 font-mono text-xs">{material.material_lot ?? '—'}</td>
												<td class="px-4 py-2 text-muted-foreground">
													{#if material.quantity_used}
														{material.quantity_used} {material.unit ?? ''}
													{:else}
														—
													{/if}
												</td>
											</tr>
										{/each}
									</tbody>
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
				<!-- Properties -->
				<Card>
					<CardHeader>
						<CardTitle>Properties</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if lotNumber}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Lot Number</span>
								<span class="font-mono text-sm font-medium">{lotNumber}</span>
							</div>
						{/if}
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Lot Status</span>
							<Badge variant={getLotStatusVariant(lotStatus)}>
								{formatLotStatus(lotStatus)}
							</Badge>
						</div>
						{#if product}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Product</span>
								<span class="text-sm font-medium">{product}</span>
							</div>
						{/if}
						{#if quantity}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Quantity</span>
								<div class="flex items-center gap-1">
									<Hash class="h-3 w-3 text-muted-foreground" />
									<span class="text-sm font-medium">{quantity.toLocaleString()}</span>
								</div>
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

				<!-- Timeline -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Clock class="h-4 w-4" />
							Timeline
						</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if startDate}
							<div class="flex items-center gap-2">
								<Calendar class="h-4 w-4 text-muted-foreground" />
								<span class="text-sm text-muted-foreground">Start</span>
								<span class="ml-auto text-sm font-medium">{formatDate(startDate)}</span>
							</div>
						{/if}
						{#if completionDate}
							<div class="flex items-center gap-2">
								<Calendar class="h-4 w-4 text-muted-foreground" />
								<span class="text-sm text-muted-foreground">Completed</span>
								<span class="ml-auto text-sm font-medium">{formatDate(completionDate)}</span>
							</div>
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
				<p class="text-muted-foreground">Lot not found</p>
			</CardContent>
		</Card>
	{/if}
</div>

<!-- Update Step Modal -->
<Dialog bind:open={showStepModal}>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Update Step</DialogTitle>
			<DialogDescription>
				Update the status and record details for this execution step.
			</DialogDescription>
		</DialogHeader>
		<div class="space-y-4 py-4">
			<div class="space-y-2">
				<Label for="step_status">Status</Label>
				<Select bind:value={stepStatus}>
					<SelectTrigger>
						<SelectValue placeholder="Select status..." />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="pending">Pending</SelectItem>
						<SelectItem value="in_progress">In Progress</SelectItem>
						<SelectItem value="completed">Completed</SelectItem>
						<SelectItem value="skipped">Skipped</SelectItem>
					</SelectContent>
				</Select>
			</div>
			<div class="space-y-2">
				<Label for="step_operator">Operator</Label>
				<Input
					id="step_operator"
					bind:value={stepOperator}
					placeholder="Enter operator name"
				/>
			</div>
			<div class="space-y-2">
				<Label for="step_notes">Notes (optional)</Label>
				<Textarea
					id="step_notes"
					bind:value={stepNotes}
					placeholder="Enter any notes about this step..."
					rows={3}
				/>
			</div>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => { showStepModal = false; }}>Cancel</Button>
			<Button onclick={handleUpdateStep} disabled={actionInProgress}>
				Update Step
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
