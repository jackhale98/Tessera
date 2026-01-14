<script lang="ts">
	import { goto } from '$app/navigation';
	import {
		Card,
		CardContent,
		CardHeader,
		CardTitle,
		CardFooter,
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
	let reqType = $state('input');
	let level = $state('system');
	let text = $state('');
	let rationale = $state('');
	let priority = $state('medium');
	let acceptanceCriteria = $state<string[]>([]);
	let tags = $state<string[]>([]);
	let newCriterion = $state('');
	let newTag = $state('');

	let saving = $state(false);
	let error = $state<string | null>(null);

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

	function handleTagKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			addTag();
		}
	}

	function handleCriterionKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			addCriterion();
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
			const data: Record<string, unknown> = {
				title: title.trim(),
				req_type: reqType,
				level,
				text: text.trim(),
				priority,
				author: $projectAuthor,
				acceptance_criteria: acceptanceCriteria,
				tags
			};

			if (rationale.trim()) {
				data.rationale = rationale.trim();
			}

			const newId = await entities.save('REQ', data);
			await refreshProject();
			goto(`/requirements/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to create requirement:', e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/requirements')}>
			<ArrowLeft class="mr-2 h-4 w-4" />
			Back to Requirements
		</Button>

		<div>
			<h1 class="text-2xl font-bold">New Requirement</h1>
			<p class="text-muted-foreground">Create a new requirement</p>
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
								placeholder="Enter requirement title"
								required
							/>
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
							<Textarea
								id="text"
								bind:value={text}
								placeholder="The system shall..."
								rows={4}
							/>
						</div>

						<div class="space-y-2">
							<Label for="rationale">Rationale</Label>
							<Textarea
								id="rationale"
								bind:value={rationale}
								placeholder="Why is this requirement needed?"
								rows={3}
							/>
						</div>
					</CardContent>
				</Card>

				<!-- Acceptance Criteria -->
				<Card>
					<CardHeader>
						<CardTitle>Acceptance Criteria</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex gap-2">
							<Input
								bind:value={newCriterion}
								placeholder="Add acceptance criterion"
								onkeydown={handleCriterionKeydown}
							/>
							<Button type="button" variant="outline" onclick={addCriterion}>
								<Plus class="h-4 w-4" />
							</Button>
						</div>

						{#if acceptanceCriteria.length > 0}
							<ul class="space-y-2">
								{#each acceptanceCriteria as criterion, i}
									<li class="flex items-center justify-between rounded-lg border p-3">
										<span class="text-sm">{criterion}</span>
										<Button
											type="button"
											variant="ghost"
											size="sm"
											onclick={() => removeCriterion(i)}
										>
											<X class="h-4 w-4" />
										</Button>
									</li>
								{/each}
							</ul>
						{:else}
							<p class="text-sm text-muted-foreground">No acceptance criteria added yet</p>
						{/if}
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
								{saving ? 'Creating...' : 'Create Requirement'}
							</Button>
							<Button type="button" variant="outline" onclick={() => goto('/requirements')}>
								Cancel
							</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
