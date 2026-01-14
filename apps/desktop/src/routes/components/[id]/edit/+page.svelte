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
	let partNumber = $state('');
	let revision = $state('');
	let description = $state('');
	let category = $state('mechanical');
	let makeBuy = $state('make');
	let material = $state('');
	let massKg = $state<number | null>(null);
	let unitCost = $state<number | null>(null);
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
				partNumber = (data.part_number as string) ?? '';
				revision = (data.revision as string) ?? '';
				description = (data.description as string) ?? '';
				category = (data.category as string) ?? 'mechanical';
				makeBuy = (data.make_buy as string) ?? 'make';
				material = (data.material as string) ?? '';
				massKg = (data.mass_kg as number) ?? null;
				unitCost = (data.unit_cost as number) ?? null;
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

	function removeTag(tag: string) {
		tags = tags.filter((t) => t !== tag);
	}

	async function handleSubmit() {
		if (!title.trim()) { error = 'Title is required'; return; }
		if (!partNumber.trim()) { error = 'Part number is required'; return; }

		saving = true;
		error = null;

		try {
			const data: Record<string, unknown> = {
				title: title.trim(),
				part_number: partNumber.trim(),
				category,
				make_buy: makeBuy,
				author: $projectAuthor,
				tags
			};

			if (revision.trim()) data.revision = revision.trim();
			if (description.trim()) data.description = description.trim();
			if (material.trim()) data.material = material.trim();
			if (massKg !== null) data.mass_kg = massKg;
			if (unitCost !== null) data.unit_cost = unitCost;

			await entities.save('CMP', { ...data, id });
			await refreshProject();
			goto(`/components/${id}`);
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
			<Button variant="ghost" size="sm" onclick={() => goto(`/components/${id}`)}>
				<ArrowLeft class="mr-2 h-4 w-4" />Back to Component
			</Button>
			<div>
				<h1 class="text-2xl font-bold">Edit Component</h1>
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
						<CardHeader><CardTitle>Basic Information</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="space-y-2">
								<Label for="title">Title *</Label>
								<Input id="title" bind:value={title} placeholder="Component name/description" required />
							</div>
							<div class="grid gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="part-number">Part Number *</Label>
									<Input id="part-number" bind:value={partNumber} placeholder="e.g., 100-0001" required />
								</div>
								<div class="space-y-2">
									<Label for="revision">Revision</Label>
									<Input id="revision" bind:value={revision} placeholder="e.g., A, 01" />
								</div>
							</div>
							<div class="space-y-2">
								<Label for="description">Description</Label>
								<Textarea id="description" bind:value={description} placeholder="Detailed component description" rows={3} />
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Specifications</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="grid gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="category">Category *</Label>
									<Select id="category" bind:value={category}>
										<option value="mechanical">Mechanical</option>
										<option value="electrical">Electrical</option>
										<option value="software">Software</option>
										<option value="fastener">Fastener</option>
										<option value="consumable">Consumable</option>
									</Select>
								</div>
								<div class="space-y-2">
									<Label for="make-buy">Make/Buy *</Label>
									<Select id="make-buy" bind:value={makeBuy}>
										<option value="make">Make (In-house)</option>
										<option value="buy">Buy (Purchased)</option>
									</Select>
								</div>
							</div>
							<div class="space-y-2">
								<Label for="material">Material</Label>
								<Input id="material" bind:value={material} placeholder="e.g., 6061-T6 Aluminum, ABS Plastic" />
							</div>
							<div class="grid gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="mass">Mass (kg)</Label>
									<Input id="mass" type="number" step="0.001" min={0} bind:value={massKg} placeholder="0.000" />
								</div>
								<div class="space-y-2">
									<Label for="cost">Unit Cost ($)</Label>
									<Input id="cost" type="number" step="0.01" min={0} bind:value={unitCost} placeholder="0.00" />
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
										<span class="inline-flex items-center gap-1 rounded-full border px-3 py-1 text-sm">{tag}<button type="button" class="text-muted-foreground hover:text-foreground" onclick={() => removeTag(tag)}><X class="h-3 w-3" /></button></span>
									{/each}
								</div>
							{:else}
								<p class="text-sm text-muted-foreground">No tags</p>
							{/if}
						</CardContent>
					</Card>

					<Card>
						<CardContent class="pt-6">
							<div class="flex flex-col gap-2">
								<Button type="submit" disabled={saving}>{saving ? 'Saving...' : 'Save Changes'}</Button>
								<Button type="button" variant="outline" onclick={() => goto(`/components/${id}`)}>Cancel</Button>
							</div>
						</CardContent>
					</Card>
				</div>
			</div>
		</form>
	{:else}
		<Card><CardContent class="flex h-64 items-center justify-center"><p class="text-muted-foreground">Component not found</p></CardContent></Card>
	{/if}
</div>
