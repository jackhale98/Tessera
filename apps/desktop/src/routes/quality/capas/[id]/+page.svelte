<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge, Button, Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter, Input, Label, Textarea } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { capas, traceability } from '$lib/api/tauri';
	import type { LinkInfo } from '$lib/api/tauri';
	import EntityHistory from '$lib/components/EntityHistory.svelte';
	import {
		Shield,
		User,
		Calendar,
		Tag,
		FileText,
		Search,
		ListChecks,
		CheckCircle2,
		Clock,
		ChevronRight,
		XCircle,
		AlertTriangle,
		History
	} from 'lucide-svelte';

	const id = $derived($page.params.id ?? '');

	let entity = $state<Record<string, unknown> | null>(null);
	let linksFrom = $state<LinkInfo[]>([]);
	let linksTo = $state<LinkInfo[]>([]);
	let loading = $state(true);
	let linksLoading = $state(true);
	let error = $state<string | null>(null);
	let actionInProgress = $state(false);

	// Modal states
	let showVerifyModal = $state(false);

	// Verify effectiveness form data
	let verifyEffective = $state(true);
	let verifyBy = $state('');
	let verifyNotes = $state('');

	// Track if we've loaded for this ID to prevent double-loads
	let loadedId = $state<string | null>(null);

	// Type-safe data access
	const capaType = $derived((entity?.capa_type as string) ?? 'corrective');
	const capaStatus = $derived((entity?.capa_status as string) ?? 'initiation');
	const description = $derived((entity?.description as string) ?? '');
	const rootCauseAnalysis = $derived((entity?.root_cause_analysis as string) ?? null);
	const effectivenessVerified = $derived(entity?.effectiveness_verified as boolean | null);
	const effectivenessNotes = $derived((entity?.effectiveness_notes as string) ?? null);
	const effectivenessDate = $derived((entity?.effectiveness_date as string) ?? null);
	const verifiedBy = $derived((entity?.verified_by as string) ?? null);
	const dueDate = $derived((entity?.due_date as string) ?? null);
	const revision = $derived((entity?.entity_revision as number) ?? 1);
	const entityStatus = $derived((entity?.status as string) ?? 'draft');
	const entityTitle = $derived((entity?.title as string) ?? '');
	const entityAuthor = $derived((entity?.author as string) ?? '');
	const entityCreated = $derived((entity?.created as string) ?? '');
	const entityTags = $derived((entity?.tags as string[]) ?? []);

	interface Action {
		description: string;
		owner?: string;
		due_date?: string;
		status?: string;
		completion_date?: string;
	}
	const actions = $derived((entity?.actions as Action[]) ?? []);

	async function loadData() {
		if (!id) return;

		loading = true;
		linksLoading = true;
		error = null;

		try {
			const [entityResult, fromLinks, toLinks] = await Promise.all([
				capas.get(id),
				traceability.getLinksFrom(id),
				traceability.getLinksTo(id)
			]);

			entity = entityResult as Record<string, unknown>;
			linksFrom = fromLinks;
			linksTo = toLinks;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load CAPA:', e);
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

	function formatCapaType(type: string): string {
		return type === 'corrective' ? 'Corrective Action' : 'Preventive Action';
	}

	function formatCapaStatus(status: string): string {
		const statuses: Record<string, string> = {
			initiation: 'Initiation',
			investigation: 'Investigation',
			implementation: 'Implementation',
			verification: 'Verification',
			closed: 'Closed'
		};
		return statuses[status.toLowerCase()] ?? status;
	}

	function getCapaStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			initiation: 'outline',
			investigation: 'secondary',
			implementation: 'secondary',
			verification: 'destructive',
			closed: 'default'
		};
		return variants[status.toLowerCase()] ?? 'outline';
	}

	function getActionStatusVariant(status: string | undefined): 'default' | 'secondary' | 'destructive' | 'outline' {
		if (!status) return 'outline';
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			open: 'outline',
			in_progress: 'secondary',
			completed: 'default',
			verified: 'default',
			overdue: 'destructive'
		};
		return variants[status.toLowerCase()] ?? 'outline';
	}

	function getNextStatusLabel(status: string): string {
		const labels: Record<string, string> = {
			initiation: 'Start Investigation',
			investigation: 'Start Implementation',
			implementation: 'Ready for Verification',
			verification: 'Verify Effectiveness'
		};
		return labels[status.toLowerCase()] ?? 'Advance Status';
	}

	function isOverdue(): boolean {
		if (!dueDate) return false;
		return new Date(dueDate) < new Date();
	}

	// Workflow Actions
	async function handleAdvanceStatus() {
		if (capaStatus === 'verification') {
			showVerifyModal = true;
			return;
		}

		actionInProgress = true;
		try {
			await capas.advanceStatus(id);
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			actionInProgress = false;
		}
	}

	async function handleVerifyEffectiveness() {
		if (!verifyBy) return;

		actionInProgress = true;
		try {
			await capas.verifyEffectiveness(id, {
				effective: verifyEffective,
				verified_by: verifyBy,
				notes: verifyNotes || undefined
			});
			showVerifyModal = false;
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			actionInProgress = false;
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
			id={id}
			title={entityTitle}
			status={entityStatus}
			subtitle={formatCapaType(capaType)}
			backHref="/quality/capas"
			backLabel="CAPAs"
			onEdit={() => goto(`/quality/capas/${id}/edit`)}
		/>

		<!-- Workflow Actions -->
		{#if capaStatus !== 'closed'}
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						Workflow Actions
						{#if isOverdue()}
							<Badge variant="destructive" class="ml-2">
								<AlertTriangle class="mr-1 h-3 w-3" />
								Overdue
							</Badge>
						{/if}
					</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="flex flex-wrap gap-3">
						<Button
							variant="default"
							onclick={handleAdvanceStatus}
							disabled={actionInProgress}
						>
							<ChevronRight class="mr-2 h-4 w-4" />
							{getNextStatusLabel(capaStatus)}
						</Button>
					</div>

					<!-- Workflow Status Display -->
					<div class="mt-4 flex items-center gap-2 text-sm text-muted-foreground">
						<span class={capaStatus === 'initiation' ? 'font-bold text-foreground' : ''}>Initiation</span>
						<ChevronRight class="h-4 w-4" />
						<span class={capaStatus === 'investigation' ? 'font-bold text-foreground' : ''}>Investigation</span>
						<ChevronRight class="h-4 w-4" />
						<span class={capaStatus === 'implementation' ? 'font-bold text-foreground' : ''}>Implementation</span>
						<ChevronRight class="h-4 w-4" />
						<span class={capaStatus === 'verification' ? 'font-bold text-foreground' : ''}>Verification</span>
						<ChevronRight class="h-4 w-4" />
						<span class={capaStatus === 'closed' ? 'font-bold text-foreground' : ''}>Closed</span>
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
							<FileText class="h-5 w-5" />
							Description
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p class="whitespace-pre-wrap">{description || 'No description specified.'}</p>
					</CardContent>
				</Card>

				<!-- Root Cause Analysis -->
				{#if rootCauseAnalysis}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Search class="h-5 w-5" />
								Root Cause Analysis
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap text-muted-foreground">{rootCauseAnalysis}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Actions -->
				{#if actions.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<ListChecks class="h-5 w-5" />
								Actions ({actions.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-4">
								{#each actions as action, i}
									<div class="rounded-lg border p-4">
										<div class="flex items-start justify-between gap-4">
											<div class="flex-1">
												<div class="flex items-center gap-2">
													<span class="text-sm font-medium text-muted-foreground">#{i + 1}</span>
													{#if action.status}
														<Badge variant={getActionStatusVariant(action.status)} class="capitalize">
															{action.status.replace(/_/g, ' ')}
														</Badge>
													{/if}
												</div>
												<p class="mt-2">{action.description}</p>
											</div>
										</div>
										{#if action.owner || action.due_date || action.completion_date}
											<div class="mt-3 flex items-center gap-4 text-sm text-muted-foreground">
												{#if action.owner}
													<span class="flex items-center gap-1">
														<User class="h-3 w-3" />
														{action.owner}
													</span>
												{/if}
												{#if action.due_date}
													<span class="flex items-center gap-1">
														<Clock class="h-3 w-3" />
														Due: {formatDate(action.due_date)}
													</span>
												{/if}
												{#if action.completion_date}
													<span class="flex items-center gap-1">
														<CheckCircle2 class="h-3 w-3" />
														Completed: {formatDate(action.completion_date)}
													</span>
												{/if}
											</div>
										{/if}
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Effectiveness Verification -->
				{#if effectivenessVerified !== null}
					<Card class={effectivenessVerified ? 'border-green-500' : 'border-red-500'}>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								{#if effectivenessVerified}
									<CheckCircle2 class="h-5 w-5 text-green-500" />
								{:else}
									<XCircle class="h-5 w-5 text-red-500" />
								{/if}
								Effectiveness Verification
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							<div class="flex items-center gap-4">
								<Badge variant={effectivenessVerified ? 'default' : 'destructive'} class="text-lg px-4 py-1">
									{effectivenessVerified ? 'Effective' : 'Not Effective'}
								</Badge>
							</div>
							{#if effectivenessNotes}
								<div>
									<h4 class="text-sm font-medium text-muted-foreground">Notes</h4>
									<p class="mt-1">{effectivenessNotes}</p>
								</div>
							{/if}
							<div class="flex items-center gap-4 text-sm text-muted-foreground">
								{#if verifiedBy}
									<span class="flex items-center gap-1">
										<User class="h-3 w-3" />
										Verified by: {verifiedBy}
									</span>
								{/if}
								{#if effectivenessDate}
									<span class="flex items-center gap-1">
										<Calendar class="h-3 w-3" />
										{formatDate(effectivenessDate)}
									</span>
								{/if}
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
				<!-- CAPA Status -->
				<Card>
					<CardHeader>
						<CardTitle>CAPA Status</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">CAPA Status</span>
							<Badge variant={getCapaStatusVariant(capaStatus)}>
								{formatCapaStatus(capaStatus)}
							</Badge>
						</div>
						{#if dueDate}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Due Date</span>
								<span class={`text-sm font-medium ${isOverdue() ? 'text-red-500' : ''}`}>
									{formatDate(dueDate)}
								</span>
							</div>
						{/if}
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
							<Badge variant={capaType === 'corrective' ? 'secondary' : 'default'}>
								{capaType === 'corrective' ? 'Corrective' : 'Preventive'}
							</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entityStatus} />
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Actions</span>
							<Badge variant="outline">{actions.length}</Badge>
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
				<p class="text-muted-foreground">CAPA not found</p>
			</CardContent>
		</Card>
	{/if}
</div>

<!-- Verify Effectiveness Modal -->
<Dialog bind:open={showVerifyModal}>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Verify CAPA Effectiveness</DialogTitle>
			<DialogDescription>
				Record the effectiveness verification for this CAPA. This will close the CAPA.
			</DialogDescription>
		</DialogHeader>
		<div class="space-y-4 py-4">
			<div class="space-y-2">
				<Label>Effectiveness Assessment</Label>
				<div class="flex gap-4">
					<label class="flex items-center gap-2">
						<input
							type="radio"
							name="effectiveness"
							value={true}
							checked={verifyEffective}
							onchange={() => { verifyEffective = true; }}
							class="h-4 w-4"
						/>
						<span class="flex items-center gap-1">
							<CheckCircle2 class="h-4 w-4 text-green-500" />
							Effective
						</span>
					</label>
					<label class="flex items-center gap-2">
						<input
							type="radio"
							name="effectiveness"
							value={false}
							checked={!verifyEffective}
							onchange={() => { verifyEffective = false; }}
							class="h-4 w-4"
						/>
						<span class="flex items-center gap-1">
							<XCircle class="h-4 w-4 text-red-500" />
							Not Effective
						</span>
					</label>
				</div>
			</div>
			<div class="space-y-2">
				<Label for="verified_by">Verified By</Label>
				<Input
					id="verified_by"
					bind:value={verifyBy}
					placeholder="Enter name of verifier"
				/>
			</div>
			<div class="space-y-2">
				<Label for="verify_notes">Notes (optional)</Label>
				<Textarea
					id="verify_notes"
					bind:value={verifyNotes}
					placeholder="Enter verification notes..."
					rows={3}
				/>
			</div>
			{#if !verifyEffective}
				<div class="rounded-lg bg-yellow-500/10 p-3 text-sm text-yellow-600 dark:text-yellow-400">
					<AlertTriangle class="inline-block h-4 w-4 mr-1" />
					If the CAPA is not effective, you may need to initiate a new CAPA to address the root cause.
				</div>
			{/if}
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => { showVerifyModal = false; }}>Cancel</Button>
			<Button
				onclick={handleVerifyEffectiveness}
				disabled={!verifyBy || actionInProgress}
				variant={verifyEffective ? 'default' : 'destructive'}
			>
				{verifyEffective ? 'Close as Effective' : 'Close as Not Effective'}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
