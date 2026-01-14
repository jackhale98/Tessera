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
		Package,
		User,
		Calendar,
		Tag,
		FileText,
		Hash,
		Clock,
		CheckCircle2,
		Box
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
	const lotNumber = $derived((data.lot_number as string) ?? null);
	const lotStatus = $derived((data.lot_status as string) ?? 'in_progress');
	const quantity = $derived(data.quantity as number | null);
	const startDate = $derived((data.start_date as string) ?? null);
	const endDate = $derived((data.end_date as string) ?? null);
	const description = $derived((data.description as string) ?? '');
	const revision = $derived((data.revision as number) ?? 1);

	interface Material {
		name: string;
		lot?: string;
		quantity?: number;
		unit?: string;
	}
	const materialsUsed = $derived((data.materials_used as Material[]) ?? []);

	interface Execution {
		step: string;
		completed: boolean;
		operator?: string;
		timestamp?: string;
	}
	const executionRecords = $derived((data.execution_records as Execution[]) ?? []);

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
			pending_review: 'Pending Review',
			released: 'Released',
			quarantine: 'Quarantine',
			rejected: 'Rejected'
		};
		return statuses[status.toLowerCase()] ?? status;
	}

	function getLotStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			in_progress: 'secondary',
			pending_review: 'outline',
			released: 'default',
			quarantine: 'destructive',
			rejected: 'destructive'
		};
		return variants[status.toLowerCase()] ?? 'outline';
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
			subtitle={lotNumber ? `Lot: ${lotNumber}` : 'Production Lot'}
			backHref="/manufacturing/lots"
			backLabel="Lots"
			onEdit={() => goto(`/manufacturing/lots/${id}/edit`)}
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

				<!-- Execution Records -->
				{#if executionRecords.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<CheckCircle2 class="h-5 w-5" />
								Execution Records ({executionRecords.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-3">
								{#each executionRecords as record, i}
									<div class="flex items-center gap-4 rounded-lg border p-3">
										<div class="flex h-8 w-8 items-center justify-center rounded-full {record.completed ? 'bg-green-500/10 text-green-500' : 'bg-muted text-muted-foreground'}">
											{#if record.completed}
												<CheckCircle2 class="h-4 w-4" />
											{:else}
												<span class="text-sm font-medium">{i + 1}</span>
											{/if}
										</div>
										<div class="flex-1">
											<p class="font-medium">{record.step}</p>
											{#if record.operator || record.timestamp}
												<p class="text-sm text-muted-foreground">
													{#if record.operator}by {record.operator}{/if}
													{#if record.timestamp} on {formatDate(record.timestamp)}{/if}
												</p>
											{/if}
										</div>
										<Badge variant={record.completed ? 'default' : 'outline'}>
											{record.completed ? 'Complete' : 'Pending'}
										</Badge>
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
											<th class="px-4 py-2 text-left font-medium">Material</th>
											<th class="px-4 py-2 text-left font-medium">Lot</th>
											<th class="px-4 py-2 text-left font-medium">Quantity</th>
										</tr>
									</thead>
									<tbody>
										{#each materialsUsed as material}
											<tr class="border-b">
												<td class="px-4 py-2 font-medium">{material.name}</td>
												<td class="px-4 py-2 font-mono text-xs">{material.lot ?? '—'}</td>
												<td class="px-4 py-2 text-muted-foreground">
													{#if material.quantity}
														{material.quantity} {material.unit ?? ''}
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
						{#if quantity}
							<div class="flex items-center justify-between">
								<span class="text-sm text-muted-foreground">Quantity</span>
								<div class="flex items-center gap-1">
									<Hash class="h-3 w-3 text-muted-foreground" />
									<span class="text-sm font-medium">{quantity}</span>
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
						{#if endDate}
							<div class="flex items-center gap-2">
								<Calendar class="h-4 w-4 text-muted-foreground" />
								<span class="text-sm text-muted-foreground">End</span>
								<span class="ml-auto text-sm font-medium">{formatDate(endDate)}</span>
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
				<p class="text-muted-foreground">Lot not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
