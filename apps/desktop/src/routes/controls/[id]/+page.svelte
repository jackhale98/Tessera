<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { entities, traceability } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import { Crosshair, User, Calendar, Tag, Target, Gauge, AlertTriangle, Activity } from 'lucide-svelte';

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
	const controlType = $derived((data.control_type as string) ?? 'inspection');
	const controlCategory = $derived((data.control_category as string) ?? 'attribute');
	const description = $derived((data.description as string) ?? null);
	const reactionPlan = $derived((data.reaction_plan as string) ?? null);
	const entityRevision = $derived((data.entity_revision as number) ?? 1);

	// Characteristic
	interface Characteristic {
		name: string;
		nominal?: number;
		upper_limit?: number;
		lower_limit?: number;
		units?: string;
		critical: boolean;
	}
	const characteristic = $derived(data.characteristic as Characteristic | null);

	// Measurement
	interface Measurement {
		method?: string;
		equipment?: string;
		gage_rr_percent?: number;
	}
	const measurement = $derived(data.measurement as Measurement | null);

	// Sampling
	interface Sampling {
		sampling_type: string;
		frequency?: string;
		sample_size?: number;
	}
	const sampling = $derived(data.sampling as Sampling | null);

	// Control limits
	interface ControlLimits {
		ucl?: number;
		lcl?: number;
		target?: number;
	}
	const controlLimits = $derived(data.control_limits as ControlLimits | null);

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
			console.error('Failed to load control:', e);
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

	function getControlTypeLabel(type: string): string {
		const labels: Record<string, string> = {
			spc: 'SPC',
			inspection: 'Inspection',
			poka_yoke: 'Poka-Yoke',
			pokayoke: 'Poka-Yoke',
			visual: 'Visual',
			functional_test: 'Functional Test',
			functionaltest: 'Functional Test',
			attribute: 'Attribute'
		};
		return labels[type.toLowerCase()] ?? type;
	}

	function getSamplingTypeLabel(type: string): string {
		const labels: Record<string, string> = {
			continuous: 'Continuous',
			periodic: 'Periodic',
			lot: 'Lot',
			first_article: 'First Article',
			firstarticle: 'First Article'
		};
		return labels[type.toLowerCase()] ?? type;
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
			subtitle={getControlTypeLabel(controlType)}
			backHref="/controls"
			backLabel="Controls"
			onEdit={() => goto(`/controls/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Description -->
				{#if description}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Crosshair class="h-5 w-5" />
								Description
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap">{description}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Characteristic -->
				{#if characteristic}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Target class="h-5 w-5" />
								Characteristic
								{#if characteristic.critical}
									<Badge variant="destructive">Critical</Badge>
								{/if}
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="mb-4">
								<h4 class="text-lg font-medium">{characteristic.name}</h4>
							</div>
							{#if characteristic.nominal !== undefined || characteristic.upper_limit !== undefined || characteristic.lower_limit !== undefined}
								<div class="grid gap-4 sm:grid-cols-3">
									{#if characteristic.lower_limit !== undefined}
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">LSL</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{characteristic.lower_limit} {characteristic.units ?? ''}
											</div>
										</div>
									{/if}
									{#if characteristic.nominal !== undefined}
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">Nominal</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{characteristic.nominal} {characteristic.units ?? ''}
											</div>
										</div>
									{/if}
									{#if characteristic.upper_limit !== undefined}
										<div class="rounded-lg border p-4 text-center">
											<div class="text-sm text-muted-foreground">USL</div>
											<div class="mt-1 font-mono text-lg font-bold">
												{characteristic.upper_limit} {characteristic.units ?? ''}
											</div>
										</div>
									{/if}
								</div>
							{/if}
						</CardContent>
					</Card>
				{/if}

				<!-- Measurement Method -->
				{#if measurement && (measurement.method || measurement.equipment || measurement.gage_rr_percent)}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Gauge class="h-5 w-5" />
								Measurement
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if measurement.method}
								<div>
									<h4 class="text-sm text-muted-foreground">Method</h4>
									<p class="mt-1">{measurement.method}</p>
								</div>
							{/if}
							{#if measurement.equipment}
								<div>
									<h4 class="text-sm text-muted-foreground">Equipment</h4>
									<p class="mt-1">{measurement.equipment}</p>
								</div>
							{/if}
							{#if measurement.gage_rr_percent !== undefined}
								<div>
									<h4 class="text-sm text-muted-foreground">Gage R&R</h4>
									<p class="mt-1 font-mono text-lg font-bold">
										{measurement.gage_rr_percent.toFixed(1)}%
									</p>
								</div>
							{/if}
						</CardContent>
					</Card>
				{/if}

				<!-- Control Limits (for SPC) -->
				{#if controlLimits && (controlLimits.ucl !== undefined || controlLimits.lcl !== undefined || controlLimits.target !== undefined)}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Activity class="h-5 w-5" />
								Control Limits
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="grid gap-4 sm:grid-cols-3">
								{#if controlLimits.lcl !== undefined}
									<div class="rounded-lg border p-4 text-center">
										<div class="text-sm text-muted-foreground">LCL</div>
										<div class="mt-1 font-mono text-lg font-bold text-red-500">
											{controlLimits.lcl}
										</div>
									</div>
								{/if}
								{#if controlLimits.target !== undefined}
									<div class="rounded-lg border p-4 text-center">
										<div class="text-sm text-muted-foreground">Target</div>
										<div class="mt-1 font-mono text-lg font-bold text-green-500">
											{controlLimits.target}
										</div>
									</div>
								{/if}
								{#if controlLimits.ucl !== undefined}
									<div class="rounded-lg border p-4 text-center">
										<div class="text-sm text-muted-foreground">UCL</div>
										<div class="mt-1 font-mono text-lg font-bold text-red-500">
											{controlLimits.ucl}
										</div>
									</div>
								{/if}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Reaction Plan -->
				{#if reactionPlan}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<AlertTriangle class="h-5 w-5" />
								Reaction Plan
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p class="whitespace-pre-wrap">{reactionPlan}</p>
						</CardContent>
					</Card>
				{/if}

				<!-- Links -->
				<LinksSection {linksFrom} {linksTo} loading={linksLoading} />
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Control Info -->
				<Card>
					<CardHeader>
						<CardTitle>Control Information</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Type</span>
							<Badge variant="outline">{getControlTypeLabel(controlType)}</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Category</span>
							<Badge variant="secondary" class="capitalize">{controlCategory}</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						{#if characteristic?.critical}
							<div class="flex items-center gap-2 text-destructive">
								<AlertTriangle class="h-4 w-4" />
								<span class="text-sm font-medium">Critical Characteristic</span>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Sampling -->
				{#if sampling}
					<Card>
						<CardHeader>
							<CardTitle>Sampling Plan</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Type</span>
								<Badge variant="outline">{getSamplingTypeLabel(sampling.sampling_type)}</Badge>
							</div>
							{#if sampling.frequency}
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">Frequency</span>
									<span class="font-medium">{sampling.frequency}</span>
								</div>
							{/if}
							{#if sampling.sample_size}
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">Sample Size</span>
									<span class="font-medium">{sampling.sample_size}</span>
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
				<p class="text-muted-foreground">Control not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
