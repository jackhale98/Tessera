<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import {
		Card,
		CardContent,
		CardHeader,
		CardTitle,
		Button,
		Input,
		Label,
		Textarea,
		Select
	} from '$lib/components/ui';
	import { entities } from '$lib/api';
	import { refreshProject, isProjectOpen, projectAuthor } from '$lib/stores/project';
	import { ArrowLeft, Plus, X } from 'lucide-svelte';
	import type { EntityData } from '$lib/api/types';

	const id = $derived($page.params.id);

	let entity = $state<EntityData | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let error = $state<string | null>(null);

	// Form fields
	let title = $state('');
	let componentId = $state('');
	let featureType = $state('external');
	let description = $state('');
	let geometryClass = $state('');
	let datumLabel = $state('');
	let drawingNumber = $state('');
	let drawingRevision = $state('');
	let drawingZone = $state('');
	let tags = $state<string[]>([]);
	let newTag = $state('');

	async function loadData() {
		if (!$isProjectOpen || !id) return;

		loading = true;
		error = null;

		try {
			const result = await entities.get(id);
			if (result) {
				entity = result;
				title = result.title;
				tags = [...result.tags];

				const data = result.data ?? {};
				componentId = (data.component as string) ?? '';
				featureType = (data.feature_type as string) ?? 'external';
				description = (data.description as string) ?? '';
				geometryClass = (data.geometry_class as string) ?? '';
				datumLabel = (data.datum_label as string) ?? '';

				const drawing = data.drawing as { number?: string; revision?: string; zone?: string } | null;
				if (drawing) {
					drawingNumber = drawing.number ?? '';
					drawingRevision = drawing.revision ?? '';
					drawingZone = drawing.zone ?? '';
				}
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function addTag() {
		if (newTag.trim() && !tags.includes(newTag.trim())) {
			tags = [...tags, newTag.trim()];
			newTag = '';
		}
	}

	async function handleSubmit() {
		if (!title.trim()) { error = 'Title is required'; return; }
		if (!componentId.trim()) { error = 'Component ID is required'; return; }

		saving = true;
		error = null;

		try {
			// Start with existing entity data to preserve all fields (dimensions, gd_t, bounds, etc.)
			const existingData = entity?.data ?? {};

			// Build updated data, preserving existing fields
			const data: Record<string, unknown> = {
				...existingData,
				id,
				title: title.trim(),
				component: componentId.trim(),
				feature_type: featureType,
				author: entity?.author ?? $projectAuthor,
				tags,
				status: entity?.status ?? 'draft',
				created: entity?.created ?? new Date().toISOString(),
				entity_revision: ((existingData.entity_revision as number) ?? 0) + 1
			};

			if (description.trim()) {
				data.description = description.trim();
			} else {
				delete data.description;
			}
			if (geometryClass) {
				data.geometry_class = geometryClass;
			} else {
				delete data.geometry_class;
			}
			if (datumLabel.trim()) {
				data.datum_label = datumLabel.trim().toUpperCase();
			} else {
				delete data.datum_label;
			}
			if (drawingNumber.trim()) {
				data.drawing = {
					number: drawingNumber.trim(),
					revision: drawingRevision.trim() || 'A',
					zone: drawingZone.trim() || ''
				};
			} else {
				delete data.drawing;
			}

			await entities.save('FEAT', data);
			await refreshProject();
			goto(`/features/${id}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}

	onMount(() => {
		loadData();
	});

	$effect(() => {
		if ($isProjectOpen && id) {
			loadData();
		}
	});
</script>

<div class="space-y-6">
	{#if loading}
		<div class="flex h-64 items-center justify-center">
			<div class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
		</div>
	{:else if entity}
		<div class="space-y-4">
			<Button variant="ghost" size="sm" onclick={() => goto(`/features/${id}`)}>
				<ArrowLeft class="mr-2 h-4 w-4" />Back to Feature
			</Button>
			<div>
				<h1 class="text-2xl font-bold">Edit Feature</h1>
				<p class="text-muted-foreground font-mono text-sm">{id}</p>
			</div>
		</div>

		{#if error}
			<Card class="border-destructive"><CardContent class="pt-6"><p class="text-destructive">{error}</p></CardContent></Card>
		{/if}

		<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
			<div class="grid gap-6 lg:grid-cols-3">
				<div class="space-y-6 lg:col-span-2">
					<Card>
						<CardHeader><CardTitle>Feature Information</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="space-y-2">
								<Label for="title">Title *</Label>
								<Input id="title" bind:value={title} placeholder="Feature name (e.g., Mounting Hole A)" required />
							</div>
							<div class="grid gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="component">Component ID *</Label>
									<Input id="component" bind:value={componentId} placeholder="CMP-..." required />
									<p class="text-xs text-muted-foreground">Enter the parent component's ID</p>
								</div>
								<div class="space-y-2">
									<Label for="feature-type">Feature Type *</Label>
									<Select id="feature-type" bind:value={featureType}>
										<option value="internal">Internal (Hole/Bore)</option>
										<option value="external">External (Shaft/Boss)</option>
									</Select>
								</div>
							</div>
							<div class="space-y-2">
								<Label for="description">Description</Label>
								<Textarea id="description" bind:value={description} placeholder="Feature description" rows={2} />
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Geometry</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="grid gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="geometry-class">Geometry Class</Label>
									<Select id="geometry-class" bind:value={geometryClass}>
										<option value="">Not specified</option>
										<option value="plane">Plane</option>
										<option value="cylinder">Cylinder</option>
										<option value="sphere">Sphere</option>
										<option value="cone">Cone</option>
										<option value="point">Point</option>
										<option value="line">Line</option>
										<option value="complex">Complex</option>
									</Select>
								</div>
								<div class="space-y-2">
									<Label for="datum-label">Datum Label</Label>
									<Input id="datum-label" bind:value={datumLabel} placeholder="e.g., A, B, C" maxlength={3} />
									<p class="text-xs text-muted-foreground">If this feature is a datum</p>
								</div>
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Drawing Reference</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="grid gap-4 sm:grid-cols-3">
								<div class="space-y-2">
									<Label for="drawing-number">Drawing Number</Label>
									<Input id="drawing-number" bind:value={drawingNumber} placeholder="DWG-001" />
								</div>
								<div class="space-y-2">
									<Label for="drawing-revision">Revision</Label>
									<Input id="drawing-revision" bind:value={drawingRevision} placeholder="A" />
								</div>
								<div class="space-y-2">
									<Label for="drawing-zone">Zone</Label>
									<Input id="drawing-zone" bind:value={drawingZone} placeholder="B3" />
								</div>
							</div>
						</CardContent>
					</Card>
				</div>

				<div class="space-y-6">
					<Card>
						<CardHeader><CardTitle>Tags</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="flex gap-2">
								<Input bind:value={newTag} placeholder="Add tag" onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addTag())} />
								<Button type="button" variant="outline" onclick={addTag}><Plus class="h-4 w-4" /></Button>
							</div>
							{#if tags.length > 0}
								<div class="flex flex-wrap gap-2">
									{#each tags as tag}
										<span class="inline-flex items-center gap-1 rounded-full border px-3 py-1 text-sm">{tag}<button type="button" class="text-muted-foreground hover:text-foreground" onclick={() => tags = tags.filter(t => t !== tag)}><X class="h-3 w-3" /></button></span>
									{/each}
								</div>
							{/if}
						</CardContent>
					</Card>

					<Card>
						<CardContent class="pt-6">
							<div class="flex flex-col gap-2">
								<Button type="submit" disabled={saving}>{saving ? 'Saving...' : 'Save Changes'}</Button>
								<Button type="button" variant="outline" onclick={() => goto(`/features/${id}`)}>Cancel</Button>
							</div>
						</CardContent>
					</Card>
				</div>
			</div>
		</form>
	{:else}
		<Card><CardContent class="flex h-64 items-center justify-center"><p class="text-muted-foreground">Feature not found</p></CardContent></Card>
	{/if}
</div>
