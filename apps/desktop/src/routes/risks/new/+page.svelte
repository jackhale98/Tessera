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

	let saving = $state(false);
	let error = $state<string | null>(null);

	// Calculate RPN
	const rpn = $derived(
		severity && occurrence && detection ? severity * occurrence * detection : null
	);

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
		if (!description.trim()) {
			error = 'Description is required';
			return;
		}

		saving = true;
		error = null;

		try {
			const data: Record<string, unknown> = {
				title: title.trim(),
				risk_type: riskType,
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

			const newId = await entities.save('RISK', data);
			await refreshProject();
			goto(`/risks/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to create risk:', e);
		} finally {
			saving = false;
		}
	}

	function getRpnColor(rpn: number | null): string {
		if (!rpn) return 'text-muted-foreground';
		if (rpn >= 200) return 'text-destructive';
		if (rpn >= 100) return 'text-orange-500';
		if (rpn >= 50) return 'text-yellow-500';
		return 'text-green-500';
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/risks')}>
			<ArrowLeft class="mr-2 h-4 w-4" />
			Back to Risks
		</Button>

		<div>
			<h1 class="text-2xl font-bold">New Risk</h1>
			<p class="text-muted-foreground">Create a new FMEA risk entry</p>
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
								placeholder="Brief risk title"
								required
							/>
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
							<Textarea
								id="description"
								bind:value={description}
								placeholder="Detailed description of the risk"
								rows={3}
							/>
						</div>
					</CardContent>
				</Card>

				<!-- FMEA Analysis -->
				<Card>
					<CardHeader>
						<CardTitle>FMEA Analysis</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<Label for="failure-mode">Failure Mode</Label>
							<Textarea
								id="failure-mode"
								bind:value={failureMode}
								placeholder="How could this fail?"
								rows={2}
							/>
						</div>

						<div class="space-y-2">
							<Label for="cause">Cause</Label>
							<Textarea
								id="cause"
								bind:value={cause}
								placeholder="What could cause this failure?"
								rows={2}
							/>
						</div>

						<div class="space-y-2">
							<Label for="effect">Effect</Label>
							<Textarea
								id="effect"
								bind:value={riskEffect}
								placeholder="What are the consequences?"
								rows={2}
							/>
						</div>

						<!-- RPN Ratings -->
						<div class="mt-6">
							<h4 class="mb-4 text-sm font-medium">Risk Priority Number (RPN)</h4>
							<div class="grid gap-4 sm:grid-cols-4">
								<div class="space-y-2">
									<Label for="severity">Severity (1-10)</Label>
									<Input
										id="severity"
										type="number"
										min="1"
										max="10"
										bind:value={severity}
										placeholder="1-10"
									/>
								</div>
								<div class="space-y-2">
									<Label for="occurrence">Occurrence (1-10)</Label>
									<Input
										id="occurrence"
										type="number"
										min="1"
										max="10"
										bind:value={occurrence}
										placeholder="1-10"
									/>
								</div>
								<div class="space-y-2">
									<Label for="detection">Detection (1-10)</Label>
									<Input
										id="detection"
										type="number"
										min="1"
										max="10"
										bind:value={detection}
										placeholder="1-10"
									/>
								</div>
								<div class="space-y-2">
									<Label>RPN</Label>
									<div class={`flex h-9 items-center rounded-md border px-3 font-bold ${getRpnColor(rpn)}`}>
										{rpn ?? '—'}
									</div>
								</div>
							</div>
							<p class="mt-2 text-xs text-muted-foreground">
								RPN = Severity × Occurrence × Detection (Range: 1-1000)
							</p>
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
								placeholder="e.g., Electrical, Mechanical"
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
								{saving ? 'Creating...' : 'Create Risk'}
							</Button>
							<Button type="button" variant="outline" onclick={() => goto('/risks')}>
								Cancel
							</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
