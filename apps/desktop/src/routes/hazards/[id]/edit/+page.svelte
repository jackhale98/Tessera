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
	let category = $state('mechanical');
	let description = $state('');
	let severity = $state('serious');
	let energyLevel = $state('');
	let exposureScenario = $state('');
	let potentialHarms = $state<string[]>([]);
	let affectedPopulations = $state<string[]>([]);
	let tags = $state<string[]>([]);
	let newHarm = $state('');
	let newPopulation = $state('');
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
				category = (data.category as string) ?? 'mechanical';
				description = (data.description as string) ?? '';
				severity = (data.severity as string) ?? 'serious';
				energyLevel = (data.energy_level as string) ?? '';
				exposureScenario = (data.exposure_scenario as string) ?? '';
				potentialHarms = [...((data.potential_harms as string[]) ?? [])];
				affectedPopulations = [...((data.affected_populations as string[]) ?? [])];
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function addHarm() {
		if (newHarm.trim()) {
			potentialHarms = [...potentialHarms, newHarm.trim()];
			newHarm = '';
		}
	}

	function addPopulation() {
		if (newPopulation.trim() && !affectedPopulations.includes(newPopulation.trim())) {
			affectedPopulations = [...affectedPopulations, newPopulation.trim()];
			newPopulation = '';
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
		if (!description.trim()) { error = 'Description is required'; return; }

		saving = true;
		error = null;

		try {
			// Start with existing entity data to preserve all fields (linked risks, controls, etc.)
			const existingData = entity?.data ?? {};

			// Build updated data, preserving existing fields
			const data: Record<string, unknown> = {
				...existingData,
				id,
				title: title.trim(),
				category,
				description: description.trim(),
				severity,
				author: entity?.author ?? $projectAuthor,
				potential_harms: potentialHarms,
				affected_populations: affectedPopulations,
				tags,
				status: entity?.status ?? 'draft',
				created: entity?.created ?? new Date().toISOString(),
				entity_revision: ((existingData.entity_revision as number) ?? 0) + 1
			};

			if (energyLevel.trim()) {
				data.energy_level = energyLevel.trim();
			} else {
				delete data.energy_level;
			}
			if (exposureScenario.trim()) {
				data.exposure_scenario = exposureScenario.trim();
			} else {
				delete data.exposure_scenario;
			}

			await entities.save('HAZ', data);
			await refreshProject();
			goto(`/hazards/${id}`);
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
			<Button variant="ghost" size="sm" onclick={() => goto(`/hazards/${id}`)}>
				<ArrowLeft class="mr-2 h-4 w-4" />Back to Hazard
			</Button>
			<div>
				<h1 class="text-2xl font-bold">Edit Hazard</h1>
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
						<CardHeader><CardTitle>Hazard Information</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="space-y-2">
								<Label for="title">Title *</Label>
								<Input id="title" bind:value={title} placeholder="Hazard name" required />
							</div>
							<div class="grid gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="category">Category *</Label>
									<Select id="category" bind:value={category}>
										<option value="electrical">Electrical</option>
										<option value="mechanical">Mechanical</option>
										<option value="thermal">Thermal</option>
										<option value="chemical">Chemical</option>
										<option value="biological">Biological</option>
										<option value="radiation">Radiation</option>
										<option value="ergonomic">Ergonomic</option>
										<option value="software">Software</option>
										<option value="environmental">Environmental</option>
									</Select>
								</div>
								<div class="space-y-2">
									<Label for="severity">Severity *</Label>
									<Select id="severity" bind:value={severity}>
										<option value="negligible">Negligible</option>
										<option value="minor">Minor</option>
										<option value="serious">Serious</option>
										<option value="severe">Severe</option>
										<option value="catastrophic">Catastrophic</option>
									</Select>
								</div>
							</div>
							<div class="space-y-2">
								<Label for="description">Description *</Label>
								<Textarea id="description" bind:value={description} placeholder="Describe the hazard" rows={3} />
							</div>
							<div class="space-y-2">
								<Label for="energy-level">Energy Level</Label>
								<Input id="energy-level" bind:value={energyLevel} placeholder="e.g., 300V DC, 500°C" />
							</div>
							<div class="space-y-2">
								<Label for="exposure">Exposure Scenario</Label>
								<Textarea id="exposure" bind:value={exposureScenario} placeholder="How could exposure occur?" rows={2} />
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Potential Harms</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="flex gap-2">
								<Input bind:value={newHarm} placeholder="Add potential harm" onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addHarm())} />
								<Button type="button" variant="outline" onclick={addHarm}><Plus class="h-4 w-4" /></Button>
							</div>
							{#if potentialHarms.length > 0}
								<ul class="space-y-2">
									{#each potentialHarms as harm, i}
										<li class="flex items-center justify-between rounded-lg border p-3">
											<span class="text-sm">{harm}</span>
											<Button type="button" variant="ghost" size="sm" onclick={() => potentialHarms = potentialHarms.filter((_, idx) => idx !== i)}><X class="h-4 w-4" /></Button>
										</li>
									{/each}
								</ul>
							{/if}
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Affected Populations</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="flex gap-2">
								<Input bind:value={newPopulation} placeholder="e.g., Operators, Patients" onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addPopulation())} />
								<Button type="button" variant="outline" onclick={addPopulation}><Plus class="h-4 w-4" /></Button>
							</div>
							{#if affectedPopulations.length > 0}
								<div class="flex flex-wrap gap-2">
									{#each affectedPopulations as pop}
										<span class="inline-flex items-center gap-1 rounded-full border px-3 py-1 text-sm">
											{pop}
											<button type="button" class="text-muted-foreground hover:text-foreground" onclick={() => affectedPopulations = affectedPopulations.filter(p => p !== pop)}><X class="h-3 w-3" /></button>
										</span>
									{/each}
								</div>
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
								<Button type="submit" disabled={saving}>{saving ? 'Saving...' : 'Save Changes'}</Button>
								<Button type="button" variant="outline" onclick={() => goto(`/hazards/${id}`)}>Cancel</Button>
							</div>
						</CardContent>
					</Card>
				</div>
			</div>
		</form>
	{:else}
		<Card><CardContent class="flex h-64 items-center justify-center"><p class="text-muted-foreground">Hazard not found</p></CardContent></Card>
	{/if}
</div>
