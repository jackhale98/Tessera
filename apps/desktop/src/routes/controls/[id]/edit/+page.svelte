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
	let controlType = $state('inspection');
	let controlCategory = $state('variable');
	let description = $state('');
	let charName = $state('');
	let charNominal = $state<number | null>(null);
	let charUpperLimit = $state<number | null>(null);
	let charLowerLimit = $state<number | null>(null);
	let charUnits = $state('mm');
	let charCritical = $state(false);
	let measurementMethod = $state('');
	let measurementEquipment = $state('');
	let samplingType = $state('periodic');
	let samplingFrequency = $state('');
	let sampleSize = $state<number | null>(null);
	let reactionPlan = $state('');
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
				controlType = (data.control_type as string) ?? 'inspection';
				controlCategory = (data.control_category as string) ?? 'variable';
				description = (data.description as string) ?? '';
				reactionPlan = (data.reaction_plan as string) ?? '';

				const characteristic = data.characteristic as {
					name?: string;
					nominal?: number;
					upper_limit?: number;
					lower_limit?: number;
					units?: string;
					critical?: boolean;
				} | null;

				if (characteristic) {
					charName = characteristic.name ?? '';
					charNominal = characteristic.nominal ?? null;
					charUpperLimit = characteristic.upper_limit ?? null;
					charLowerLimit = characteristic.lower_limit ?? null;
					charUnits = characteristic.units ?? 'mm';
					charCritical = characteristic.critical ?? false;
				}

				const measurement = data.measurement as {
					method?: string;
					equipment?: string;
				} | null;

				if (measurement) {
					measurementMethod = measurement.method ?? '';
					measurementEquipment = measurement.equipment ?? '';
				}

				const sampling = data.sampling as {
					sampling_type?: string;
					frequency?: string;
					sample_size?: number;
				} | null;

				if (sampling) {
					samplingType = sampling.sampling_type ?? 'periodic';
					samplingFrequency = sampling.frequency ?? '';
					sampleSize = sampling.sample_size ?? null;
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
		if (!charName.trim()) { error = 'Characteristic name is required'; return; }

		saving = true;
		error = null;

		try {
			// Start with existing entity data to preserve all fields (process links, limits data, etc.)
			const existingData = entity?.data ?? {};

			// Build updated data, preserving existing fields
			const data: Record<string, unknown> = {
				...existingData,
				id,
				title: title.trim(),
				control_type: controlType,
				control_category: controlCategory,
				characteristic: {
					name: charName.trim(),
					nominal: charNominal,
					upper_limit: charUpperLimit,
					lower_limit: charLowerLimit,
					units: charUnits,
					critical: charCritical
				},
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
			if (reactionPlan.trim()) {
				data.reaction_plan = reactionPlan.trim();
			} else {
				delete data.reaction_plan;
			}

			if (measurementMethod.trim() || measurementEquipment.trim()) {
				data.measurement = {
					method: measurementMethod.trim() || undefined,
					equipment: measurementEquipment.trim() || undefined
				};
			} else {
				delete data.measurement;
			}

			data.sampling = {
				sampling_type: samplingType,
				frequency: samplingFrequency.trim() || undefined,
				sample_size: sampleSize
			};

			await entities.save('CTRL', data);
			await refreshProject();
			goto(`/controls/${id}`);
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
			<Button variant="ghost" size="sm" onclick={() => goto(`/controls/${id}`)}>
				<ArrowLeft class="mr-2 h-4 w-4" />Back to Control
			</Button>
			<div>
				<h1 class="text-2xl font-bold">Edit Control</h1>
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
						<CardHeader><CardTitle>Control Information</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="space-y-2">
								<Label for="title">Title *</Label>
								<Input id="title" bind:value={title} placeholder="Control name" required />
							</div>
							<div class="grid gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="control-type">Control Type *</Label>
									<Select id="control-type" bind:value={controlType}>
										<option value="spc">SPC</option>
										<option value="inspection">Inspection</option>
										<option value="poka_yoke">Poka-Yoke</option>
										<option value="visual">Visual</option>
										<option value="functional_test">Functional Test</option>
										<option value="attribute">Attribute</option>
									</Select>
								</div>
								<div class="space-y-2">
									<Label for="control-category">Category *</Label>
									<Select id="control-category" bind:value={controlCategory}>
										<option value="variable">Variable</option>
										<option value="attribute">Attribute</option>
									</Select>
								</div>
							</div>
							<div class="space-y-2">
								<Label for="description">Description</Label>
								<Textarea id="description" bind:value={description} placeholder="Control description" rows={2} />
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Characteristic</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="space-y-2">
								<Label for="char-name">Characteristic Name *</Label>
								<Input id="char-name" bind:value={charName} placeholder="e.g., Diameter, Length" required />
							</div>
							<div class="grid gap-4 sm:grid-cols-4">
								<div class="space-y-2">
									<Label for="char-nominal">Nominal</Label>
									<Input id="char-nominal" type="number" step="0.001" bind:value={charNominal} placeholder="10.000" />
								</div>
								<div class="space-y-2">
									<Label for="char-lsl">LSL</Label>
									<Input id="char-lsl" type="number" step="0.001" bind:value={charLowerLimit} placeholder="9.990" />
								</div>
								<div class="space-y-2">
									<Label for="char-usl">USL</Label>
									<Input id="char-usl" type="number" step="0.001" bind:value={charUpperLimit} placeholder="10.010" />
								</div>
								<div class="space-y-2">
									<Label for="char-units">Units</Label>
									<Input id="char-units" bind:value={charUnits} placeholder="mm" />
								</div>
							</div>
							<label class="flex items-center gap-2">
								<input type="checkbox" bind:checked={charCritical} class="h-4 w-4 rounded border" />
								<span class="text-sm">Critical characteristic (CTQ)</span>
							</label>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Measurement & Sampling</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="grid gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="method">Measurement Method</Label>
									<Input id="method" bind:value={measurementMethod} placeholder="e.g., Caliper measurement" />
								</div>
								<div class="space-y-2">
									<Label for="equipment">Equipment</Label>
									<Input id="equipment" bind:value={measurementEquipment} placeholder="e.g., Digital caliper" />
								</div>
							</div>
							<div class="grid gap-4 sm:grid-cols-3">
								<div class="space-y-2">
									<Label for="sampling-type">Sampling Type</Label>
									<Select id="sampling-type" bind:value={samplingType}>
										<option value="continuous">Continuous</option>
										<option value="periodic">Periodic</option>
										<option value="lot">Lot</option>
										<option value="first_article">First Article</option>
									</Select>
								</div>
								<div class="space-y-2">
									<Label for="frequency">Frequency</Label>
									<Input id="frequency" bind:value={samplingFrequency} placeholder="e.g., Every 2 hours" />
								</div>
								<div class="space-y-2">
									<Label for="sample-size">Sample Size</Label>
									<Input id="sample-size" type="number" min={1} bind:value={sampleSize} placeholder="5" />
								</div>
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Reaction Plan</CardTitle></CardHeader>
						<CardContent>
							<Textarea bind:value={reactionPlan} placeholder="Actions to take when out of control..." rows={3} />
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
								<Button type="button" variant="outline" onclick={() => goto(`/controls/${id}`)}>Cancel</Button>
							</div>
						</CardContent>
					</Card>
				</div>
			</div>
		</form>
	{:else}
		<Card><CardContent class="flex h-64 items-center justify-center"><p class="text-muted-foreground">Control not found</p></CardContent></Card>
	{/if}
</div>
