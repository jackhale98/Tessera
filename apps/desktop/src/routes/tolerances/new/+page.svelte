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
	import { entities } from '$lib/api';
	import { refreshProject, projectAuthor, isProjectOpen } from '$lib/stores/project';
	import { ArrowLeft, Plus, X, Trash2, ArrowUp, ArrowDown } from 'lucide-svelte';
	import type { EntityData } from '$lib/api/types';

	interface Contributor {
		name: string;
		featureId: string;
		componentId: string;
		direction: 'positive' | 'negative';
		nominal: number;
		plusTol: number;
		minusTol: number;
		distribution: string;
		source: string;
	}

	let title = $state('');
	let description = $state('');
	let targetName = $state('');
	let targetNominal = $state<number | null>(null);
	let targetUpperLimit = $state<number | null>(null);
	let targetLowerLimit = $state<number | null>(null);
	let targetUnits = $state('mm');
	let targetCritical = $state(false);
	let sigmaLevel = $state(6);
	let meanShiftK = $state(0);
	let includeGdt = $state(false);
	let tags = $state<string[]>([]);
	let newTag = $state('');

	// Contributors
	let contributors = $state<Contributor[]>([]);

	// Component and feature data for selection
	let components = $state<EntityData[]>([]);
	let allFeatures = $state<EntityData[]>([]);
	let loadingComponents = $state(true);

	let saving = $state(false);
	let error = $state<string | null>(null);

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

	function getFeaturesForComponent(componentId: string): EntityData[] {
		if (!componentId) return [];
		return allFeatures.filter(f => {
			const data = f.data ?? {};
			return data.component === componentId;
		});
	}

	function getFeatureTitle(featureId: string): string {
		const feature = allFeatures.find(f => f.id === featureId);
		return feature?.title ?? featureId;
	}

	function getComponentTitle(componentId: string): string {
		const component = components.find(c => c.id === componentId);
		return component?.title ?? componentId;
	}

	function addContributor() {
		contributors = [...contributors, {
			name: '',
			featureId: '',
			componentId: '',
			direction: 'positive',
			nominal: 0,
			plusTol: 0,
			minusTol: 0,
			distribution: 'normal',
			source: ''
		}];
	}

	function removeContributor(index: number) {
		contributors = contributors.filter((_, i) => i !== index);
	}

	function handleContributorComponentChange(index: number, componentId: string) {
		contributors[index].componentId = componentId;
		contributors[index].featureId = '';
	}

	function addTag() {
		if (newTag.trim() && !tags.includes(newTag.trim())) {
			tags = [...tags, newTag.trim()];
			newTag = '';
		}
	}

	onMount(() => {
		loadComponentsAndFeatures();
	});

	$effect(() => {
		if ($isProjectOpen) loadComponentsAndFeatures();
	});

	async function handleSubmit() {
		if (!title.trim()) { error = 'Title is required'; return; }
		if (!targetName.trim()) { error = 'Target name is required'; return; }
		if (targetNominal === null) { error = 'Target nominal is required'; return; }
		if (targetUpperLimit === null) { error = 'Target USL is required'; return; }
		if (targetLowerLimit === null) { error = 'Target LSL is required'; return; }

		saving = true;
		error = null;

		try {
			// Build contributors array with feature references
			const validContributors = contributors
				.filter(c => c.name.trim() && c.featureId)
				.map(c => {
					const feature = allFeatures.find(f => f.id === c.featureId);
					const component = components.find(comp => comp.id === c.componentId);
					return {
						name: c.name.trim(),
						feature: {
							id: c.featureId,
							name: feature?.title ?? '',
							component_id: c.componentId,
							component_name: component?.title ?? ''
						},
						direction: c.direction,
						nominal: c.nominal,
						plus_tol: c.plusTol,
						minus_tol: c.minusTol,
						distribution: c.distribution,
						...(c.source.trim() && { source: c.source.trim() })
					};
				});

			const data: Record<string, unknown> = {
				title: title.trim(),
				target: {
					name: targetName.trim(),
					nominal: targetNominal,
					upper_limit: targetUpperLimit,
					lower_limit: targetLowerLimit,
					units: targetUnits,
					critical: targetCritical
				},
				contributors: validContributors,
				sigma_level: sigmaLevel,
				mean_shift_k: meanShiftK,
				include_gdt: includeGdt,
				disposition: 'under_review',
				author: $projectAuthor,
				tags
			};

			if (description.trim()) data.description = description.trim();

			const newId = await entities.save('TOL', data);
			await refreshProject();
			goto(`/tolerances/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="space-y-6">
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/tolerances')}>
			<ArrowLeft class="mr-2 h-4 w-4" />Back to Stackups
		</Button>
		<div>
			<h1 class="text-2xl font-bold">New Tolerance Stackup</h1>
			<p class="text-muted-foreground">Create a new stackup analysis with feature contributors</p>
		</div>
	</div>

	{#if error}
		<Card class="border-destructive"><CardContent class="pt-6"><p class="text-destructive">{error}</p></CardContent></Card>
	{/if}

	<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
		<div class="grid gap-6 lg:grid-cols-3">
			<div class="space-y-6 lg:col-span-2">
				<Card>
					<CardHeader><CardTitle>Stackup Information</CardTitle></CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<Label for="title">Title *</Label>
							<Input id="title" bind:value={title} placeholder="Stackup name (e.g., Assembly Gap Analysis)" required />
						</div>
						<div class="space-y-2">
							<Label for="description">Description</Label>
							<Textarea id="description" bind:value={description} placeholder="Describe what this stackup analyzes" rows={2} />
						</div>
					</CardContent>
				</Card>

				<Card>
					<CardHeader><CardTitle>Target Specification</CardTitle></CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<Label for="target-name">Target Dimension Name *</Label>
							<Input id="target-name" bind:value={targetName} placeholder="e.g., Assembly Gap" required />
						</div>
						<div class="grid gap-4 sm:grid-cols-4">
							<div class="space-y-2">
								<Label for="target-nominal">Nominal *</Label>
								<Input id="target-nominal" type="number" step="0.001" bind:value={targetNominal} placeholder="1.000" required />
							</div>
							<div class="space-y-2">
								<Label for="target-lsl">LSL *</Label>
								<Input id="target-lsl" type="number" step="0.001" bind:value={targetLowerLimit} placeholder="0.500" required />
							</div>
							<div class="space-y-2">
								<Label for="target-usl">USL *</Label>
								<Input id="target-usl" type="number" step="0.001" bind:value={targetUpperLimit} placeholder="1.500" required />
							</div>
							<div class="space-y-2">
								<Label for="target-units">Units</Label>
								<Input id="target-units" bind:value={targetUnits} placeholder="mm" />
							</div>
						</div>
						<label class="flex items-center gap-2">
							<input type="checkbox" bind:checked={targetCritical} class="h-4 w-4 rounded border" />
							<span class="text-sm">Critical dimension</span>
						</label>
					</CardContent>
				</Card>

				<Card>
					<CardHeader><CardTitle>Analysis Settings</CardTitle></CardHeader>
					<CardContent class="space-y-4">
						<div class="grid gap-4 sm:grid-cols-3">
							<div class="space-y-2">
								<Label for="sigma-level">Sigma Level</Label>
								<Select id="sigma-level" bind:value={sigmaLevel}>
									<option value={3}>3σ (99.73%)</option>
									<option value={4}>4σ (99.994%)</option>
									<option value={5}>5σ (99.99994%)</option>
									<option value={6}>6σ (99.9999998%)</option>
								</Select>
							</div>
							<div class="space-y-2">
								<Label for="mean-shift">Mean Shift (k)</Label>
								<Input id="mean-shift" type="number" step="0.1" min="0" max="2" bind:value={meanShiftK} placeholder="0" />
								<p class="text-xs text-muted-foreground">Bender k factor (0-1.5 typical)</p>
							</div>
							<div class="flex items-end pb-2">
								<label class="flex items-center gap-2">
									<input type="checkbox" bind:checked={includeGdt} class="h-4 w-4 rounded border" />
									<span class="text-sm">Include GD&T</span>
								</label>
							</div>
						</div>
					</CardContent>
				</Card>

				<!-- Contributors -->
				<Card>
					<CardHeader class="flex flex-row items-center justify-between">
						<CardTitle>Contributors ({contributors.length})</CardTitle>
						<Button type="button" variant="outline" size="sm" onclick={addContributor} disabled={loadingComponents}>
							<Plus class="mr-2 h-4 w-4" /> Add Contributor
						</Button>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if contributors.length === 0}
							<p class="text-sm text-muted-foreground py-4 text-center">No contributors added yet. Add features that contribute to this stackup.</p>
						{:else}
							{#each contributors as contrib, i}
								<div class="rounded-lg border p-4 space-y-4">
									<div class="flex items-start justify-between">
										<div class="flex items-center gap-3">
											<span class="flex h-6 w-6 items-center justify-center rounded-full bg-muted text-sm font-medium">#{i + 1}</span>
											{#if contrib.direction === 'positive'}
												<ArrowUp class="h-4 w-4 text-green-500" />
											{:else}
												<ArrowDown class="h-4 w-4 text-red-500" />
											{/if}
										</div>
										<Button type="button" variant="ghost" size="sm" onclick={() => removeContributor(i)}>
											<Trash2 class="h-4 w-4" />
										</Button>
									</div>

									<div class="grid gap-4 sm:grid-cols-2">
										<div class="space-y-2">
											<Label for={`contrib-name-${i}`}>Name *</Label>
											<Input id={`contrib-name-${i}`} bind:value={contrib.name} placeholder="Contributor name" />
										</div>
										<div class="space-y-2">
											<Label for={`contrib-direction-${i}`}>Direction</Label>
											<Select id={`contrib-direction-${i}`} bind:value={contrib.direction}>
												<option value="positive">Positive (+)</option>
												<option value="negative">Negative (-)</option>
											</Select>
										</div>
									</div>

									<div class="grid gap-4 sm:grid-cols-2">
										<div class="space-y-2">
											<Label for={`contrib-component-${i}`}>Component</Label>
											<Select
												id={`contrib-component-${i}`}
												value={contrib.componentId}
												onchange={(e) => handleContributorComponentChange(i, (e.target as HTMLSelectElement).value)}
												disabled={loadingComponents}
											>
												<option value="">Select a component...</option>
												{#each components as comp}
													<option value={comp.id}>{comp.title}</option>
												{/each}
											</Select>
										</div>
										<div class="space-y-2">
											<Label for={`contrib-feature-${i}`}>Feature</Label>
											<Select id={`contrib-feature-${i}`} bind:value={contrib.featureId} disabled={!contrib.componentId}>
												<option value="">{contrib.componentId ? 'Select a feature...' : 'Select component first'}</option>
												{#each getFeaturesForComponent(contrib.componentId) as feat}
													<option value={feat.id}>{feat.title}</option>
												{/each}
											</Select>
											{#if contrib.componentId && getFeaturesForComponent(contrib.componentId).length === 0}
												<p class="text-xs text-muted-foreground">No features found for this component</p>
											{/if}
										</div>
									</div>

									<div class="grid gap-4 sm:grid-cols-4">
										<div class="space-y-2">
											<Label for={`contrib-nominal-${i}`}>Nominal</Label>
											<Input id={`contrib-nominal-${i}`} type="number" step="0.001" bind:value={contrib.nominal} placeholder="0.000" />
										</div>
										<div class="space-y-2">
											<Label for={`contrib-plus-${i}`}>+Tol</Label>
											<Input id={`contrib-plus-${i}`} type="number" step="0.001" bind:value={contrib.plusTol} placeholder="0.000" />
										</div>
										<div class="space-y-2">
											<Label for={`contrib-minus-${i}`}>-Tol</Label>
											<Input id={`contrib-minus-${i}`} type="number" step="0.001" bind:value={contrib.minusTol} placeholder="0.000" />
										</div>
										<div class="space-y-2">
											<Label for={`contrib-dist-${i}`}>Distribution</Label>
											<Select id={`contrib-dist-${i}`} bind:value={contrib.distribution}>
												<option value="normal">Normal</option>
												<option value="uniform">Uniform</option>
												<option value="triangular">Triangular</option>
											</Select>
										</div>
									</div>

									<div class="space-y-2">
										<Label for={`contrib-source-${i}`}>Source (optional)</Label>
										<Input id={`contrib-source-${i}`} bind:value={contrib.source} placeholder="e.g., Drawing DWG-001 Rev A" />
									</div>
								</div>
							{/each}
						{/if}
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
							<Button type="submit" disabled={saving}>{saving ? 'Creating...' : 'Create Stackup'}</Button>
							<Button type="button" variant="outline" onclick={() => goto('/tolerances')}>Cancel</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
