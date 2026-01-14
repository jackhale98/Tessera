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
		BarChart3
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
	const mitigations = $derived((data.mitigations as Mitigation[]) ?? []);

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

						<!-- RPN Breakdown -->
						<div class="mt-6 grid gap-4 sm:grid-cols-4">
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

						<!-- Initial Risk comparison -->
						{#if initialRisk && (initialRisk.severity || initialRisk.occurrence || initialRisk.detection)}
							<div class="mt-4 rounded-lg bg-muted/50 p-4">
								<h4 class="mb-2 text-sm font-medium">Initial Risk (Before Mitigation)</h4>
								<div class="grid gap-4 text-sm sm:grid-cols-4">
									<div>
										<span class="text-muted-foreground">Severity:</span>
										<span class="ml-2 font-medium">{initialRisk.severity ?? '—'}</span>
									</div>
									<div>
										<span class="text-muted-foreground">Occurrence:</span>
										<span class="ml-2 font-medium">{initialRisk.occurrence ?? '—'}</span>
									</div>
									<div>
										<span class="text-muted-foreground">Detection:</span>
										<span class="ml-2 font-medium">{initialRisk.detection ?? '—'}</span>
									</div>
									<div>
										<span class="text-muted-foreground">RPN:</span>
										<span class="ml-2 font-medium">{initialRisk.rpn ?? '—'}</span>
									</div>
								</div>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Mitigations -->
				{#if mitigations.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Shield class="h-5 w-5" />
								Mitigations ({mitigations.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
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
						</CardContent>
					</Card>
				{/if}

				<!-- Links -->
				<LinksSection {linksFrom} {linksTo} loading={linksLoading} />
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
