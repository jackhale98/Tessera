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
	import { capas } from '$lib/api/tauri';
	import { refreshProject, projectAuthor } from '$lib/stores/project';
	import { ArrowLeft, Plus, X } from 'lucide-svelte';

	// Query params
	const initialNcrId = $derived($page.url.searchParams.get('ncrId') ?? '');

	let title = $state('');
	let description = $state('');
	let capaType = $state('corrective');
	let sourceNcr = $state('');
	let dueDate = $state('');
	let tags = $state<string[]>([]);
	let newTag = $state('');

	let saving = $state(false);
	let error = $state<string | null>(null);

	// Initialize NCR link from query param
	$effect(() => {
		if (initialNcrId && !sourceNcr) {
			sourceNcr = initialNcrId;
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
			const result = await capas.create({
				title: title.trim(),
				description: description.trim() || undefined,
				capa_type: capaType,
				source_ncr: sourceNcr || undefined,
				author: $projectAuthor
			});

			const newId = typeof result === 'string' ? result : (result as Record<string, unknown>)?.id as string;

			await refreshProject();
			goto(`/quality/capas/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to create CAPA:', e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/quality/capas')}>
			<ArrowLeft class="mr-2 h-4 w-4" />
			Back to CAPAs
		</Button>

		<div>
			<h1 class="text-2xl font-bold">New CAPA</h1>
			<p class="text-muted-foreground">Create a corrective or preventive action</p>
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
								placeholder="Brief description of the corrective/preventive action"
								required
							/>
						</div>

						<div class="space-y-2">
							<Label for="capa-type">CAPA Type</Label>
							<Select id="capa-type" bind:value={capaType}>
								<option value="corrective">Corrective</option>
								<option value="preventive">Preventive</option>
							</Select>
						</div>

						<div class="space-y-2">
							<Label for="description">Description</Label>
							<Textarea
								id="description"
								bind:value={description}
								placeholder="Detailed description of the action to be taken, root cause analysis, and expected outcomes"
								rows={4}
							/>
						</div>
					</CardContent>
				</Card>

				<!-- Source NCR -->
				<Card>
					<CardHeader>
						<CardTitle>Source NCR</CardTitle>
					</CardHeader>
					<CardContent>
						<EntityPicker
							entityTypes={['NCR']}
							placeholder="Search NCRs..."
							value={sourceNcr}
							onSelect={(entity) => { sourceNcr = entity.id; }}
							onClear={() => { sourceNcr = ''; }}
							label="Originating Non-Conformance"
						/>
						<p class="mt-2 text-xs text-muted-foreground">
							Link this CAPA to the NCR that triggered it
						</p>
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
							<Label for="due-date">Due Date</Label>
							<Input
								id="due-date"
								type="date"
								bind:value={dueDate}
							/>
						</div>
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
								{saving ? 'Creating...' : 'Create CAPA'}
							</Button>
							<Button type="button" variant="outline" onclick={() => goto('/quality/capas')}>
								Cancel
							</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
