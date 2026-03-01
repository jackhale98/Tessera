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
		Badge
	} from '$lib/components/ui';
	import { EntityPicker } from '$lib/components/common';
	import { lots, assemblies, entities } from '$lib/api/tauri';
	import { refreshProject, projectAuthor } from '$lib/stores/project';
	import { ArrowLeft, Plus, X, ListOrdered, Package } from 'lucide-svelte';

	let title = $state('');
	let lotNumber = $state('');
	let quantity = $state<number | null>(null);
	let notes = $state('');
	let tags = $state<string[]>([]);
	let newTag = $state('');
	let productId = $state('');
	let routingSteps = $state<{ id: string; title: string }[]>([]);
	let routingLoading = $state(false);
	let fromRouting = $state(true);

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

	async function loadRouting(assemblyId: string) {
		routingLoading = true;
		routingSteps = [];
		try {
			const processIds = await assemblies.getRouting(assemblyId);
			// Resolve process names
			const steps: { id: string; title: string }[] = [];
			for (const procId of processIds) {
				try {
					const proc = await entities.get(procId);
					steps.push({ id: procId, title: (proc?.title as string) ?? procId });
				} catch {
					steps.push({ id: procId, title: procId });
				}
			}
			routingSteps = steps;
		} catch (e) {
			console.error('Failed to load routing:', e);
			routingSteps = [];
		} finally {
			routingLoading = false;
		}
	}

	function handleProductSelect(entity: { id: string; title: string; status: string; prefix: string }) {
		productId = entity.id;
		loadRouting(entity.id);
	}

	function handleProductClear() {
		productId = '';
		routingSteps = [];
	}

	async function handleSubmit() {
		if (!title.trim()) {
			error = 'Title is required';
			return;
		}

		saving = true;
		error = null;

		try {
			const result = await lots.create({
				title: title.trim(),
				lot_number: lotNumber.trim() || undefined,
				quantity: quantity ?? undefined,
				product: productId || undefined,
				notes: notes.trim() || undefined,
				author: $projectAuthor,
				from_routing: fromRouting && routingSteps.length > 0 ? true : undefined
			});

			const newId = typeof result === 'string' ? result : (result as Record<string, unknown>)?.id as string;

			await refreshProject();
			goto(`/manufacturing/lots/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to create lot:', e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/manufacturing/lots')}>
			<ArrowLeft class="mr-2 h-4 w-4" />
			Back to Lots
		</Button>

		<div>
			<h1 class="text-2xl font-bold">New Lot</h1>
			<p class="text-muted-foreground">Create a new production lot</p>
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
								placeholder="Production lot title"
								required
							/>
						</div>

						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="lot-number">Lot Number</Label>
								<Input
									id="lot-number"
									bind:value={lotNumber}
									placeholder="e.g., LOT-2026-001"
								/>
							</div>

							<div class="space-y-2">
								<Label for="quantity">Quantity</Label>
								<Input
									id="quantity"
									type="number"
									min="1"
									bind:value={quantity}
									placeholder="Number of units"
								/>
							</div>
						</div>

						<div class="space-y-2">
							<Label for="notes">Notes</Label>
							<Textarea
								id="notes"
								bind:value={notes}
								placeholder="Additional notes about this production lot"
								rows={3}
							/>
						</div>
					</CardContent>
				</Card>

				<!-- Product Selection -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Package class="h-5 w-5" />
							Product
						</CardTitle>
					</CardHeader>
					<CardContent>
						<EntityPicker
							entityTypes={['ASM', 'CMP']}
							placeholder="Search assemblies or components..."
							value={productId}
							onSelect={handleProductSelect}
							onClear={handleProductClear}
							label="Product to manufacture"
						/>
						{#if routingSteps.length > 0}
							<div class="mt-3 flex items-center gap-2">
								<input
									type="checkbox"
									id="from_routing"
									bind:checked={fromRouting}
									class="h-4 w-4 rounded border-gray-300"
								/>
								<Label for="from_routing">Populate execution steps from routing</Label>
							</div>
						{/if}
						<p class="mt-2 text-xs text-muted-foreground">
							Select an assembly or component. If it has a routing defined, execution steps will be created automatically.
						</p>
					</CardContent>
				</Card>
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Routing Preview -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<ListOrdered class="h-4 w-4" />
							Routing Preview
						</CardTitle>
					</CardHeader>
					<CardContent>
						{#if routingLoading}
							<div class="flex items-center gap-2 text-sm text-muted-foreground">
								<div class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
								Loading routing...
							</div>
						{:else if routingSteps.length > 0}
							<div class="space-y-2">
								{#each routingSteps as step, i}
									<div class="flex items-center gap-2 rounded-md border p-2">
										<Badge variant="outline" class="shrink-0 font-mono text-xs">
											{i + 1}
										</Badge>
										<span class="text-sm truncate">{step.title}</span>
									</div>
								{/each}
								<p class="text-xs text-muted-foreground mt-2">
									{routingSteps.length} step{routingSteps.length !== 1 ? 's' : ''} will be created
								</p>
							</div>
						{:else if productId}
							<p class="text-sm text-muted-foreground">No routing defined for this product</p>
						{:else}
							<p class="text-sm text-muted-foreground">Select a product to preview routing</p>
						{/if}
					</CardContent>
				</Card>

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
								{saving ? 'Creating...' : 'Create Lot'}
							</Button>
							<Button type="button" variant="outline" onclick={() => goto('/manufacturing/lots')}>
								Cancel
							</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
