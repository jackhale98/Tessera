<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
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
	import { ncrs, lots } from '$lib/api/tauri';
	import { refreshProject, projectAuthor } from '$lib/stores/project';
	import { ArrowLeft, Plus, X } from 'lucide-svelte';

	// Query params
	const initialLotId = $derived($page.url.searchParams.get('lotId') ?? '');

	let title = $state('');
	let description = $state('');
	let ncrType = $state('internal');
	let severity = $state('major');
	let category = $state('');
	let tags = $state<string[]>([]);
	let newTag = $state('');
	let linkedLotId = $state('');

	let saving = $state(false);
	let error = $state<string | null>(null);

	// Initialize lot link from query param
	$effect(() => {
		if (initialLotId && !linkedLotId) {
			linkedLotId = initialLotId;
		}
	});

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

		saving = true;
		error = null;

		try {
			const result = await ncrs.create({
				title: title.trim(),
				description: description.trim() || undefined,
				ncr_type: ncrType,
				severity,
				category: category.trim() || undefined,
				author: $projectAuthor
			});

			const newId = typeof result === 'string' ? result : (result as Record<string, unknown>)?.id as string;

			// Link to lot if specified
			if (linkedLotId && newId) {
				try {
					await lots.addNcr(linkedLotId, newId);
				} catch (linkErr) {
					console.error('Failed to link NCR to lot:', linkErr);
				}
			}

			await refreshProject();
			goto(`/quality/ncrs/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to create NCR:', e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/quality/ncrs')}>
			<ArrowLeft class="mr-2 h-4 w-4" />
			Back to NCRs
		</Button>

		<div>
			<h1 class="text-2xl font-bold">New Non-Conformance</h1>
			<p class="text-muted-foreground">Record a non-conformance report</p>
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
								placeholder="Brief description of the non-conformance"
								required
							/>
						</div>

						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="ncr-type">NCR Type</Label>
								<Select id="ncr-type" bind:value={ncrType}>
									<option value="internal">Internal</option>
									<option value="supplier">Supplier</option>
									<option value="customer">Customer</option>
								</Select>
							</div>

							<div class="space-y-2">
								<Label for="severity">Severity</Label>
								<Select id="severity" bind:value={severity}>
									<option value="minor">Minor</option>
									<option value="major">Major</option>
									<option value="critical">Critical</option>
								</Select>
							</div>
						</div>

						<div class="space-y-2">
							<Label for="description">Description</Label>
							<Textarea
								id="description"
								bind:value={description}
								placeholder="Detailed description of the non-conformance, including affected parts, quantities, and observations"
								rows={4}
							/>
						</div>
					</CardContent>
				</Card>
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Properties -->
				<Card>
					<CardHeader>
						<CardTitle>Properties</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<Label for="category">Category</Label>
							<Input
								id="category"
								bind:value={category}
								placeholder="e.g., Dimensional, Material"
							/>
						</div>
					</CardContent>
				</Card>

				<!-- Lot Link -->
				<Card>
					<CardHeader>
						<CardTitle>Lot Link</CardTitle>
					</CardHeader>
					<CardContent>
						<EntityPicker
							entityTypes={['LOT']}
							placeholder="Search lots..."
							value={linkedLotId}
							onSelect={(entity) => { linkedLotId = entity.id; }}
							onClear={() => { linkedLotId = ''; }}
							label="Associated Lot"
						/>
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
								{saving ? 'Creating...' : 'Create NCR'}
							</Button>
							<Button type="button" variant="outline" onclick={() => goto('/quality/ncrs')}>
								Cancel
							</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
