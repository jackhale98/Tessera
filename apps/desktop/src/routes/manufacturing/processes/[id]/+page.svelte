<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { entities, traceability } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import EntityHistory from '$lib/components/EntityHistory.svelte';
	import {
		Cog,
		User,
		Calendar,
		Tag,
		Settings,
		Clock,
		TrendingUp,
		FileText,
		History
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
	const processType = $derived((data.process_type as string) ?? 'manufacturing');
	const description = $derived((data.description as string) ?? '');
	const cycleTime = $derived((data.cycle_time as number) ?? null);
	const revision = $derived((data.revision as number) ?? 1);

	interface Parameter {
		name: string;
		value?: string | number;
		unit?: string;
		min?: number;
		max?: number;
	}
	const parameters = $derived((data.parameters as Parameter[]) ?? []);

	interface Capability {
		cpk?: number;
		ppk?: number;
		sigma?: number;
	}
	const capability = $derived(data.capability as Capability | null);

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
			console.error('Failed to load process:', e);
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

	function formatProcessType(type: string): string {
		const types: Record<string, string> = {
			manufacturing: 'Manufacturing',
			assembly: 'Assembly',
			inspection: 'Inspection',
			packaging: 'Packaging',
			testing: 'Testing'
		};
		return types[type.toLowerCase()] ?? type;
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
			id={entity.id}
			title={entity.title}
			status={entity.status}
			subtitle={formatProcessType(processType)}
			backHref="/manufacturing/processes"
			backLabel="Processes"
			onEdit={() => goto(`/manufacturing/processes/${id}/edit`)}
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

				<!-- Parameters -->
				{#if parameters.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Settings class="h-5 w-5" />
								Parameters ({parameters.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="overflow-x-auto">
								<table class="w-full text-sm">
									<thead>
										<tr class="border-b">
											<th class="px-4 py-2 text-left font-medium">Parameter</th>
											<th class="px-4 py-2 text-left font-medium">Value</th>
											<th class="px-4 py-2 text-left font-medium">Unit</th>
											<th class="px-4 py-2 text-left font-medium">Range</th>
										</tr>
									</thead>
									<tbody>
										{#each parameters as param}
											<tr class="border-b">
												<td class="px-4 py-2 font-medium">{param.name}</td>
												<td class="px-4 py-2">{param.value ?? '—'}</td>
												<td class="px-4 py-2 text-muted-foreground">{param.unit ?? '—'}</td>
												<td class="px-4 py-2 text-muted-foreground">
													{#if param.min !== undefined && param.max !== undefined}
														{param.min} - {param.max}
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

				<!-- Capability -->
				{#if capability && (capability.cpk || capability.ppk || capability.sigma)}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<TrendingUp class="h-5 w-5" />
								Process Capability
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="grid gap-4 sm:grid-cols-3">
								{#if capability.cpk !== undefined}
									<div class="rounded-lg border p-4 text-center">
										<div class="text-sm text-muted-foreground">Cpk</div>
										<div class="mt-1 text-2xl font-bold">{capability.cpk.toFixed(2)}</div>
									</div>
								{/if}
								{#if capability.ppk !== undefined}
									<div class="rounded-lg border p-4 text-center">
										<div class="text-sm text-muted-foreground">Ppk</div>
										<div class="mt-1 text-2xl font-bold">{capability.ppk.toFixed(2)}</div>
									</div>
								{/if}
								{#if capability.sigma !== undefined}
									<div class="rounded-lg border p-4 text-center">
										<div class="text-sm text-muted-foreground">Sigma</div>
										<div class="mt-1 text-2xl font-bold">{capability.sigma.toFixed(1)}σ</div>
									</div>
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
							<span class="text-sm text-muted-foreground">Type</span>
							<Badge variant="outline">{formatProcessType(processType)}</Badge>
						</div>
						{#if cycleTime}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Cycle Time</span>
								<div class="flex items-center gap-1">
									<Clock class="h-3 w-3 text-muted-foreground" />
									<span class="text-sm font-medium">{cycleTime} min</span>
								</div>
							</div>
						{/if}
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
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
				<p class="text-muted-foreground">Process not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
