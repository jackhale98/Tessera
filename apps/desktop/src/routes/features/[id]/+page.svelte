<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { entities, traceability } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import { CircleDot, User, Calendar, Tag, Box, Ruler, FileText } from 'lucide-svelte';

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
	const component = $derived((data.component as string) ?? '');
	const featureType = $derived((data.feature_type as string) ?? 'external');
	const description = $derived((data.description as string) ?? null);
	const geometryClass = $derived((data.geometry_class as string) ?? null);
	const datumLabel = $derived((data.datum_label as string) ?? null);
	const entityRevision = $derived((data.entity_revision as number) ?? 1);

	// Dimensions
	interface Dimension {
		name: string;
		nominal: number;
		plus_tol: number;
		minus_tol: number;
		units: string;
		internal: boolean;
		distribution?: string;
	}
	const dimensions = $derived((data.dimensions as Dimension[]) ?? []);

	// GD&T Controls
	interface GdtControl {
		symbol: string;
		value: number;
		units: string;
		datum_refs?: string[];
		material_condition?: string;
	}
	const gdt = $derived((data.gdt as GdtControl[]) ?? []);

	// Drawing reference
	interface DrawingRef {
		number: string;
		revision: string;
		zone: string;
	}
	const drawing = $derived(data.drawing as DrawingRef | null);

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
			console.error('Failed to load feature:', e);
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

	function formatTolerance(dim: Dimension): string {
		const plusStr = dim.plus_tol >= 0 ? `+${dim.plus_tol}` : dim.plus_tol.toString();
		const minusStr = dim.minus_tol >= 0 ? `-${dim.minus_tol}` : dim.minus_tol.toString();
		return `${dim.nominal} ${plusStr}/${minusStr} ${dim.units}`;
	}

	function getGdtSymbolLabel(symbol: string): string {
		const labels: Record<string, string> = {
			position: 'Position',
			flatness: 'Flatness',
			perpendicularity: 'Perpendicularity',
			parallelism: 'Parallelism',
			concentricity: 'Concentricity',
			cylindricity: 'Cylindricity',
			circularity: 'Circularity',
			straightness: 'Straightness',
			angularity: 'Angularity',
			profile_surface: 'Profile of a Surface',
			profile_line: 'Profile of a Line',
			runout: 'Runout',
			total_runout: 'Total Runout',
			symmetry: 'Symmetry'
		};
		return labels[symbol.toLowerCase()] ?? symbol;
	}

	function getMaterialConditionLabel(mc: string | undefined): string {
		if (!mc) return '';
		const labels: Record<string, string> = {
			mmc: 'MMC',
			lmc: 'LMC',
			rfs: 'RFS'
		};
		return labels[mc.toLowerCase()] ?? mc;
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
			subtitle={`${featureType === 'internal' ? 'Internal' : 'External'} Feature`}
			backHref="/features"
			backLabel="Features"
			onEdit={() => goto(`/features/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Description -->
				{#if description}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<CircleDot class="h-5 w-5" />
								Description
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap">{description}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Dimensions -->
				{#if dimensions.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Ruler class="h-5 w-5" />
								Dimensions ({dimensions.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-3">
								{#each dimensions as dim}
									<div class="rounded-lg border p-4">
										<div class="flex items-center justify-between">
											<div>
												<p class="font-medium">{dim.name}</p>
												<p class="mt-1 font-mono text-lg">{formatTolerance(dim)}</p>
											</div>
											<div class="flex items-center gap-2">
												<Badge variant={dim.internal ? 'secondary' : 'outline'}>
													{dim.internal ? 'Internal' : 'External'}
												</Badge>
												{#if dim.distribution}
													<Badge variant="outline" class="capitalize">
														{dim.distribution}
													</Badge>
												{/if}
											</div>
										</div>
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- GD&T Controls -->
				{#if gdt.length > 0}
					<Card>
						<CardHeader>
							<CardTitle>GD&T Controls ({gdt.length})</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-3">
								{#each gdt as control}
									<div class="rounded-lg border p-4">
										<div class="flex items-center justify-between">
											<div>
												<p class="font-medium">{getGdtSymbolLabel(control.symbol)}</p>
												<p class="mt-1 font-mono text-lg">
													{control.value} {control.units}
													{#if control.material_condition}
														<span class="ml-2 text-sm">
															({getMaterialConditionLabel(control.material_condition)})
														</span>
													{/if}
												</p>
											</div>
											{#if control.datum_refs && control.datum_refs.length > 0}
												<div class="flex items-center gap-1">
													<span class="text-sm text-muted-foreground">Datums:</span>
													{#each control.datum_refs as datum}
														<Badge variant="outline">{datum}</Badge>
													{/each}
												</div>
											{/if}
										</div>
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
				<!-- Feature Info -->
				<Card>
					<CardHeader>
						<CardTitle>Feature Information</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Component</span>
							<button
								class="font-mono text-sm text-primary hover:underline"
								onclick={() => goto(`/components/${component}`)}
							>
								{component || '—'}
							</button>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Type</span>
							<Badge variant={featureType === 'internal' ? 'secondary' : 'outline'}>
								{featureType === 'internal' ? 'Internal' : 'External'}
							</Badge>
						</div>
						{#if geometryClass}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Geometry</span>
								<Badge variant="outline" class="capitalize">{geometryClass}</Badge>
							</div>
						{/if}
						{#if datumLabel}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Datum Label</span>
								<Badge variant="default">{datumLabel}</Badge>
							</div>
						{/if}
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
					</CardContent>
				</Card>

				<!-- Drawing Reference -->
				{#if drawing}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<FileText class="h-4 w-4" />
								Drawing Reference
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Drawing</span>
								<span class="font-medium">{drawing.number}</span>
							</div>
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Revision</span>
								<span class="font-medium">{drawing.revision}</span>
							</div>
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Zone</span>
								<Badge variant="outline">{drawing.zone}</Badge>
							</div>
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
							<span class="ml-auto text-sm font-medium">{entity.author}</span>
						</div>
						<div class="flex items-center gap-2">
							<Calendar class="h-4 w-4 text-muted-foreground" />
							<span class="text-sm text-muted-foreground">Created</span>
							<span class="ml-auto text-sm font-medium">{formatDate(entity.created)}</span>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Entity Revision</span>
							<span class="text-sm font-medium">{entityRevision}</span>
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
				<p class="text-muted-foreground">Feature not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
