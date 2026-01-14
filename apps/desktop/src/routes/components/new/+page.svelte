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

	let saving = $state(false);
	let error = $state<string | null>(null);

	function addTag() {
		if (newTag.trim() && !tags.includes(newTag.trim())) {
			tags = [...tags, newTag.trim()];
			newTag = '';
		}
	}

	function removeTag(tag: string) {
		tags = tags.filter((t) => t !== tag);
	}

	function handleTagKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			addTag();
		}
	}

	async function handleSubmit() {
		if (!title.trim()) {
			error = 'Title is required';
			return;
		}
		if (!partNumber.trim()) {
			error = 'Part number is required';
			return;
		}

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

			const newId = await entities.save('CMP', data);
			await refreshProject();
			goto(`/components/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to create component:', e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/components')}>
			<ArrowLeft class="mr-2 h-4 w-4" />
			Back to Components
		</Button>

		<div>
			<h1 class="text-2xl font-bold">New Component</h1>
			<p class="text-muted-foreground">Add a new component to the BOM</p>
		</div>
	</div>

	<!-- Error display -->
	{#if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{/if}

	<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main form -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Basic Info -->
				<Card>
					<CardHeader>
						<CardTitle>Basic Information</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<Label for="title">Title *</Label>
							<Input
								id="title"
								bind:value={title}
								placeholder="Component name/description"
								required
							/>
						</div>

						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="part-number">Part Number *</Label>
								<Input
									id="part-number"
									bind:value={partNumber}
									placeholder="e.g., 100-0001"
									required
								/>
							</div>

							<div class="space-y-2">
								<Label for="revision">Revision</Label>
								<Input
									id="revision"
									bind:value={revision}
									placeholder="e.g., A, 01"
								/>
							</div>
						</div>

						<div class="space-y-2">
							<Label for="description">Description</Label>
							<Textarea
								id="description"
								bind:value={description}
								placeholder="Detailed component description"
								rows={3}
							/>
						</div>
					</CardContent>
				</Card>

				<!-- Specifications -->
				<Card>
					<CardHeader>
						<CardTitle>Specifications</CardTitle>
					</CardHeader>
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
							<Input
								id="material"
								bind:value={material}
								placeholder="e.g., 6061-T6 Aluminum, ABS Plastic"
							/>
						</div>

						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="mass">Mass (kg)</Label>
								<Input
									id="mass"
									type="number"
									step="0.001"
									min="0"
									bind:value={massKg}
									placeholder="0.000"
								/>
							</div>

							<div class="space-y-2">
								<Label for="cost">Unit Cost ($)</Label>
								<Input
									id="cost"
									type="number"
									step="0.01"
									min="0"
									bind:value={unitCost}
									placeholder="0.00"
								/>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Tags -->
				<Card>
					<CardHeader>
						<CardTitle>Tags</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex gap-2">
							<Input
								bind:value={newTag}
								placeholder="Add tag"
								onkeydown={handleTagKeydown}
							/>
							<Button type="button" variant="outline" onclick={addTag}>
								<Plus class="h-4 w-4" />
							</Button>
						</div>

						{#if tags.length > 0}
							<div class="flex flex-wrap gap-2">
								{#each tags as tag}
									<span class="inline-flex items-center gap-1 rounded-full border px-3 py-1 text-sm">
										{tag}
										<button
											type="button"
											class="text-muted-foreground hover:text-foreground"
											onclick={() => removeTag(tag)}
										>
											<X class="h-3 w-3" />
										</button>
									</span>
								{/each}
							</div>
						{:else}
							<p class="text-sm text-muted-foreground">No tags added</p>
						{/if}
					</CardContent>
				</Card>

				<!-- Actions -->
				<Card>
					<CardContent class="pt-6">
						<div class="flex flex-col gap-2">
							<Button type="submit" disabled={saving}>
								{saving ? 'Creating...' : 'Create Component'}
							</Button>
							<Button type="button" variant="outline" onclick={() => goto('/components')}>
								Cancel
							</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
