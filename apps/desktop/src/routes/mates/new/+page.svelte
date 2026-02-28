<script lang="ts">
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
	import { EntityPicker } from '$lib/components/common';
	import { entities } from '$lib/api';
	import { refreshProject, projectAuthor, isProjectOpen } from '$lib/stores/project';
	import { ArrowLeft, Plus, X, Search } from 'lucide-svelte';
	import type { EntityData } from '$lib/api/types';

	interface EntitySearchResult {
		id: string;
		title: string;
		status: string;
		prefix: string;
	}

	let title = $state('');
	let mateType = $state('clearance');
	let description = $state('');
	let notes = $state('');
	let tags = $state<string[]>([]);
	let newTag = $state('');

	// Component and feature selection
	let featuresA = $state<EntityData[]>([]);
	let featuresB = $state<EntityData[]>([]);
	let componentAId = $state('');
	let componentBId = $state('');
	let featureAId = $state('');
	let featureBId = $state('');
	let loadingFeaturesA = $state(false);
	let loadingFeaturesB = $state(false);
	let featureSearchA = $state('');
	let featureSearchB = $state('');

	let saving = $state(false);
	let error = $state<string | null>(null);

	function handleComponentSelect(entity: EntitySearchResult, target: 'A' | 'B') {
		if (target === 'A') {
			componentAId = entity.id;
			featureAId = '';
			loadFeaturesForComponent(entity.id, 'A');
		} else {
			componentBId = entity.id;
			featureBId = '';
			loadFeaturesForComponent(entity.id, 'B');
		}
	}

	function handleComponentClear(target: 'A' | 'B') {
		if (target === 'A') {
			componentAId = '';
			featuresA = [];
			featureAId = '';
		} else {
			componentBId = '';
			featuresB = [];
			featureBId = '';
		}
	}

	async function loadFeaturesForComponent(componentId: string, target: 'A' | 'B') {
		if (!componentId) return;

		if (target === 'A') {
			loadingFeaturesA = true;
		} else {
			loadingFeaturesB = true;
		}

		try {
			const result = await entities.list('FEAT', { include_data: true } as any);
			const filtered = result.items.filter(f => {
				const data = f.data ?? {};
				return data.component === componentId;
			});

			if (target === 'A') {
				featuresA = filtered;
				featureAId = '';
			} else {
				featuresB = filtered;
				featureBId = '';
			}
		} catch (e) {
			console.error('Failed to load features:', e);
		} finally {
			if (target === 'A') {
				loadingFeaturesA = false;
			} else {
				loadingFeaturesB = false;
			}
		}
	}

	// Filtered features based on search
	const filteredFeaturesA = $derived(
		featureSearchA
			? featuresA.filter(f => {
				const search = featureSearchA.toLowerCase();
				return f.title.toLowerCase().includes(search) ||
					f.id.toLowerCase().includes(search) ||
					((f.data as Record<string, unknown>)?.feature_type as string ?? '').toLowerCase().includes(search) ||
					((f.data as Record<string, unknown>)?.description as string ?? '').toLowerCase().includes(search);
			})
			: featuresA
	);

	const filteredFeaturesB = $derived(
		featureSearchB
			? featuresB.filter(f => {
				const search = featureSearchB.toLowerCase();
				return f.title.toLowerCase().includes(search) ||
					f.id.toLowerCase().includes(search) ||
					((f.data as Record<string, unknown>)?.feature_type as string ?? '').toLowerCase().includes(search) ||
					((f.data as Record<string, unknown>)?.description as string ?? '').toLowerCase().includes(search);
			})
			: featuresB
	);

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

			const newId = await entities.save('MATE', data);
			await refreshProject();
			goto(`/mates/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}

	// Features are loaded when components are selected via handleComponentSelect
</script>

<div class="space-y-6">
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/mates')}>
			<ArrowLeft class="mr-2 h-4 w-4" />Back to Mates
		</Button>
		<div>
			<h1 class="text-2xl font-bold">New Mate</h1>
			<p class="text-muted-foreground">Define an assembly mate between two features</p>
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
						<div class="space-y-4">
							<EntityPicker
								label="Component *"
								entityTypes={['CMP']}
								placeholder="Search components..."
								onSelect={(entity) => handleComponentSelect(entity, 'A')}
								onClear={() => handleComponentClear('A')}
							/>
							{#if componentAId}
								<div class="space-y-2">
									<Label for="feature-a">Feature *</Label>
									{#if loadingFeaturesA}
										<p class="text-sm text-muted-foreground">Loading features...</p>
									{:else if featuresA.length === 0}
										<p class="text-xs text-muted-foreground">No features found for this component</p>
									{:else}
										<div class="relative">
											<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
											<Input
												bind:value={featureSearchA}
												placeholder="Search features by title, type..."
												class="pl-9"
											/>
										</div>
										<div class="border rounded-md max-h-48 overflow-y-auto">
											{#each filteredFeaturesA as feat}
												{@const data = feat.data as Record<string, unknown> | null}
												{@const dims = (data?.dimensions as Array<Record<string, unknown>> | undefined)?.[0]}
												<button
													type="button"
													class="w-full px-3 py-2 flex items-center justify-between text-left hover:bg-accent border-b last:border-b-0 {featureAId === feat.id ? 'bg-accent' : ''}"
													onclick={() => { featureAId = feat.id; }}
												>
													<div class="min-w-0 flex-1">
														<div class="font-medium text-sm">{feat.title}</div>
														<div class="text-xs text-muted-foreground">
															{(data?.feature_type as string) ?? 'external'}
															{#if dims}
																| {dims.nominal}{dims.units ? ` ${dims.units}` : ''}
															{/if}
														</div>
													</div>
													{#if featureAId === feat.id}
														<span class="text-primary text-xs font-medium">Selected</span>
													{/if}
												</button>
											{/each}
										</div>
									{/if}
								</div>
							{/if}
						</div>
					</CardContent>
				</Card>

				<Card>
					<CardHeader><CardTitle>Feature B (Shaft/Pin)</CardTitle></CardHeader>
					<CardContent class="space-y-4">
						<p class="text-sm text-muted-foreground">Select the external feature (typically a shaft or pin)</p>
						<div class="space-y-4">
							<EntityPicker
								label="Component *"
								entityTypes={['CMP']}
								placeholder="Search components..."
								onSelect={(entity) => handleComponentSelect(entity, 'B')}
								onClear={() => handleComponentClear('B')}
							/>
							{#if componentBId}
								<div class="space-y-2">
									<Label for="feature-b">Feature *</Label>
									{#if loadingFeaturesB}
										<p class="text-sm text-muted-foreground">Loading features...</p>
									{:else if featuresB.length === 0}
										<p class="text-xs text-muted-foreground">No features found for this component</p>
									{:else}
										<div class="relative">
											<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
											<Input
												bind:value={featureSearchB}
												placeholder="Search features by title, type..."
												class="pl-9"
											/>
										</div>
										<div class="border rounded-md max-h-48 overflow-y-auto">
											{#each filteredFeaturesB as feat}
												{@const data = feat.data as Record<string, unknown> | null}
												{@const dims = (data?.dimensions as Array<Record<string, unknown>> | undefined)?.[0]}
												<button
													type="button"
													class="w-full px-3 py-2 flex items-center justify-between text-left hover:bg-accent border-b last:border-b-0 {featureBId === feat.id ? 'bg-accent' : ''}"
													onclick={() => { featureBId = feat.id; }}
												>
													<div class="min-w-0 flex-1">
														<div class="font-medium text-sm">{feat.title}</div>
														<div class="text-xs text-muted-foreground">
															{(data?.feature_type as string) ?? 'external'}
															{#if dims}
																| {dims.nominal}{dims.units ? ` ${dims.units}` : ''}
															{/if}
														</div>
													</div>
													{#if featureBId === feat.id}
														<span class="text-primary text-xs font-medium">Selected</span>
													{/if}
												</button>
											{/each}
										</div>
									{/if}
								</div>
							{/if}
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
							<Button type="submit" disabled={saving}>{saving ? 'Creating...' : 'Create Mate'}</Button>
							<Button type="button" variant="outline" onclick={() => goto('/mates')}>Cancel</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
