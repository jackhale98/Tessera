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
		Textarea
	} from '$lib/components/ui';
	import { entities } from '$lib/api';
	import { refreshProject, projectAuthor } from '$lib/stores/project';
	import { ArrowLeft, Plus, X } from 'lucide-svelte';

	let title = $state('');
	let partNumber = $state('');
	let revision = $state('');
	let description = $state('');
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
				author: $projectAuthor,
				bom: [],
				subassemblies: [],
				tags
			};

			if (revision.trim()) data.revision = revision.trim();
			if (description.trim()) data.description = description.trim();

			const newId = await entities.save('ASM', data);
			await refreshProject();
			goto(`/assemblies/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to create assembly:', e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/assemblies')}>
			<ArrowLeft class="mr-2 h-4 w-4" />
			Back to Assemblies
		</Button>

		<div>
			<h1 class="text-2xl font-bold">New Assembly</h1>
			<p class="text-muted-foreground">Create a new assembly (BOM items can be added after creation)</p>
		</div>
	</div>

	{#if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{/if}

	<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
		<div class="grid gap-6 lg:grid-cols-3">
			<div class="space-y-6 lg:col-span-2">
				<Card>
					<CardHeader>
						<CardTitle>Assembly Information</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<Label for="title">Title *</Label>
							<Input id="title" bind:value={title} placeholder="Assembly name" required />
						</div>

						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="part-number">Part Number *</Label>
								<Input id="part-number" bind:value={partNumber} placeholder="e.g., ASM-0001" required />
							</div>
							<div class="space-y-2">
								<Label for="revision">Revision</Label>
								<Input id="revision" bind:value={revision} placeholder="e.g., A" />
							</div>
						</div>

						<div class="space-y-2">
							<Label for="description">Description</Label>
							<Textarea id="description" bind:value={description} placeholder="Assembly description" rows={3} />
						</div>
					</CardContent>
				</Card>
			</div>

			<div class="space-y-6">
				<Card>
					<CardHeader>
						<CardTitle>Tags</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex gap-2">
							<Input bind:value={newTag} placeholder="Add tag" onkeydown={handleTagKeydown} />
							<Button type="button" variant="outline" onclick={addTag}><Plus class="h-4 w-4" /></Button>
						</div>
						{#if tags.length > 0}
							<div class="flex flex-wrap gap-2">
								{#each tags as tag}
									<span class="inline-flex items-center gap-1 rounded-full border px-3 py-1 text-sm">
										{tag}
										<button type="button" class="text-muted-foreground hover:text-foreground" onclick={() => removeTag(tag)}><X class="h-3 w-3" /></button>
									</span>
								{/each}
							</div>
						{/if}
					</CardContent>
				</Card>

				<Card>
					<CardContent class="pt-6">
						<div class="flex flex-col gap-2">
							<Button type="submit" disabled={saving}>{saving ? 'Creating...' : 'Create Assembly'}</Button>
							<Button type="button" variant="outline" onclick={() => goto('/assemblies')}>Cancel</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
