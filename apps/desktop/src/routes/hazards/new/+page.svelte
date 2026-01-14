<script lang="ts">
	import { goto } from '$app/navigation';
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
	import { refreshProject, projectAuthor } from '$lib/stores/project';
	import { ArrowLeft, Plus, X } from 'lucide-svelte';

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

	let saving = $state(false);
	let error = $state<string | null>(null);

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
			const data: Record<string, unknown> = {
				title: title.trim(),
				category,
				description: description.trim(),
				severity,
				author: $projectAuthor,
				potential_harms: potentialHarms,
				affected_populations: affectedPopulations,
				tags
			};

			if (energyLevel.trim()) data.energy_level = energyLevel.trim();
			if (exposureScenario.trim()) data.exposure_scenario = exposureScenario.trim();

			const newId = await entities.save('HAZ', data);
			await refreshProject();
			goto(`/hazards/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="space-y-6">
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/hazards')}>
			<ArrowLeft class="mr-2 h-4 w-4" />Back to Hazards
		</Button>
		<div>
			<h1 class="text-2xl font-bold">New Hazard</h1>
			<p class="text-muted-foreground">Identify a new safety hazard</p>
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
							<Button type="submit" disabled={saving}>{saving ? 'Creating...' : 'Create Hazard'}</Button>
							<Button type="button" variant="outline" onclick={() => goto('/hazards')}>Cancel</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
