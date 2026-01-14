<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { entities, traceability } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import { Zap, User, Calendar, Tag, AlertTriangle, Users, Activity } from 'lucide-svelte';

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
	const category = $derived((data.category as string) ?? 'mechanical');
	const description = $derived((data.description as string) ?? '');
	const severity = $derived((data.severity as string) ?? null);
	const potentialHarms = $derived((data.potential_harms as string[]) ?? []);
	const energyLevel = $derived((data.energy_level as string) ?? null);
	const exposureScenario = $derived((data.exposure_scenario as string) ?? null);
	const affectedPopulations = $derived((data.affected_populations as string[]) ?? []);
	const revision = $derived((data.revision as number) ?? 1);

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
			console.error('Failed to load hazard:', e);
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

	function getCategoryLabel(cat: string): string {
		const labels: Record<string, string> = {
			electrical: 'Electrical',
			mechanical: 'Mechanical',
			thermal: 'Thermal',
			chemical: 'Chemical',
			biological: 'Biological',
			radiation: 'Radiation',
			ergonomic: 'Ergonomic',
			software: 'Software',
			environmental: 'Environmental'
		};
		return labels[cat.toLowerCase()] ?? cat;
	}

	function getSeverityVariant(sev: string | null): 'default' | 'secondary' | 'destructive' | 'outline' {
		if (!sev) return 'outline';
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			negligible: 'outline',
			minor: 'secondary',
			serious: 'default',
			severe: 'destructive',
			catastrophic: 'destructive'
		};
		return variants[sev.toLowerCase()] ?? 'outline';
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
			subtitle={getCategoryLabel(category) + ' Hazard'}
			backHref="/hazards"
			backLabel="Hazards"
			onEdit={() => goto(`/hazards/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Description -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Zap class="h-5 w-5" />
							Description
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p class="whitespace-pre-wrap">{description || 'No description specified.'}</p>
					</CardContent>
				</Card>

				<!-- Potential Harms -->
				{#if potentialHarms.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<AlertTriangle class="h-5 w-5" />
								Potential Harms
							</CardTitle>
						</CardHeader>
						<CardContent>
							<ul class="list-inside list-disc space-y-2">
								{#each potentialHarms as harm}
									<li class="text-muted-foreground">{harm}</li>
								{/each}
							</ul>
						</CardContent>
					</Card>
				{/if}

				<!-- Exposure Scenario -->
				{#if exposureScenario}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Activity class="h-5 w-5" />
								Exposure Scenario
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap text-muted-foreground">{exposureScenario}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Affected Populations -->
				{#if affectedPopulations.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Users class="h-5 w-5" />
								Affected Populations
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="flex flex-wrap gap-2">
								{#each affectedPopulations as population}
									<Badge variant="outline">{population}</Badge>
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
				<!-- Hazard Assessment -->
				<Card>
					<CardHeader>
						<CardTitle>Hazard Assessment</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Category</span>
							<Badge variant="outline">{getCategoryLabel(category)}</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Severity</span>
							{#if severity}
								<Badge variant={getSeverityVariant(severity)} class="capitalize">
									{severity}
								</Badge>
							{:else}
								<span class="text-sm text-muted-foreground">Not assessed</span>
							{/if}
						</div>
						{#if energyLevel}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Energy Level</span>
								<span class="font-medium">{energyLevel}</span>
							</div>
						{/if}
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
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
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Revision</span>
							<span class="text-sm font-medium">{revision}</span>
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
				<p class="text-muted-foreground">Hazard not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
