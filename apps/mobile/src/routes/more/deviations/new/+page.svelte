<script lang="ts">
	import { goto } from '$app/navigation';
	import { deviations } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';

	let title = $state('');
	let deviation_type = $state('temporary');
	let category = $state('material');
	let risk_level = $state('low');
	let description = $state('');
	let author = $state('');

	let submitting = $state(false);
	let error = $state('');

	async function handleSubmit() {
		if (!title.trim() || !author.trim()) {
			error = 'Title and author are required.';
			return;
		}

		submitting = true;
		error = '';

		try {
			await deviations.create({
				title: title.trim(),
				deviation_type,
				category,
				risk_level,
				description: description.trim() || undefined,
				author: author.trim()
			});
			goto('/more/deviations');
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			submitting = false;
		}
	}
</script>

<MobileHeader title="New Deviation" backHref="/more/deviations" />

<div class="form-page">
	<form onsubmit={e => { e.preventDefault(); handleSubmit(); }}>
		<!-- Title -->
		<div class="field">
			<label class="field-label" for="dev-title">Title *</label>
			<input
				id="dev-title"
				type="text"
				class="field-input"
				placeholder="Deviation title"
				bind:value={title}
				required
			/>
		</div>

		<!-- Deviation Type -->
		<div class="field">
			<label class="field-label" for="dev-type">Deviation Type</label>
			<select id="dev-type" class="field-select" bind:value={deviation_type}>
				<option value="temporary">Temporary</option>
				<option value="permanent">Permanent</option>
				<option value="emergency">Emergency</option>
			</select>
		</div>

		<!-- Category -->
		<div class="field">
			<label class="field-label" for="dev-category">Category</label>
			<select id="dev-category" class="field-select" bind:value={category}>
				<option value="material">Material</option>
				<option value="process">Process</option>
				<option value="equipment">Equipment</option>
				<option value="tooling">Tooling</option>
				<option value="specification">Specification</option>
				<option value="documentation">Documentation</option>
			</select>
		</div>

		<!-- Risk Level -->
		<div class="field">
			<label class="field-label" for="dev-risk">Risk Level</label>
			<select id="dev-risk" class="field-select" bind:value={risk_level}>
				<option value="low">Low</option>
				<option value="medium">Medium</option>
				<option value="high">High</option>
			</select>
		</div>

		<!-- Description -->
		<div class="field">
			<label class="field-label" for="dev-desc">Description</label>
			<textarea
				id="dev-desc"
				class="field-textarea"
				placeholder="Describe the deviation..."
				bind:value={description}
				rows="4"
			></textarea>
		</div>

		<!-- Author -->
		<div class="field">
			<label class="field-label" for="dev-author">Author *</label>
			<input
				id="dev-author"
				type="text"
				class="field-input"
				placeholder="Your name"
				bind:value={author}
				required
			/>
		</div>

		{#if error}
			<div class="error-card">
				<p>{error}</p>
			</div>
		{/if}

		<button type="submit" class="submit-btn" disabled={submitting}>
			{submitting ? 'Creating...' : 'Create Deviation'}
		</button>
	</form>
</div>

<style>
	.form-page {
		padding: 16px;
		padding-bottom: 32px;
	}

	form {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.field-label {
		font-size: 13px;
		font-weight: 600;
		color: var(--theme-muted-foreground);
		text-transform: uppercase;
		letter-spacing: 0.04em;
		padding-left: 2px;
	}

	.field-input,
	.field-select,
	.field-textarea {
		width: 100%;
		padding: 12px 14px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 12px;
		color: var(--theme-foreground);
		font-size: 15px;
		font-family: inherit;
		outline: none;
		transition: border-color 0.15s ease;
	}

	.field-input:focus,
	.field-select:focus,
	.field-textarea:focus {
		border-color: var(--theme-primary);
	}

	.field-input::placeholder,
	.field-textarea::placeholder {
		color: var(--theme-muted-foreground);
	}

	.field-select {
		appearance: none;
		background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23888' d='M6 8.825c-.2 0-.4-.075-.55-.225l-3-3a.75.75 0 1 1 1.1-1.05L6 7l2.45-2.45a.75.75 0 0 1 1.1 1.05l-3 3a.776.776 0 0 1-.55.225z'/%3E%3C/svg%3E");
		background-repeat: no-repeat;
		background-position: right 14px center;
		padding-right: 36px;
	}

	.field-textarea {
		resize: vertical;
		min-height: 100px;
		line-height: 1.5;
	}

	.error-card {
		padding: 12px 16px;
		background-color: color-mix(in oklch, var(--theme-error) 15%, transparent);
		border: 1px solid var(--theme-error);
		border-radius: 12px;
		color: var(--theme-error);
		font-size: 13px;
	}

	.submit-btn {
		width: 100%;
		padding: 14px 16px;
		border-radius: 14px;
		font-size: 16px;
		font-weight: 600;
		border: none;
		background-color: var(--theme-primary);
		color: var(--theme-primary-foreground);
		cursor: pointer;
		transition: all 0.15s ease;
		-webkit-tap-highlight-color: transparent;
	}

	.submit-btn:active {
		transform: scale(0.98);
	}

	.submit-btn:disabled {
		opacity: 0.5;
	}
</style>
