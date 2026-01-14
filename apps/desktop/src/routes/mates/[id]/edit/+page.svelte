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
	let mateType = $state('clearance');
	let description = $state('');
	let notes = $state('');
	let tags = $state<string[]>([]);
	let newTag = $state('');

	// Component and feature selection
	let components = $state<EntityData[]>([]);
	let allFeatures = $state<EntityData[]>([]);
	let featuresA = $state<EntityData[]>([]);
	let featuresB = $state<EntityData[]>([]);
	let componentAId = $state('');
	let componentBId = $state('');
	let featureAId = $state('');
	let featureBId = $state('');
	let loadingComponents = $state(true);
	let loadingFeaturesA = $state(false);
	let loadingFeaturesB = $state(false);
	let initialLoadComplete = $state(false);

	async function loadComponentsAndFeatures() {
		if (!$isProjectOpen) return;
		loadingComponents = true;
		try {
			const [compResult, featResult] = await Promise.all([
				entities.list('CMP'),
				entities.list('FEAT')
			]);
			components = compResult.items;
			allFeatures = featResult.items;
		} catch (e) {
			console.error('Failed to load components/features:', e);
		} finally {
			loadingComponents = false;
		}
	}

	function filterFeaturesByComponent(componentId: string): EntityData[] {
		if (!componentId) return [];
		return allFeatures.filter(f => {
			const data = f.data ?? {};
			return data.component === componentId;
		});
	}

	function findComponentForFeature(featureId: string): string {
		const feature = allFeatures.find(f => f.id === featureId);
		if (feature) {
			const data = feature.data ?? {};
			return (data.component as string) ?? '';
		}
		return '';
	}

	async function loadData() {
		if (!$isProjectOpen || !id) return;

		loading = true;
		error = null;

		try {
			// Load components and features first
			await loadComponentsAndFeatures();

			const result = await entities.get(id);
			if (result) {
				entity = result;
				title = result.title;
				tags = [...result.tags];

				const data = result.data ?? {};
				mateType = (data.mate_type as string) ?? 'clearance';
				description = (data.description as string) ?? '';
				notes = (data.notes as string) ?? '';

				const featureA = data.feature_a as { id?: string } | null;
				const featureB = data.feature_b as { id?: string } | null;
				featureAId = featureA?.id ?? '';
				featureBId = featureB?.id ?? '';

				// Find which components these features belong to
				if (featureAId) {
					componentAId = findComponentForFeature(featureAId);
					featuresA = filterFeaturesByComponent(componentAId);
				}
				if (featureBId) {
					componentBId = findComponentForFeature(featureBId);
					featuresB = filterFeaturesByComponent(componentBId);
				}

				initialLoadComplete = true;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function handleComponentAChange() {
		if (!initialLoadComplete) return;
		featuresA = filterFeaturesByComponent(componentAId);
		featureAId = '';
	}

	function handleComponentBChange() {
		if (!initialLoadComplete) return;
		featuresB = filterFeaturesByComponent(componentBId);
		featureBId = '';
	}

	function addTag() {
		if (newTag.trim() && !tags.includes(newTag.trim())) {
			tags = [...tags, newTag.trim()];
			newTag = '';
		}
	}

	async function handleSubmit() {
		if (!title.trim()) { error = 'Title is required'; return; }
		if (!featureAId) { error = 'Feature A is required'; return; }
		if (!featureBId) { error = 'Feature B is required'; return; }

		saving = true;
		error = null;

		try {
			const data: Record<string, unknown> = {
				title: title.trim(),
				mate_type: mateType,
				feature_a: { id: featureAId },
				feature_b: { id: featureBId },
				author: $projectAuthor,
				tags
			};

			if (description.trim()) data.description = description.trim();
			if (notes.trim()) data.notes = notes.trim();

			await entities.save('MATE', { ...data, id });
			await refreshProject();
			goto(`/mates/${id}`);
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
			<Button variant="ghost" size="sm" onclick={() => goto(`/mates/${id}`)}>
				<ArrowLeft class="mr-2 h-4 w-4" />Back to Mate
			</Button>
			<div>
				<h1 class="text-2xl font-bold">Edit Mate</h1>
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
						<CardHeader><CardTitle>Mate Information</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="space-y-2">
								<Label for="title">Title *</Label>
								<Input id="title" bind:value={title} placeholder="Mate name (e.g., Pin-to-Hole Fit)" required />
							</div>
							<div class="space-y-2">
								<Label for="mate-type">Mate Type *</Label>
								<Select id="mate-type" bind:value={mateType}>
									<option value="clearance">Clearance Fit</option>
									<option value="transition">Transition Fit</option>
									<option value="interference">Interference Fit</option>
								</Select>
							</div>
							<div class="space-y-2">
								<Label for="description">Description</Label>
								<Textarea id="description" bind:value={description} placeholder="Describe the mate" rows={2} />
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Feature A (Hole/Bore)</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<p class="text-sm text-muted-foreground">Select the internal feature (typically a hole or bore)</p>
							<div class="grid gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="component-a">Component *</Label>
									<Select id="component-a" bind:value={componentAId} disabled={loadingComponents} onchange={handleComponentAChange}>
										<option value="">Select a component...</option>
										{#each components as comp}
											<option value={comp.id}>{comp.title} ({comp.id.slice(0, 12)}...)</option>
										{/each}
									</Select>
								</div>
								<div class="space-y-2">
									<Label for="feature-a">Feature *</Label>
									<Select id="feature-a" bind:value={featureAId} disabled={!componentAId}>
										<option value="">{componentAId ? 'Select a feature...' : 'Select component first'}</option>
										{#each featuresA as feat}
											<option value={feat.id}>{feat.title}</option>
										{/each}
									</Select>
									{#if componentAId && featuresA.length === 0}
										<p class="text-xs text-muted-foreground">No features found for this component</p>
									{/if}
								</div>
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Feature B (Shaft/Pin)</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<p class="text-sm text-muted-foreground">Select the external feature (typically a shaft or pin)</p>
							<div class="grid gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="component-b">Component *</Label>
									<Select id="component-b" bind:value={componentBId} disabled={loadingComponents} onchange={handleComponentBChange}>
										<option value="">Select a component...</option>
										{#each components as comp}
											<option value={comp.id}>{comp.title} ({comp.id.slice(0, 12)}...)</option>
										{/each}
									</Select>
								</div>
								<div class="space-y-2">
									<Label for="feature-b">Feature *</Label>
									<Select id="feature-b" bind:value={featureBId} disabled={!componentBId}>
										<option value="">{componentBId ? 'Select a feature...' : 'Select component first'}</option>
										{#each featuresB as feat}
											<option value={feat.id}>{feat.title}</option>
										{/each}
									</Select>
									{#if componentBId && featuresB.length === 0}
										<p class="text-xs text-muted-foreground">No features found for this component</p>
									{/if}
								</div>
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Notes</CardTitle></CardHeader>
						<CardContent>
							<Textarea bind:value={notes} placeholder="Additional notes about this mate" rows={3} />
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
								<Button type="button" variant="outline" onclick={() => goto(`/mates/${id}`)}>Cancel</Button>
							</div>
						</CardContent>
					</Card>
				</div>
			</div>
		</form>
	{:else}
		<Card><CardContent class="flex h-64 items-center justify-center"><p class="text-muted-foreground">Mate not found</p></CardContent></Card>
	{/if}
</div>
