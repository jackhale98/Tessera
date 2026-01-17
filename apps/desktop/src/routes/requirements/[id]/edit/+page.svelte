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
	let reqType = $state('input');
	let level = $state('system');
	let text = $state('');
	let rationale = $state('');
	let priority = $state('medium');
	let acceptanceCriteria = $state<string[]>([]);
	let tags = $state<string[]>([]);
	let newCriterion = $state('');
	let newTag = $state('');

	async function loadData() {
		if (!$isProjectOpen || !id) return;

		loading = true;
		error = null;

		try {
			const result = await entities.get(id);
			if (result) {
				entity = result;
				// Populate form fields
				title = result.title;
				tags = [...result.tags];

				const data = result.data ?? {};
				reqType = (data.req_type as string) ?? 'input';
				level = (data.level as string) ?? 'system';
				text = (data.text as string) ?? '';
				rationale = (data.rationale as string) ?? '';
				priority = (data.priority as string) ?? 'medium';
				acceptanceCriteria = [...((data.acceptance_criteria as string[]) ?? [])];
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function addCriterion() {
		if (newCriterion.trim()) {
			acceptanceCriteria = [...acceptanceCriteria, newCriterion.trim()];
			newCriterion = '';
		}
	}

	function removeCriterion(index: number) {
		acceptanceCriteria = acceptanceCriteria.filter((_, i) => i !== index);
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

		saving = true;
		error = null;

		try {
			// Start with existing entity data to preserve all fields
			const existingData = entity?.data ?? {};

			// Build updated data, preserving existing fields
			const data: Record<string, unknown> = {
				...existingData,
				id,
				title: title.trim(),
				req_type: reqType,
				level,
				text: text.trim(),
				priority,
				author: entity?.author ?? $projectAuthor,
				acceptance_criteria: acceptanceCriteria,
				tags,
				status: entity?.status ?? 'draft',
				created: entity?.created ?? new Date().toISOString(),
				entity_revision: ((existingData.entity_revision as number) ?? 0) + 1
			};

			if (rationale.trim()) {
				data.rationale = rationale.trim();
			} else {
				delete data.rationale;
			}

			await entities.save('REQ', data);
			await refreshProject();
			goto(`/requirements/${id}`);
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
			<Button variant="ghost" size="sm" onclick={() => goto(`/requirements/${id}`)}>
				<ArrowLeft class="mr-2 h-4 w-4" />Back to Requirement
			</Button>
			<div>
				<h1 class="text-2xl font-bold">Edit Requirement</h1>
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
								<Input id="title" bind:value={title} placeholder="Enter requirement title" required />
							</div>
							<div class="grid gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="req-type">Type *</Label>
									<Select id="req-type" bind:value={reqType}>
										<option value="input">Input (Stakeholder Need)</option>
										<option value="output">Output (Design Solution)</option>
									</Select>
								</div>
								<div class="space-y-2">
									<Label for="level">Level *</Label>
									<Select id="level" bind:value={level}>
										<option value="stakeholder">Stakeholder</option>
										<option value="system">System</option>
										<option value="subsystem">Subsystem</option>
										<option value="component">Component</option>
										<option value="detail">Detail</option>
									</Select>
								</div>
							</div>
							<div class="space-y-2">
								<Label for="text">Requirement Text *</Label>
								<Textarea id="text" bind:value={text} placeholder="The system shall..." rows={4} />
							</div>
							<div class="space-y-2">
								<Label for="rationale">Rationale</Label>
								<Textarea id="rationale" bind:value={rationale} placeholder="Why is this requirement needed?" rows={3} />
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>Acceptance Criteria</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="flex gap-2">
								<Input bind:value={newCriterion} placeholder="Add acceptance criterion" onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addCriterion())} />
								<Button type="button" variant="outline" onclick={addCriterion}><Plus class="h-4 w-4" /></Button>
							</div>
							{#if acceptanceCriteria.length > 0}
								<ul class="space-y-2">
									{#each acceptanceCriteria as criterion, i}
										<li class="flex items-center justify-between rounded-lg border p-3">
											<span class="text-sm">{criterion}</span>
											<Button type="button" variant="ghost" size="sm" onclick={() => removeCriterion(i)}><X class="h-4 w-4" /></Button>
										</li>
									{/each}
								</ul>
							{:else}
								<p class="text-sm text-muted-foreground">No acceptance criteria</p>
							{/if}
						</CardContent>
					</Card>
				</div>

				<div class="space-y-6">
					<Card>
						<CardHeader><CardTitle>Properties</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="space-y-2">
								<Label for="priority">Priority *</Label>
								<Select id="priority" bind:value={priority}>
									<option value="low">Low</option>
									<option value="medium">Medium</option>
									<option value="high">High</option>
									<option value="critical">Critical</option>
								</Select>
							</div>
						</CardContent>
					</Card>

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
								<Button type="button" variant="outline" onclick={() => goto(`/requirements/${id}`)}>Cancel</Button>
							</div>
						</CardContent>
					</Card>
				</div>
			</div>
		</form>
	{:else}
		<Card><CardContent class="flex h-64 items-center justify-center"><p class="text-muted-foreground">Requirement not found</p></CardContent></Card>
	{/if}
</div>
