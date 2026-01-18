<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge, Button } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import EntityHistory from '$lib/components/EntityHistory.svelte';
	import { entities, traceability } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import {
		AlertTriangle,
		User,
		Calendar,
		Tag,
		Shield,
		Target,
		Eye,
		Activity,
		CheckCircle2,
		Clock,
		BarChart3,
		Plus,
		Link2,
		Pencil,
		History
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
	const riskType = $derived((data.risk_type as string) ?? 'design');
	const description = $derived((data.description as string) ?? '');
	const failureMode = $derived((data.failure_mode as string) ?? null);
	const cause = $derived((data.cause as string) ?? null);
	const riskEffect = $derived((data.effect as string) ?? null);
	const severity = $derived(data.severity as number | null);
	const occurrence = $derived(data.occurrence as number | null);
	const detection = $derived(data.detection as number | null);
	const rpn = $derived(data.rpn as number | null);
	const riskLevel = $derived((data.risk_level as string) ?? null);
	const category = $derived((data.category as string) ?? null);
	const revision = $derived((data.revision as number) ?? 1);

	interface Mitigation {
		action: string;
		type?: string;
		status?: string;
		owner?: string;
		due_date?: string;
	}
	const rawMitigations = $derived((data.mitigations as Mitigation[]) ?? []);
	// Filter out empty mitigations (placeholders with no action)
	const mitigations = $derived(rawMitigations.filter(m => m.action && m.action.trim().length > 0));

	interface InitialRisk {
		severity?: number;
		occurrence?: number;
		detection?: number;
		rpn?: number;
	}
	const initialRisk = $derived(data.initial_risk as InitialRisk | null);

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
			console.error('Failed to load risk:', e);
		} finally {
			loading = false;
			linksLoading = false;
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

	function formatRiskType(type: string): string {
		const types: Record<string, string> = {
			design: 'Design Risk',
			process: 'Process Risk',
			use: 'Use Risk',
			software: 'Software Risk'
		};
		return types[type] ?? type;
	}

	function getRiskLevelVariant(level: string | null): 'default' | 'secondary' | 'destructive' | 'outline' {
		if (!level) return 'outline';
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			low: 'outline',
			medium: 'secondary',
			high: 'default',
			critical: 'destructive'
		};
		return variants[level.toLowerCase()] ?? 'outline';
	}

	function getMitigationStatusVariant(status: string | undefined): 'default' | 'secondary' | 'destructive' | 'outline' {
		if (!status) return 'outline';
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			proposed: 'outline',
			inprogress: 'secondary',
			in_progress: 'secondary',
			completed: 'default',
			verified: 'default'
		};
		return variants[status.toLowerCase()] ?? 'outline';
	}

	function getRpnColor(rpn: number | null): string {
		if (!rpn) return 'text-muted-foreground';
		if (rpn >= 200) return 'text-destructive';
		if (rpn >= 100) return 'text-orange-500';
		if (rpn >= 50) return 'text-yellow-500';
		return 'text-green-500';
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
			subtitle={formatRiskType(riskType)}
			backHref="/risks"
			backLabel="Risks"
			onEdit={() => goto(`/risks/${id}/edit`)}
		/>

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

				<!-- FMEA Analysis -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<BarChart3 class="h-5 w-5" />
							FMEA Analysis
						</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if failureMode}
							<div>
								<h4 class="text-sm font-medium text-muted-foreground">Failure Mode</h4>
								<p class="mt-1">{failureMode}</p>
							</div>
						{/if}
						{#if cause}
							<div>
								<h4 class="text-sm font-medium text-muted-foreground">Cause</h4>
								<p class="mt-1">{cause}</p>
							</div>
						{/if}
						{#if riskEffect}
							<div>
								<h4 class="text-sm font-medium text-muted-foreground">Effect</h4>
								<p class="mt-1">{riskEffect}</p>
							</div>
						{/if}

						<!-- Risk Ratings Section -->
						{#if initialRisk && (initialRisk.severity || initialRisk.occurrence || initialRisk.detection)}
							<!-- Show both initial and residual when initial data exists -->
							<div class="mt-6 space-y-4">
								<!-- Initial Risk (Before Mitigation) -->
								<div class="rounded-lg border border-orange-200 bg-orange-50 dark:border-orange-800 dark:bg-orange-950/30 p-4">
									<h4 class="mb-3 text-sm font-semibold flex items-center gap-2">
										<AlertTriangle class="h-4 w-4 text-orange-500" />
										Initial Risk (Before Mitigation)
									</h4>
									<div class="grid gap-4 sm:grid-cols-4">
										<div class="text-center">
											<div class="text-xs text-muted-foreground">Severity</div>
											<div class="text-xl font-bold">{initialRisk.severity ?? '—'}</div>
										</div>
										<div class="text-center">
											<div class="text-xs text-muted-foreground">Occurrence</div>
											<div class="text-xl font-bold">{initialRisk.occurrence ?? '—'}</div>
										</div>
										<div class="text-center">
											<div class="text-xs text-muted-foreground">Detection</div>
											<div class="text-xl font-bold">{initialRisk.detection ?? '—'}</div>
										</div>
										<div class="text-center">
											<div class="text-xs text-muted-foreground">RPN</div>
											<div class="text-xl font-bold text-orange-600">{initialRisk.rpn ?? '—'}</div>
										</div>
									</div>
								</div>

								<!-- Residual Risk (After Mitigation) -->
								<div class="rounded-lg border border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950/30 p-4">
									<h4 class="mb-3 text-sm font-semibold flex items-center gap-2">
										<CheckCircle2 class="h-4 w-4 text-green-500" />
										Residual Risk (After Mitigation)
									</h4>
									<div class="grid gap-4 sm:grid-cols-4">
										<div class="text-center">
											<div class="text-xs text-muted-foreground">Severity</div>
											<div class="text-xl font-bold">{severity ?? '—'}</div>
											{#if initialRisk.severity && severity && severity < initialRisk.severity}
												<div class="text-xs text-green-600">-{initialRisk.severity - severity}</div>
											{/if}
										</div>
										<div class="text-center">
											<div class="text-xs text-muted-foreground">Occurrence</div>
											<div class="text-xl font-bold">{occurrence ?? '—'}</div>
											{#if initialRisk.occurrence && occurrence && occurrence < initialRisk.occurrence}
												<div class="text-xs text-green-600">-{initialRisk.occurrence - occurrence}</div>
											{/if}
										</div>
										<div class="text-center">
											<div class="text-xs text-muted-foreground">Detection</div>
											<div class="text-xl font-bold">{detection ?? '—'}</div>
											{#if initialRisk.detection && detection && detection < initialRisk.detection}
												<div class="text-xs text-green-600">-{initialRisk.detection - detection}</div>
											{/if}
										</div>
										<div class="text-center">
											<div class="text-xs text-muted-foreground">RPN</div>
											<div class={`text-xl font-bold ${getRpnColor(rpn)}`}>{rpn ?? '—'}</div>
											{#if initialRisk.rpn && rpn && rpn < initialRisk.rpn}
												<div class="text-xs text-green-600">-{initialRisk.rpn - rpn} ({Math.round((1 - rpn / initialRisk.rpn) * 100)}% reduction)</div>
											{/if}
										</div>
									</div>
								</div>
							</div>
						{:else}
							<!-- Show current risk only when no initial data -->
							<div class="mt-6">
								<h4 class="mb-3 text-sm font-medium text-muted-foreground">Current Risk Rating</h4>
								<div class="grid gap-4 sm:grid-cols-4">
									<div class="rounded-lg border p-4 text-center">
										<div class="flex items-center justify-center gap-1 text-sm text-muted-foreground">
											<Target class="h-4 w-4" />
											Severity
										</div>
										<div class="mt-1 text-2xl font-bold">{severity ?? '—'}</div>
									</div>
									<div class="rounded-lg border p-4 text-center">
										<div class="flex items-center justify-center gap-1 text-sm text-muted-foreground">
											<Activity class="h-4 w-4" />
											Occurrence
										</div>
										<div class="mt-1 text-2xl font-bold">{occurrence ?? '—'}</div>
									</div>
									<div class="rounded-lg border p-4 text-center">
										<div class="flex items-center justify-center gap-1 text-sm text-muted-foreground">
											<Eye class="h-4 w-4" />
											Detection
										</div>
										<div class="mt-1 text-2xl font-bold">{detection ?? '—'}</div>
									</div>
									<div class="rounded-lg border p-4 text-center">
										<div class="text-sm text-muted-foreground">RPN</div>
										<div class={`mt-1 text-2xl font-bold ${getRpnColor(rpn)}`}>{rpn ?? '—'}</div>
									</div>
								</div>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Mitigations / Controls -->
				<Card>
					<CardHeader class="flex flex-row items-center justify-between">
						<CardTitle class="flex items-center gap-2">
							<Shield class="h-5 w-5" />
							Mitigations & Controls
							{#if mitigations.length > 0}
								<Badge variant="secondary">{mitigations.length}</Badge>
							{/if}
						</CardTitle>
						<div class="flex items-center gap-2">
							<Button variant="outline" size="sm" onclick={() => goto(`/risks/${id}/edit`)}>
								<Pencil class="h-4 w-4 mr-1" />
								Edit Mitigations
							</Button>
							<Button variant="outline" size="sm" onclick={() => goto('/controls/new')}>
								<Plus class="h-4 w-4 mr-1" />
								New Control
							</Button>
						</div>
					</CardHeader>
					<CardContent>
						{#if mitigations.length > 0}
							<div class="space-y-4">
								{#each mitigations as mitigation, i}
									<div class="rounded-lg border p-4">
										<div class="flex items-start justify-between gap-4">
											<div class="flex-1">
												<div class="flex items-center gap-2">
													<span class="text-sm font-medium text-muted-foreground">#{i + 1}</span>
													{#if mitigation.type}
														<Badge variant="outline" class="capitalize">
															{mitigation.type}
														</Badge>
													{/if}
													{#if mitigation.status}
														<Badge variant={getMitigationStatusVariant(mitigation.status)} class="capitalize">
															{mitigation.status.replace(/_/g, ' ')}
														</Badge>
													{/if}
												</div>
												<p class="mt-2">{mitigation.action}</p>
											</div>
										</div>
										{#if mitigation.owner || mitigation.due_date}
											<div class="mt-3 flex items-center gap-4 text-sm text-muted-foreground">
												{#if mitigation.owner}
													<span class="flex items-center gap-1">
														<User class="h-3 w-3" />
														{mitigation.owner}
													</span>
												{/if}
												{#if mitigation.due_date}
													<span class="flex items-center gap-1">
														<Clock class="h-3 w-3" />
														Due: {formatDate(mitigation.due_date)}
													</span>
												{/if}
											</div>
										{/if}
									</div>
								{/each}
							</div>
						{:else}
							<div class="flex flex-col items-center justify-center py-8 text-center">
								<Shield class="h-12 w-12 text-muted-foreground/30 mb-3" />
								<p class="text-sm text-muted-foreground">No mitigations defined</p>
								<p class="text-xs text-muted-foreground/70 mt-1 mb-4">
									Add mitigations to reduce risk severity, occurrence, or improve detection
								</p>
								<div class="flex gap-2">
									<Button variant="outline" size="sm" onclick={() => goto(`/risks/${id}/edit`)}>
										<Pencil class="h-4 w-4 mr-1" />
										Add Mitigation
									</Button>
									<Button variant="outline" size="sm" onclick={() => goto('/controls/new')}>
										<Plus class="h-4 w-4 mr-1" />
										Create Control
									</Button>
								</div>
								<p class="text-xs text-muted-foreground/70 mt-3">
									Use "Add Link" below to link existing controls to this risk
								</p>
							</div>
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
						<EntityHistory entityId={entity?.id} />
					</CardContent>
				</Card>
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Risk Assessment -->
				<Card>
					<CardHeader>
						<CardTitle>Risk Assessment</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Risk Level</span>
							{#if riskLevel}
								<Badge variant={getRiskLevelVariant(riskLevel)} class="capitalize">
									{riskLevel}
								</Badge>
							{:else}
								<span class="text-sm text-muted-foreground">Not assessed</span>
							{/if}
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">RPN</span>
							<span class={`font-bold ${getRpnColor(rpn)}`}>{rpn ?? '—'}</span>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Mitigations</span>
							<Badge variant={mitigations.length > 0 ? 'default' : 'outline'}>
								{mitigations.length}
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
							<Badge variant="outline" class="capitalize">{riskType}</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						{#if category}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Category</span>
								<span class="text-sm font-medium">{category}</span>
							</div>
						{/if}
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
				<p class="text-muted-foreground">Risk not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
