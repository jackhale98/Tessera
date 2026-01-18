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
	import { ArrowLeft, Plus, X, Ruler, Trash2 } from 'lucide-svelte';

	let title = $state('');
	let componentId = $state('');
	let featureType = $state('external');
	let description = $state('');
	let geometryClass = $state('');
	let datumLabel = $state('');
	let drawingNumber = $state('');
	let drawingRevision = $state('');
	let drawingZone = $state('');
	let tags = $state<string[]>([]);
	let newTag = $state('');

	let saving = $state(false);
	let error = $state<string | null>(null);

	// Dimension management
	interface NewDimension {
		name: string;
		nominal: number;
		plus_tol: number;
		minus_tol: number;
		units: string;
		internal: boolean;
		distribution: string;
	}
	let dimensions = $state<NewDimension[]>([]);

	// Distribution type options
	const distributionTypes = [
		{ value: 'normal', label: 'Normal (Gaussian)' },
		{ value: 'uniform', label: 'Uniform' },
		{ value: 'triangular', label: 'Triangular' }
	];

	// Unit options
	const unitOptions = ['mm', 'in', 'um', 'mil'];

	function addDimension() {
		dimensions = [...dimensions, {
			name: '',
			nominal: 0,
			plus_tol: 0.1,
			minus_tol: 0.1,
			units: 'mm',
			internal: featureType === 'internal',
			distribution: 'normal'
		}];
	}

	function removeDimension(index: number) {
		dimensions = dimensions.filter((_, i) => i !== index);
	}

	function addTag() {
		if (newTag.trim() && !tags.includes(newTag.trim())) {
			tags = [...tags, newTag.trim()];
			newTag = '';
		}
	}

	async function handleSubmit() {
		if (!title.trim()) { error = 'Title is required'; return; }
		if (!componentId.trim()) { error = 'Component ID is required'; return; }

		saving = true;
		error = null;

		try {
			// Build dimensions list (only those with names)
			const validDimensions = dimensions
				.filter(d => d.name.trim())
				.map(d => ({
					name: d.name.trim(),
					nominal: d.nominal,
					plus_tol: d.plus_tol,
					minus_tol: d.minus_tol,
					units: d.units,
					internal: d.internal,
					distribution: d.distribution
				}));

			const data: Record<string, unknown> = {
				title: title.trim(),
				component: componentId.trim(),
				feature_type: featureType,
				author: $projectAuthor,
				dimensions: validDimensions,
				gdt: [],
				tags
			};

			if (description.trim()) data.description = description.trim();
			if (geometryClass) data.geometry_class = geometryClass;
			if (datumLabel.trim()) data.datum_label = datumLabel.trim().toUpperCase();
			if (drawingNumber.trim()) {
				data.drawing = {
					number: drawingNumber.trim(),
					revision: drawingRevision.trim() || 'A',
					zone: drawingZone.trim() || ''
				};
			}

			const newId = await entities.save('FEAT', data);
			await refreshProject();
			goto(`/features/${newId}`);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="space-y-6">
	<div class="space-y-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/features')}>
			<ArrowLeft class="mr-2 h-4 w-4" />Back to Features
		</Button>
		<div>
			<h1 class="text-2xl font-bold">New Feature</h1>
			<p class="text-muted-foreground">Define a geometric feature with dimensions and tolerances</p>
		</div>
	</div>

	{#if error}
		<Card class="border-destructive"><CardContent class="pt-6"><p class="text-destructive">{error}</p></CardContent></Card>
	{/if}

	<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
		<div class="grid gap-6 lg:grid-cols-3">
			<div class="space-y-6 lg:col-span-2">
				<Card>
					<CardHeader><CardTitle>Feature Information</CardTitle></CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<Label for="title">Title *</Label>
							<Input id="title" bind:value={title} placeholder="Feature name (e.g., Mounting Hole A)" required />
						</div>
						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="component">Component ID *</Label>
								<Input id="component" bind:value={componentId} placeholder="CMP-..." required />
								<p class="text-xs text-muted-foreground">Enter the parent component's ID</p>
							</div>
							<div class="space-y-2">
								<Label for="feature-type">Feature Type *</Label>
								<Select id="feature-type" bind:value={featureType}>
									<option value="internal">Internal (Hole/Bore)</option>
									<option value="external">External (Shaft/Boss)</option>
								</Select>
							</div>
						</div>
						<div class="space-y-2">
							<Label for="description">Description</Label>
							<Textarea id="description" bind:value={description} placeholder="Feature description" rows={2} />
						</div>
					</CardContent>
				</Card>

				<Card>
					<CardHeader><CardTitle>Geometry</CardTitle></CardHeader>
					<CardContent class="space-y-4">
						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="geometry-class">Geometry Class</Label>
								<Select id="geometry-class" bind:value={geometryClass}>
									<option value="">Not specified</option>
									<option value="plane">Plane</option>
									<option value="cylinder">Cylinder</option>
									<option value="sphere">Sphere</option>
									<option value="cone">Cone</option>
									<option value="point">Point</option>
									<option value="line">Line</option>
									<option value="complex">Complex</option>
								</Select>
							</div>
							<div class="space-y-2">
								<Label for="datum-label">Datum Label</Label>
								<Input id="datum-label" bind:value={datumLabel} placeholder="e.g., A, B, C" maxlength={3} />
								<p class="text-xs text-muted-foreground">If this feature is a datum</p>
							</div>
						</div>
					</CardContent>
				</Card>

				<Card>
					<CardHeader><CardTitle>Drawing Reference</CardTitle></CardHeader>
					<CardContent class="space-y-4">
						<div class="grid gap-4 sm:grid-cols-3">
							<div class="space-y-2">
								<Label for="drawing-number">Drawing Number</Label>
								<Input id="drawing-number" bind:value={drawingNumber} placeholder="DWG-001" />
							</div>
							<div class="space-y-2">
								<Label for="drawing-revision">Revision</Label>
								<Input id="drawing-revision" bind:value={drawingRevision} placeholder="A" />
							</div>
							<div class="space-y-2">
								<Label for="drawing-zone">Zone</Label>
								<Input id="drawing-zone" bind:value={drawingZone} placeholder="B3" />
							</div>
						</div>
					</CardContent>
				</Card>

				<!-- Dimensions -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle class="flex items-center gap-2">
								<Ruler class="h-5 w-5" />
								Dimensions ({dimensions.length})
							</CardTitle>
							<Button type="button" variant="outline" size="sm" onclick={addDimension}>
								<Plus class="mr-2 h-4 w-4" />
								Add Dimension
							</Button>
						</div>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if dimensions.length === 0}
							<p class="text-sm text-muted-foreground text-center py-4">
								No dimensions defined yet. You can add them now or later when editing.
							</p>
						{:else}
							{#each dimensions as dim, index}
								<div class="rounded-lg border p-4 space-y-4">
									<div class="flex items-center justify-between">
										<Label class="font-medium">Dimension {index + 1}</Label>
										<Button
											type="button"
											variant="ghost"
											size="sm"
											class="text-destructive hover:text-destructive"
											onclick={() => removeDimension(index)}
										>
											<Trash2 class="h-4 w-4" />
										</Button>
									</div>
									<div class="grid gap-4 sm:grid-cols-2">
										<div class="space-y-2">
											<Label for="dim-name-{index}">Name *</Label>
											<Input
												id="dim-name-{index}"
												bind:value={dim.name}
												placeholder="e.g., Diameter, Length"
											/>
										</div>
										<div class="space-y-2">
											<Label for="dim-dist-{index}">Distribution</Label>
											<Select id="dim-dist-{index}" bind:value={dim.distribution}>
												{#each distributionTypes as dt}
													<option value={dt.value}>{dt.label}</option>
												{/each}
											</Select>
											<p class="text-xs text-muted-foreground">For tolerance analysis</p>
										</div>
									</div>
									<div class="grid gap-4 sm:grid-cols-4">
										<div class="space-y-2">
											<Label for="dim-nom-{index}">Nominal</Label>
											<Input
												id="dim-nom-{index}"
												type="number"
												step="any"
												bind:value={dim.nominal}
											/>
										</div>
										<div class="space-y-2">
											<Label for="dim-plus-{index}">+Tolerance</Label>
											<Input
												id="dim-plus-{index}"
												type="number"
												step="any"
												bind:value={dim.plus_tol}
											/>
										</div>
										<div class="space-y-2">
											<Label for="dim-minus-{index}">-Tolerance</Label>
											<Input
												id="dim-minus-{index}"
												type="number"
												step="any"
												bind:value={dim.minus_tol}
											/>
										</div>
										<div class="space-y-2">
											<Label for="dim-units-{index}">Units</Label>
											<Select id="dim-units-{index}" bind:value={dim.units}>
												{#each unitOptions as unit}
													<option value={unit}>{unit}</option>
												{/each}
											</Select>
										</div>
									</div>
									<div class="flex items-center gap-2">
										<input
											type="checkbox"
											id="dim-internal-{index}"
											bind:checked={dim.internal}
											class="h-4 w-4 rounded border-gray-300"
										/>
										<Label for="dim-internal-{index}" class="text-sm font-normal">
											Internal dimension (hole/bore)
										</Label>
									</div>
								</div>
							{/each}
						{/if}
					</CardContent>
				</Card>
			</div>

			<div class="space-y-6">
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
							<Button type="submit" disabled={saving}>{saving ? 'Creating...' : 'Create Feature'}</Button>
							<Button type="button" variant="outline" onclick={() => goto('/features')}>Cancel</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	</form>
</div>
