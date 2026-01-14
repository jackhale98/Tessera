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

	interface Mitigation {
		action: string;
		type: 'prevention' | 'detection';
		status: 'proposed' | 'in_progress' | 'completed' | 'verified';
		owner: string;
		due_date: string;
	}

	const id = $derived($page.params.id);

	let entity = $state<EntityData | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let error = $state<string | null>(null);

	// Form fields
	let title = $state('');
	let riskType = $state('design');
	let description = $state('');
	let failureMode = $state('');
	let cause = $state('');
	let riskEffect = $state('');
	let severity = $state<number | null>(null);
	let occurrence = $state<number | null>(null);
	let detection = $state<number | null>(null);
	let category = $state('');
	let tags = $state<string[]>([]);
	let newTag = $state('');
	let mitigations = $state<Mitigation[]>([]);

	const rpn = $derived(
		severity && occurrence && detection ? severity * occurrence * detection : null
	);

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
				riskType = (data.risk_type as string) ?? (data.type as string) ?? 'design';
				description = (data.description as string) ?? '';
				failureMode = (data.failure_mode as string) ?? '';
				cause = (data.cause as string) ?? '';
				riskEffect = (data.effect as string) ?? '';
				severity = (data.severity as number) ?? null;
				occurrence = (data.occurrence as number) ?? null;
				detection = (data.detection as number) ?? null;
				category = (data.category as string) ?? '';

				// Load mitigations
				const rawMitigations = (data.mitigations as Array<Record<string, unknown>>) ?? [];
				mitigations = rawMitigations.map(m => ({
					action: (m.action as string) ?? '',
					type: ((m.type as string) ?? 'prevention') as 'prevention' | 'detection',
					status: ((m.status as string) ?? 'proposed') as 'proposed' | 'in_progress' | 'completed' | 'verified',
					owner: (m.owner as string) ?? '',
					due_date: (m.due_date as string) ?? ''
				}));
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

	function addMitigation() {
		mitigations = [...mitigations, {
			action: '',
			type: 'prevention',
			status: 'proposed',
			owner: '',
			due_date: ''
		}];
	}

	function removeMitigation(index: number) {
		mitigations = mitigations.filter((_, i) => i !== index);
	}

	function getRpnColor(rpn: number | null): string {
		if (!rpn) return 'text-muted-foreground';
		if (rpn >= 200) return 'text-destructive';
		if (rpn >= 100) return 'text-orange-500';
		if (rpn >= 50) return 'text-yellow-500';
		return 'text-green-500';
	}

	async function handleSubmit() {
		if (!title.trim()) { error = 'Title is required'; return; }
		if (!description.trim()) { error = 'Description is required'; return; }

		saving = true;
		error = null;

		try {
			const data: Record<string, unknown> = {
				title: title.trim(),
				risk_type: riskType,
				type: riskType,  // Also save as 'type' for compatibility with CLI
				description: description.trim(),
				author: $projectAuthor,
				tags
			};

			if (failureMode.trim()) data.failure_mode = failureMode.trim();
			if (cause.trim()) data.cause = cause.trim();
			if (riskEffect.trim()) data.effect = riskEffect.trim();
			if (severity !== null) data.severity = severity;
			if (occurrence !== null) data.occurrence = occurrence;
			if (detection !== null) data.detection = detection;
			if (category.trim()) data.category = category.trim();

			// Filter out empty mitigations and save
			const validMitigations = mitigations.filter(m => m.action.trim());
			data.mitigations = validMitigations.map(m => ({
				action: m.action.trim(),
				type: m.type,
				status: m.status,
				...(m.owner.trim() && { owner: m.owner.trim() }),
				...(m.due_date && { due_date: m.due_date })
			}));

			await entities.save('RISK', { ...data, id });
			await refreshProject();
			goto(`/risks/${id}`);
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
			<Button variant="ghost" size="sm" onclick={() => goto(`/risks/${id}`)}>
				<ArrowLeft class="mr-2 h-4 w-4" />Back to Risk
			</Button>
			<div>
				<h1 class="text-2xl font-bold">Edit Risk</h1>
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
								<Input id="title" bind:value={title} placeholder="Brief risk title" required />
							</div>
							<div class="space-y-2">
								<Label for="risk-type">Risk Type *</Label>
								<Select id="risk-type" bind:value={riskType}>
									<option value="design">Design</option>
									<option value="process">Process</option>
									<option value="use">Use</option>
									<option value="software">Software</option>
								</Select>
							</div>
							<div class="space-y-2">
								<Label for="description">Description *</Label>
								<Textarea id="description" bind:value={description} placeholder="Detailed description of the risk" rows={3} />
							</div>
						</CardContent>
					</Card>

					<Card>
						<CardHeader><CardTitle>FMEA Analysis</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="space-y-2">
								<Label for="failure-mode">Failure Mode</Label>
								<Textarea id="failure-mode" bind:value={failureMode} placeholder="How could this fail?" rows={2} />
							</div>
							<div class="space-y-2">
								<Label for="cause">Cause</Label>
								<Textarea id="cause" bind:value={cause} placeholder="What could cause this failure?" rows={2} />
							</div>
							<div class="space-y-2">
								<Label for="effect">Effect</Label>
								<Textarea id="effect" bind:value={riskEffect} placeholder="What are the consequences?" rows={2} />
							</div>

							<div class="mt-6">
								<h4 class="mb-4 text-sm font-medium">Risk Priority Number (RPN)</h4>
								<div class="grid gap-4 sm:grid-cols-4">
									<div class="space-y-2">
										<Label for="severity">Severity (1-10)</Label>
										<Input id="severity" type="number" min={1} max={10} bind:value={severity} placeholder="1-10" />
									</div>
									<div class="space-y-2">
										<Label for="occurrence">Occurrence (1-10)</Label>
										<Input id="occurrence" type="number" min={1} max={10} bind:value={occurrence} placeholder="1-10" />
									</div>
									<div class="space-y-2">
										<Label for="detection">Detection (1-10)</Label>
										<Input id="detection" type="number" min={1} max={10} bind:value={detection} placeholder="1-10" />
									</div>
									<div class="space-y-2">
										<Label>RPN</Label>
										<div class={`flex h-9 items-center rounded-md border px-3 font-bold ${getRpnColor(rpn)}`}>{rpn ?? '—'}</div>
									</div>
								</div>
								<p class="mt-2 text-xs text-muted-foreground">RPN = Severity × Occurrence × Detection (Range: 1-1000)</p>
							</div>
						</CardContent>
					</Card>

					<!-- Mitigations -->
					<Card>
						<CardHeader class="flex flex-row items-center justify-between">
							<CardTitle>Mitigations</CardTitle>
							<Button type="button" variant="outline" size="sm" onclick={addMitigation}>
								<Plus class="mr-2 h-4 w-4" /> Add Mitigation
							</Button>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if mitigations.length === 0}
								<p class="text-sm text-muted-foreground">No mitigations added yet.</p>
							{:else}
								{#each mitigations as mitigation, i}
									<div class="rounded-lg border p-4 space-y-4">
										<div class="flex items-start justify-between">
											<span class="text-sm font-medium text-muted-foreground">Mitigation #{i + 1}</span>
											<Button type="button" variant="ghost" size="sm" onclick={() => removeMitigation(i)}>
												<Trash2 class="h-4 w-4" />
											</Button>
										</div>
										<div class="space-y-2">
											<Label for={`mitigation-action-${i}`}>Action *</Label>
											<Textarea id={`mitigation-action-${i}`} bind:value={mitigation.action} placeholder="Describe the mitigation action" rows={2} />
										</div>
										<div class="grid gap-4 sm:grid-cols-2">
											<div class="space-y-2">
												<Label for={`mitigation-type-${i}`}>Type</Label>
												<Select id={`mitigation-type-${i}`} bind:value={mitigation.type}>
													<option value="prevention">Prevention</option>
													<option value="detection">Detection</option>
												</Select>
											</div>
											<div class="space-y-2">
												<Label for={`mitigation-status-${i}`}>Status</Label>
												<Select id={`mitigation-status-${i}`} bind:value={mitigation.status}>
													<option value="proposed">Proposed</option>
													<option value="in_progress">In Progress</option>
													<option value="completed">Completed</option>
													<option value="verified">Verified</option>
												</Select>
											</div>
										</div>
										<div class="grid gap-4 sm:grid-cols-2">
											<div class="space-y-2">
												<Label for={`mitigation-owner-${i}`}>Owner</Label>
												<Input id={`mitigation-owner-${i}`} bind:value={mitigation.owner} placeholder="Responsible person" />
											</div>
											<div class="space-y-2">
												<Label for={`mitigation-due-${i}`}>Due Date</Label>
												<Input id={`mitigation-due-${i}`} type="date" bind:value={mitigation.due_date} />
											</div>
										</div>
									</div>
								{/each}
							{/if}
						</CardContent>
					</Card>
				</div>

				<div class="space-y-6">
					<Card>
						<CardHeader><CardTitle>Properties</CardTitle></CardHeader>
						<CardContent class="space-y-4">
							<div class="space-y-2">
								<Label for="category">Category</Label>
								<Input id="category" bind:value={category} placeholder="e.g., Electrical, Mechanical" />
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
								<Button type="button" variant="outline" onclick={() => goto(`/risks/${id}`)}>Cancel</Button>
							</div>
						</CardContent>
					</Card>
				</div>
			</div>
		</form>
	{:else}
		<Card><CardContent class="flex h-64 items-center justify-center"><p class="text-muted-foreground">Risk not found</p></CardContent></Card>
	{/if}
</div>
