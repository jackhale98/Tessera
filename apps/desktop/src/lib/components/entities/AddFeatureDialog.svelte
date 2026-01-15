<script lang="ts">
	import { Dialog, Button, Input, Label, Select } from '$lib/components/ui';
	import { entities, traceability } from '$lib/api';
	import { CircleDot } from 'lucide-svelte';

	interface Props {
		open: boolean;
		componentId: string;
		componentTitle?: string;
		onClose: () => void;
		onCreated?: (featureId: string) => void;
	}

	let { open = $bindable(), componentId, componentTitle, onClose, onCreated }: Props = $props();

	let title = $state('');
	let featureType = $state<'internal' | 'external'>('external');
	let geometryClass = $state('');
	let datumLabel = $state('');
	let saving = $state(false);
	let error = $state<string | null>(null);

	const geometryOptions = [
		{ value: '', label: 'Select geometry...' },
		{ value: 'planar', label: 'Planar' },
		{ value: 'cylindrical', label: 'Cylindrical' },
		{ value: 'spherical', label: 'Spherical' },
		{ value: 'conical', label: 'Conical' },
		{ value: 'prismatic', label: 'Prismatic' },
		{ value: 'complex', label: 'Complex' }
	];

	async function handleSubmit(e: Event) {
		e.preventDefault();

		if (!title.trim()) {
			error = 'Title is required';
			return;
		}

		saving = true;
		error = null;

		try {
			// Create the feature entity
			const featureData: Record<string, unknown> = {
				title: title.trim(),
				component: componentId,
				feature_type: featureType,
				status: 'draft',
				dimensions: [],
				gdt: [],
				links: {},
				created: new Date().toISOString(),
				author: 'TDT User', // TODO: Get from project config
				entity_revision: 1
			};

			if (geometryClass) {
				featureData.geometry_class = geometryClass;
			}

			if (datumLabel.trim()) {
				featureData.datum_label = datumLabel.trim().toUpperCase();
			}

			const featureId = await entities.save('FEAT', featureData);

			// Add link from component to feature
			await traceability.addLink(componentId, featureId, 'has_feature');

			// Reset form
			title = '';
			featureType = 'external';
			geometryClass = '';
			datumLabel = '';

			onCreated?.(featureId);
			onClose();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to create feature:', e);
		} finally {
			saving = false;
		}
	}

	function handleClose() {
		error = null;
		onClose();
	}
</script>

<Dialog bind:open onClose={handleClose}>
	<form onsubmit={handleSubmit} class="p-6">
		<div class="mb-6">
			<div class="flex items-center gap-3">
				<div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10">
					<CircleDot class="h-5 w-5 text-primary" />
				</div>
				<div>
					<h2 class="text-lg font-semibold">Add Feature</h2>
					{#if componentTitle}
						<p class="text-sm text-muted-foreground">to {componentTitle}</p>
					{/if}
				</div>
			</div>
		</div>

		{#if error}
			<div class="mb-4 rounded-md bg-destructive/10 p-3 text-sm text-destructive">
				{error}
			</div>
		{/if}

		<div class="space-y-4">
			<div>
				<Label for="title">Feature Title *</Label>
				<Input
					id="title"
					bind:value={title}
					placeholder="e.g., Bore Diameter, Mounting Face"
					class="mt-1.5"
					disabled={saving}
				/>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div>
					<Label for="featureType">Feature Type</Label>
					<Select
						id="featureType"
						bind:value={featureType}
						class="mt-1.5"
						disabled={saving}
					>
						<option value="external">External (shaft/boss)</option>
						<option value="internal">Internal (hole/pocket)</option>
					</Select>
				</div>

				<div>
					<Label for="geometryClass">Geometry Class</Label>
					<Select
						id="geometryClass"
						bind:value={geometryClass}
						class="mt-1.5"
						disabled={saving}
					>
						{#each geometryOptions as opt}
							<option value={opt.value}>{opt.label}</option>
						{/each}
					</Select>
				</div>
			</div>

			<div>
				<Label for="datumLabel">Datum Label (optional)</Label>
				<Input
					id="datumLabel"
					bind:value={datumLabel}
					placeholder="e.g., A, B, C"
					class="mt-1.5"
					maxlength={3}
					disabled={saving}
				/>
				<p class="mt-1 text-xs text-muted-foreground">
					If this feature is a datum, enter the label (A, B, C, etc.)
				</p>
			</div>
		</div>

		<div class="mt-6 flex justify-end gap-3">
			<Button type="button" variant="outline" onclick={handleClose} disabled={saving}>
				Cancel
			</Button>
			<Button type="submit" disabled={saving || !title.trim()}>
				{#if saving}
					Creating...
				{:else}
					Create Feature
				{/if}
			</Button>
		</div>
	</form>
</Dialog>
