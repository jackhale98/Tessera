<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge, Button, Input, Label } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { deviations, traceability } from '$lib/api/tauri';
	import type { LinkInfo } from '$lib/api/tauri';
	import { projectAuthor } from '$lib/stores/project';
	import {
		AlertTriangle,
		User,
		Calendar,
		Tag,
		FileText,
		Scale,
		CheckCircle2,
		XCircle,
		Clock,
		ShieldAlert,
		Play,
		Ban
	} from 'lucide-svelte';

	const id = $derived($page.params.id);

	// Full deviation data from API
	let deviation = $state<Record<string, unknown> | null>(null);
	let linksFrom = $state<LinkInfo[]>([]);
	let linksTo = $state<LinkInfo[]>([]);
	let loading = $state(true);
	let linksLoading = $state(true);
	let error = $state<string | null>(null);
	let actionLoading = $state(false);
	let actionError = $state<string | null>(null);

	// Workflow modal state
	let showApproveModal = $state(false);
	let showRejectModal = $state(false);
	let approverName = $state('');
	let rejectReason = $state('');

	// Track if we've loaded for this ID to prevent double-loads
	let loadedId = $state<string | null>(null);

	// Derived values from deviation data
	const title = $derived((deviation?.title as string) ?? '');
	const status = $derived((deviation?.status as string) ?? 'draft');
	const devStatus = $derived((deviation?.dev_status as string) ?? 'pending');
	const deviationType = $derived((deviation?.deviation_type as string) ?? 'temporary');
	const category = $derived((deviation?.category as string) ?? 'process');
	const description = $derived((deviation?.description as string) ?? '');
	const riskLevel = $derived((deviation?.risk_level as string) ?? 'low');
	const riskAssessment = $derived((deviation?.risk_assessment as string) ?? '');
	const approvedBy = $derived((deviation?.approved_by as string) ?? null);
	const approvalDate = $derived((deviation?.approval_date as string) ?? null);
	const effectiveDate = $derived((deviation?.effective_date as string) ?? null);
	const expirationDate = $derived((deviation?.expiration_date as string) ?? null);
	const notes = $derived((deviation?.notes as string) ?? '');
	const author = $derived((deviation?.author as string) ?? '');
	const created = $derived((deviation?.created as string) ?? '');
	const tags = $derived((deviation?.tags as string[]) ?? []);

	async function loadData() {
		if (!id) return;

		loading = true;
		linksLoading = true;
		error = null;

		try {
			const [devResult, fromLinks, toLinks] = await Promise.all([
				deviations.get(id),
				traceability.getLinksFrom(id),
				traceability.getLinksTo(id)
			]);

			deviation = devResult as Record<string, unknown>;
			linksFrom = fromLinks;
			linksTo = toLinks;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load deviation:', e);
		} finally {
			loading = false;
			linksLoading = false;
		}
	}

	function formatDate(dateStr: string | null): string {
		if (!dateStr) return '-';
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

	function formatDevStatus(status: string): string {
		const statuses: Record<string, string> = {
			pending: '⏳ Pending',
			approved: '✓ Approved',
			active: '✅ Active',
			expired: '⚠️ Expired',
			closed: '📁 Closed',
			rejected: '❌ Rejected'
		};
		return statuses[status.toLowerCase()] ?? status;
	}

	function formatRiskLevel(level: string): string {
		const levels: Record<string, string> = {
			low: '🟢 Low',
			medium: '🟡 Medium',
			high: '🔴 High'
		};
		return levels[level.toLowerCase()] ?? level;
	}

	function getRiskBadgeVariant(level: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (level.toLowerCase()) {
			case 'high': return 'destructive';
			case 'medium': return 'secondary';
			default: return 'outline';
		}
	}

	// Workflow Actions
	async function handleApprove() {
		if (!id || !approverName.trim()) return;

		actionLoading = true;
		actionError = null;

		try {
			await deviations.approve(id, {
				approved_by: approverName.trim(),
				authorization_level: 'engineering',
				activate: false
			});
			showApproveModal = false;
			approverName = '';
			await loadData();
		} catch (e) {
			actionError = e instanceof Error ? e.message : String(e);
		} finally {
			actionLoading = false;
		}
	}

	async function handleReject() {
		if (!id) return;

		actionLoading = true;
		actionError = null;

		try {
			await deviations.reject(id, { reason: rejectReason.trim() || undefined });
			showRejectModal = false;
			rejectReason = '';
			await loadData();
		} catch (e) {
			actionError = e instanceof Error ? e.message : String(e);
		} finally {
			actionLoading = false;
		}
	}

	async function handleActivate() {
		if (!id) return;

		actionLoading = true;
		actionError = null;

		try {
			await deviations.activate(id);
			await loadData();
		} catch (e) {
			actionError = e instanceof Error ? e.message : String(e);
		} finally {
			actionLoading = false;
		}
	}

	async function handleClose() {
		if (!id) return;

		actionLoading = true;
		actionError = null;

		try {
			await deviations.close(id);
			await loadData();
		} catch (e) {
			actionError = e instanceof Error ? e.message : String(e);
		} finally {
			actionLoading = false;
		}
	}

	async function handleExpire() {
		if (!id) return;

		actionLoading = true;
		actionError = null;

		try {
			await deviations.expire(id);
			await loadData();
		} catch (e) {
			actionError = e instanceof Error ? e.message : String(e);
		} finally {
			actionLoading = false;
		}
	}

	// Initialize approver name from project author
	$effect(() => {
		if (!approverName && $projectAuthor) {
			approverName = $projectAuthor;
		}
	});

	$effect(() => {
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
	{:else if deviation}
		<!-- Header -->
		<EntityDetailHeader
			{id}
			{title}
			{status}
			subtitle="{deviationType} deviation - {category}"
			backHref="/manufacturing/deviations"
			backLabel="Deviations"
			onEdit={() => goto(`/manufacturing/deviations/${id}/edit`)}
		/>

		<!-- Workflow Actions -->
		{#if devStatus === 'pending' || devStatus === 'approved' || devStatus === 'active'}
			<Card>
				<CardHeader class="pb-3">
					<CardTitle class="flex items-center gap-2 text-base">
						<ShieldAlert class="h-4 w-4" />
						Workflow Actions
					</CardTitle>
				</CardHeader>
				<CardContent>
					{#if actionError}
						<p class="mb-4 text-sm text-destructive">{actionError}</p>
					{/if}

					<div class="flex flex-wrap gap-2">
						{#if devStatus === 'pending'}
							<Button
								variant="default"
								size="sm"
								onclick={() => { showApproveModal = true; }}
								disabled={actionLoading}
							>
								<CheckCircle2 class="mr-2 h-4 w-4" />
								Approve
							</Button>
							<Button
								variant="destructive"
								size="sm"
								onclick={() => { showRejectModal = true; }}
								disabled={actionLoading}
							>
								<XCircle class="mr-2 h-4 w-4" />
								Reject
							</Button>
						{:else if devStatus === 'approved'}
							<Button
								variant="default"
								size="sm"
								onclick={handleActivate}
								disabled={actionLoading}
							>
								<Play class="mr-2 h-4 w-4" />
								Activate
							</Button>
							<Button
								variant="outline"
								size="sm"
								onclick={handleClose}
								disabled={actionLoading}
							>
								<Ban class="mr-2 h-4 w-4" />
								Close
							</Button>
						{:else if devStatus === 'active'}
							<Button
								variant="secondary"
								size="sm"
								onclick={handleExpire}
								disabled={actionLoading}
							>
								<Clock class="mr-2 h-4 w-4" />
								Expire
							</Button>
							<Button
								variant="outline"
								size="sm"
								onclick={handleClose}
								disabled={actionLoading}
							>
								<Ban class="mr-2 h-4 w-4" />
								Close
							</Button>
						{/if}
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
							<AlertTriangle class="h-5 w-5" />
							Description
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p class="whitespace-pre-wrap">{description || 'No description specified.'}</p>
					</CardContent>
				</Card>

				<!-- Risk Assessment -->
				{#if riskAssessment}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Scale class="h-5 w-5" />
								Risk Assessment
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap text-muted-foreground">{riskAssessment}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Notes -->
				{#if notes}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<FileText class="h-5 w-5" />
								Notes
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap text-muted-foreground">{notes}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Links -->
				<LinksSection {linksFrom} {linksTo} loading={linksLoading} />
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Deviation Status -->
				<Card>
					<CardHeader>
						<CardTitle>Deviation Status</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Workflow Status</span>
							<span class="text-sm font-medium">{formatDevStatus(devStatus)}</span>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Type</span>
							<Badge variant="outline">{deviationType}</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Category</span>
							<Badge variant="outline">{category}</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Document Status</span>
							<StatusBadge {status} />
						</div>
					</CardContent>
				</Card>

				<!-- Risk -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<ShieldAlert class="h-4 w-4" />
							Risk Level
						</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="text-center">
							<Badge variant={getRiskBadgeVariant(riskLevel)} class="text-lg px-4 py-2">
								{formatRiskLevel(riskLevel)}
							</Badge>
						</div>
					</CardContent>
				</Card>

				<!-- Approval -->
				{#if approvedBy || approvalDate || effectiveDate || expirationDate}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<CheckCircle2 class="h-4 w-4" />
								Approval & Dates
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if approvedBy}
								<div class="flex items-center gap-2">
									<User class="h-4 w-4 text-muted-foreground" />
									<span class="text-sm text-muted-foreground">Approver</span>
									<span class="ml-auto text-sm font-medium">{approvedBy}</span>
								</div>
							{/if}
							{#if approvalDate}
								<div class="flex items-center gap-2">
									<Calendar class="h-4 w-4 text-muted-foreground" />
									<span class="text-sm text-muted-foreground">Approved</span>
									<span class="ml-auto text-sm font-medium">{formatDate(approvalDate)}</span>
								</div>
							{/if}
							{#if effectiveDate}
								<div class="flex items-center gap-2">
									<Calendar class="h-4 w-4 text-muted-foreground" />
									<span class="text-sm text-muted-foreground">Effective</span>
									<span class="ml-auto text-sm font-medium">{formatDate(effectiveDate)}</span>
								</div>
							{/if}
							{#if expirationDate}
								<div class="flex items-center gap-2">
									<Calendar class="h-4 w-4 text-muted-foreground" />
									<span class="text-sm text-muted-foreground">Expires</span>
									<span class="ml-auto text-sm font-medium">{formatDate(expirationDate)}</span>
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
							<span class="ml-auto text-sm font-medium">{author}</span>
						</div>
						<div class="flex items-center gap-2">
							<Calendar class="h-4 w-4 text-muted-foreground" />
							<span class="text-sm text-muted-foreground">Created</span>
							<span class="ml-auto text-sm font-medium">{formatDate(created)}</span>
						</div>
					</CardContent>
				</Card>

				<!-- Tags -->
				{#if tags && tags.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Tag class="h-4 w-4" />
								Tags
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="flex flex-wrap gap-2">
								{#each tags as tag}
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
				<p class="text-muted-foreground">Deviation not found</p>
			</CardContent>
		</Card>
	{/if}
</div>

<!-- Approve Modal -->
{#if showApproveModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<Card class="w-full max-w-md">
			<CardHeader>
				<CardTitle>Approve Deviation</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="space-y-2">
					<Label for="approver">Approver Name</Label>
					<Input
						id="approver"
						bind:value={approverName}
						placeholder="Enter approver name"
					/>
				</div>
				<div class="flex justify-end gap-2">
					<Button
						variant="outline"
						onclick={() => { showApproveModal = false; }}
						disabled={actionLoading}
					>
						Cancel
					</Button>
					<Button
						onclick={handleApprove}
						disabled={actionLoading || !approverName.trim()}
					>
						{actionLoading ? 'Approving...' : 'Approve'}
					</Button>
				</div>
			</CardContent>
		</Card>
	</div>
{/if}

<!-- Reject Modal -->
{#if showRejectModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<Card class="w-full max-w-md">
			<CardHeader>
				<CardTitle>Reject Deviation</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="space-y-2">
					<Label for="reason">Reason (optional)</Label>
					<Input
						id="reason"
						bind:value={rejectReason}
						placeholder="Enter rejection reason"
					/>
				</div>
				<div class="flex justify-end gap-2">
					<Button
						variant="outline"
						onclick={() => { showRejectModal = false; }}
						disabled={actionLoading}
					>
						Cancel
					</Button>
					<Button
						variant="destructive"
						onclick={handleReject}
						disabled={actionLoading}
					>
						{actionLoading ? 'Rejecting...' : 'Reject'}
					</Button>
				</div>
			</CardContent>
		</Card>
	</div>
{/if}
