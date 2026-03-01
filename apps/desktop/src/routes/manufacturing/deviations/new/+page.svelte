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
	import { deviations } from '$lib/api/tauri';
	import { refreshProject, projectAuthor } from '$lib/stores/project';
	import { ArrowLeft, Plus, X } from 'lucide-svelte';

	// Query params
	const initialLotId = $derived($page.url.searchParams.get('lotId') ?? '');
	const initialProcessId = $derived($page.url.searchParams.get('processId') ?? '');

	let title = $state('');
	let description = $state('');
	let deviationType = $state('process');
	let category = $state('');
	let riskLevel = $state('low');
	let riskAssessment = $state('');
	let effectiveDate = $state('');
	let expirationDate = $state('');
	let notes = $state('');
	let tags = $state<string[]>([]);
	let newTag = $state('');
	let linkedProcessId = $state('');
	let linkedLotId = $state('');

	let saving = $state(false);
	let error = $state<string | null>(null);

	// Initialize links from query params
	$effect(() => {
		if (initialLotId && !linkedLotId) {
			linkedLotId = initialLotId;
		}
	});

	$effect(() => {
		if (initialProcessId && !linkedProcessId) {
			linkedProcessId = initialProcessId;
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
			const result = await deviations.create({
				title: title.trim(),
				description: description.trim() || undefined,
				deviation_type: deviationType,
				category: category.trim() || undefined,
				risk_level: riskLevel,
				risk_assessment: riskAssessment.trim() || undefined,
				effective_date: effectiveDate || undefined,
				expiration_date: expirationDate || undefined,
				notes: notes.trim() || undefined,
				author: $projectAuthor
			});

			const newId = typeof result === 'string' ? result : (result as Record<string, unknown>)?.id as string;

			// Link to process if specified
			if (linkedProcessId && newId) {
				try {
					await deviations.addProcessLink(newId, linkedProcessId);
				} catch (linkErr) {
					console.error('Failed to link deviation to process:', linkErr);
				}
			}

			// Link to lot if specified
			if (linkedLotId && newId) {
				try {
					await deviations.addLotLink(newId, linkedLotId);
				} catch (linkErr) {
					console.error('Failed to link deviation to lot:', linkErr);
				}
			}

			await refreshProject();
			goto(`/manufacturing/deviations/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to create deviation:', e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/manufacturing/deviations')}>
			<ArrowLeft class="mr-2 h-4 w-4" />
			Back to Deviations
		</Button>

		<div>
			<h1 class="text-2xl font-bold">New Deviation</h1>
			<p class="text-muted-foreground">Request a deviation from standard process or specification</p>
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
								placeholder="Brief description of the deviation"
								required
							/>
						</div>

						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="dev-type">Deviation Type</Label>
								<Select id="dev-type" bind:value={deviationType}>
									<option value="process">Process</option>
									<option value="material">Material</option>
									<option value="specification">Specification</option>
									<option value="other">Other</option>
								</Select>
							</div>

							<div class="space-y-2">
								<Label for="category">Category</Label>
								<Input
									id="category"
									bind:value={category}
									placeholder="e.g., Material, Equipment"
								/>
							</div>
						</div>

						<div class="space-y-2">
							<Label for="description">Description</Label>
							<Textarea
								id="description"
								bind:value={description}
								placeholder="Detailed description of the deviation and its justification"
								rows={4}
							/>
						</div>
					</CardContent>
				</Card>

				<!-- Risk Assessment -->
				<Card>
					<CardHeader>
						<CardTitle>Risk Assessment</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<Label for="risk-level">Risk Level</Label>
							<Select id="risk-level" bind:value={riskLevel}>
								<option value="low">Low</option>
								<option value="medium">Medium</option>
								<option value="high">High</option>
							</Select>
						</div>

						<div class="space-y-2">
							<Label for="risk-assessment">Risk Assessment</Label>
							<Textarea
								id="risk-assessment"
								bind:value={riskAssessment}
								placeholder="Describe the risk impact and any mitigation measures"
								rows={3}
							/>
						</div>
					</CardContent>
				</Card>

				<!-- Dates & Notes -->
				<Card>
					<CardHeader>
						<CardTitle>Dates & Notes</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="effective-date">Effective Date</Label>
								<Input
									id="effective-date"
									type="date"
									bind:value={effectiveDate}
								/>
							</div>

							<div class="space-y-2">
								<Label for="expiration-date">Expiration Date</Label>
								<Input
									id="expiration-date"
									type="date"
									bind:value={expirationDate}
								/>
							</div>
						</div>

						<div class="space-y-2">
							<Label for="notes">Notes</Label>
							<Textarea
								id="notes"
								bind:value={notes}
								placeholder="Additional notes or context"
								rows={3}
							/>
						</div>
					</CardContent>
				</Card>
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Process Link -->
				<Card>
					<CardHeader>
						<CardTitle>Process Link</CardTitle>
					</CardHeader>
					<CardContent>
						<EntityPicker
							entityTypes={['PROC']}
							placeholder="Search processes..."
							value={linkedProcessId}
							onSelect={(entity) => { linkedProcessId = entity.id; }}
							onClear={() => { linkedProcessId = ''; }}
							label="Affected Process"
						/>
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
								{saving ? 'Creating...' : 'Create Deviation'}
							</Button>
							<Button type="button" variant="outline" onclick={() => goto('/manufacturing/deviations')}>
								Cancel
							</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
