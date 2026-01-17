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
	import { ArrowLeft, Plus, X, Trash2 } from 'lucide-svelte';
	import type { EntityData } from '$lib/api/types';

	interface ProcedureStep {
		step: number;
		action: string;
		expected: string;
		acceptance: string;
	}

	interface EquipmentItem {
		name: string;
		specification: string;
		calibration_required: boolean;
	}

	const id = $derived($page.params.id);

	let entity = $state<EntityData | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let error = $state<string | null>(null);

	// Form fields
	let title = $state('');
	let testType = $state('verification');
	let testLevel = $state('system');
	let testMethod = $state('test');
	let objective = $state('');
	let description = $state('');
	let preconditions = $state<string[]>([]);
	let newPrecondition = $state('');
	let procedure = $state<ProcedureStep[]>([]);
	let equipment = $state<EquipmentItem[]>([]);
	let acceptanceCriteria = $state<string[]>([]);
	let newCriterion = $state('');
	let estimatedDuration = $state('');
	let priority = $state('medium');
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
				testType = (data.type as string) ?? 'verification';
				testLevel = (data.test_level as string) ?? 'system';
				testMethod = (data.test_method as string) ?? 'test';
				objective = (data.objective as string) ?? '';
				description = (data.description as string) ?? '';
				estimatedDuration = (data.estimated_duration as string) ?? '';
				priority = (data.priority as string) ?? 'medium';
				preconditions = ((data.preconditions as string[]) ?? []);
				acceptanceCriteria = ((data.acceptance_criteria as string[]) ?? []);

				// Load procedure
				const rawProcedure = (data.procedure as Array<Record<string, unknown>>) ?? [];
				procedure = rawProcedure.map(s => ({
					step: (s.step as number) ?? 1,
					action: (s.action as string) ?? '',
					expected: (s.expected as string) ?? '',
					acceptance: (s.acceptance as string) ?? ''
				}));

				// Load equipment
				const rawEquipment = (data.equipment as Array<Record<string, unknown>>) ?? [];
				equipment = rawEquipment.map(e => ({
					name: (e.name as string) ?? '',
					specification: (e.specification as string) ?? '',
					calibration_required: (e.calibration_required as boolean) ?? false
				}));
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function addPrecondition() {
		if (newPrecondition.trim()) {
			preconditions = [...preconditions, newPrecondition.trim()];
			newPrecondition = '';
		}
	}

	function removePrecondition(index: number) {
		preconditions = preconditions.filter((_, i) => i !== index);
	}

	function addStep() {
		procedure = [...procedure, {
			step: procedure.length + 1,
			action: '',
			expected: '',
			acceptance: ''
		}];
	}

	function removeStep(index: number) {
		procedure = procedure.filter((_, i) => i !== index).map((s, i) => ({ ...s, step: i + 1 }));
	}

	function addEquipment() {
		equipment = [...equipment, {
			name: '',
			specification: '',
			calibration_required: false
		}];
	}

	function removeEquipment(index: number) {
		equipment = equipment.filter((_, i) => i !== index);
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

	async function handleSubmit() {
		if (!title.trim()) { error = 'Title is required'; return; }

		saving = true;
		error = null;

		try {
			// Start with existing entity data to preserve all fields (links, results references, etc.)
			const existingData = entity?.data ?? {};

			// Build updated data, preserving existing fields
			const data: Record<string, unknown> = {
				...existingData,
				id,
				title: title.trim(),
				type: testType,
				test_level: testLevel,
				test_method: testMethod,
				priority,
				author: entity?.author ?? $projectAuthor,
				tags,
				status: entity?.status ?? 'draft',
				created: entity?.created ?? new Date().toISOString(),
				entity_revision: ((existingData.entity_revision as number) ?? 0) + 1
			};

			if (objective.trim()) {
				data.objective = objective.trim();
			} else {
				delete data.objective;
			}
			if (description.trim()) {
				data.description = description.trim();
			} else {
				delete data.description;
			}
			if (estimatedDuration.trim()) {
				data.estimated_duration = estimatedDuration.trim();
			} else {
				delete data.estimated_duration;
			}

			if (preconditions.length > 0) {
				data.preconditions = preconditions;
			} else {
				delete data.preconditions;
			}
			if (acceptanceCriteria.length > 0) {
				data.acceptance_criteria = acceptanceCriteria;
			} else {
				delete data.acceptance_criteria;
			}

			// Filter out empty steps
			const validSteps = procedure.filter(s => s.action.trim());
			if (validSteps.length > 0) {
				data.procedure = validSteps.map(s => ({
					step: s.step,
					action: s.action.trim(),
					expected: s.expected.trim(),
					...(s.acceptance.trim() && { acceptance: s.acceptance.trim() })
				}));
			} else {
				delete data.procedure;
			}

			// Filter out empty equipment
			const validEquipment = equipment.filter(e => e.name.trim());
			if (validEquipment.length > 0) {
				data.equipment = validEquipment.map(e => ({
					name: e.name.trim(),
					...(e.specification.trim() && { specification: e.specification.trim() }),
					calibration_required: e.calibration_required
				}));
			} else {
				delete data.equipment;
			}

			await entities.save('TEST', data);
			await refreshProject();
			goto(`/verification/tests/${id}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}

	onMount(() => { loadData(); });

	$effect(() => {
		if ($isProjectOpen && id) loadData();
	});
</script>

<div class="space-y-6">
	{#if loading}
		<div class="flex h-64 items-center justify-center">
			<div class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
		</div>
	{:else if entity}
		<div class="space-y-4">
			<Button variant="ghost" size="sm" onclick={() => goto(`/verification/tests/${id}`)}>
				<ArrowLeft class="mr-2 h-4 w-4" />Back to Test
			</Button>
			<div>
				<h1 class="text-2xl font-bold">Edit Test Protocol</h1>
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
								<Input id="title" bind:value={title} placeholder="Test name" required />
							</div>
							<div class="grid gap-4 sm:grid-cols-3">
								<div class="space-y-2">
									<Label for="test-type">Test Type *</Label>
									<Select id="test-type" bind:value={testType}>
										<option value="verification">Verification</option>
										<option value="validation">Validation</option>
									</Select>
								</div>
								<div class="space-y-2">
									<Label for="test-level">Test Level</Label>
									<Select id="test-level" bind:value={testLevel}>
										<option value="unit">Unit</option>
										<option value="subsystem">Subsystem</option>
										<option value="system">System</option>
										<option value="integration">Integration</option>
									</Select>
								</div>
								<div class="space-y-2">
									<Label for="test-method">Method</Label>
									<Select id="test-method" bind:value={testMethod}>
										<option value="inspection">Inspection</option>
										<option value="analysis">Analysis</option>
										<option value="demonstration">Demonstration</option>
										<option value="test">Test</option>
									</Select>
								</div>
							</div>
							<div class="space-y-2">
								<Label for="objective">Objective</Label>
								<Textarea id="objective" bind:value={objective} placeholder="What this test aims to verify" rows={2} />
							</div>
							<div class="space-y-2">
								<Label for="description">Description</Label>
								<Textarea id="description" bind:value={description} placeholder="Detailed test description" rows={3} />
							</div>
						</CardContent>
					</Card>

					<!-- Preconditions -->
					<Card>
						<CardHeader><CardTitle>Preconditions</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="flex gap-2">
								<Input bind:value={newPrecondition} placeholder="Add precondition" onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addPrecondition())} />
								<Button type="button" variant="outline" onclick={addPrecondition}><Plus class="h-4 w-4" /></Button>
							</div>
							{#if preconditions.length > 0}
								<ul class="space-y-2">
									{#each preconditions as condition, i}
										<li class="flex items-center justify-between rounded-lg border p-3">
											<span class="text-sm">{condition}</span>
											<Button type="button" variant="ghost" size="sm" onclick={() => removePrecondition(i)}><X class="h-4 w-4" /></Button>
										</li>
									{/each}
								</ul>
							{/if}
						</CardContent>
					</Card>

					<!-- Procedure -->
					<Card>
						<CardHeader class="flex flex-row items-center justify-between">
							<CardTitle>Procedure Steps</CardTitle>
							<Button type="button" variant="outline" size="sm" onclick={addStep}>
								<Plus class="mr-2 h-4 w-4" /> Add Step
							</Button>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if procedure.length === 0}
								<p class="text-sm text-muted-foreground">No steps added yet.</p>
							{:else}
								{#each procedure as step, i}
									<div class="rounded-lg border p-4 space-y-4">
										<div class="flex items-start justify-between">
											<span class="flex h-6 w-6 items-center justify-center rounded-full bg-muted text-sm font-medium">{step.step}</span>
											<Button type="button" variant="ghost" size="sm" onclick={() => removeStep(i)}>
												<Trash2 class="h-4 w-4" />
											</Button>
										</div>
										<div class="space-y-2">
											<Label for={`step-action-${i}`}>Action</Label>
											<Textarea id={`step-action-${i}`} bind:value={step.action} placeholder="What to do" rows={2} />
										</div>
										<div class="space-y-2">
											<Label for={`step-expected-${i}`}>Expected Result</Label>
											<Textarea id={`step-expected-${i}`} bind:value={step.expected} placeholder="What should happen" rows={2} />
										</div>
										<div class="space-y-2">
											<Label for={`step-acceptance-${i}`}>Acceptance Criteria</Label>
											<Input id={`step-acceptance-${i}`} bind:value={step.acceptance} placeholder="Pass/fail criteria" />
										</div>
									</div>
								{/each}
							{/if}
						</CardContent>
					</Card>

					<!-- Equipment -->
					<Card>
						<CardHeader class="flex flex-row items-center justify-between">
							<CardTitle>Equipment</CardTitle>
							<Button type="button" variant="outline" size="sm" onclick={addEquipment}>
								<Plus class="mr-2 h-4 w-4" /> Add Equipment
							</Button>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if equipment.length === 0}
								<p class="text-sm text-muted-foreground">No equipment specified.</p>
							{:else}
								{#each equipment as item, i}
									<div class="rounded-lg border p-4 space-y-4">
										<div class="flex items-start justify-between">
											<span class="text-sm font-medium text-muted-foreground">Equipment #{i + 1}</span>
											<Button type="button" variant="ghost" size="sm" onclick={() => removeEquipment(i)}>
												<Trash2 class="h-4 w-4" />
											</Button>
										</div>
										<div class="grid gap-4 sm:grid-cols-2">
											<div class="space-y-2">
												<Label for={`equip-name-${i}`}>Name</Label>
												<Input id={`equip-name-${i}`} bind:value={item.name} placeholder="Equipment name" />
											</div>
											<div class="space-y-2">
												<Label for={`equip-spec-${i}`}>Specification</Label>
												<Input id={`equip-spec-${i}`} bind:value={item.specification} placeholder="Requirements/range" />
											</div>
										</div>
										<label class="flex items-center gap-2">
											<input type="checkbox" bind:checked={item.calibration_required} class="h-4 w-4 rounded border" />
											<span class="text-sm">Calibration Required</span>
										</label>
									</div>
								{/each}
							{/if}
						</CardContent>
					</Card>

					<!-- Acceptance Criteria -->
					<Card>
						<CardHeader><CardTitle>Overall Acceptance Criteria</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="flex gap-2">
								<Input bind:value={newCriterion} placeholder="Add criterion" onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addCriterion())} />
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
							{/if}
						</CardContent>
					</Card>
				</div>

				<div class="space-y-6">
					<Card>
						<CardHeader><CardTitle>Properties</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="space-y-2">
								<Label for="priority">Priority</Label>
								<Select id="priority" bind:value={priority}>
									<option value="low">Low</option>
									<option value="medium">Medium</option>
									<option value="high">High</option>
									<option value="critical">Critical</option>
								</Select>
							</div>
							<div class="space-y-2">
								<Label for="duration">Estimated Duration</Label>
								<Input id="duration" bind:value={estimatedDuration} placeholder="e.g., 2 hours" />
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
								<Button type="button" variant="outline" onclick={() => goto(`/verification/tests/${id}`)}>Cancel</Button>
							</div>
						</CardContent>
					</Card>
				</div>
			</div>
		</form>
	{:else}
		<Card><CardContent class="flex h-64 items-center justify-center"><p class="text-muted-foreground">Test not found</p></CardContent></Card>
	{/if}
</div>
