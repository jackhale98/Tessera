<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { ncrs } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { AlertTriangle } from 'lucide-svelte';

	let title = $state('');
	let severity = $state('major');
	let ncr_type = $state('internal');
	let description = $state('');
	let submitting = $state(false);
	let error = $state('');

	let lotId = $derived($page.url.searchParams.get('lotId'));
	let canSubmit = $derived(title.trim().length > 0 && !submitting);

	async function handleSubmit() {
		if (!canSubmit) return;
		submitting = true;
		error = '';
		try {
			await ncrs.create({
				title: title.trim(),
				severity,
				ncr_type,
				description: description.trim() || undefined,
				author: 'mobile-user'
			});
			goto('/quality/ncrs');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create NCR';
			submitting = false;
		}
	}
</script>

<MobileHeader title="New NCR" backHref="/quality/ncrs" />

<div class="form-page">
	{#if lotId}
		<div class="lot-banner">
			<AlertTriangle size={16} />
			<span>Creating NCR for lot <strong>{lotId}</strong></span>
		</div>
	{/if}

	{#if error}
		<div class="error-banner">
			<p>{error}</p>
		</div>
	{/if}

	<form onsubmit={e => { e.preventDefault(); handleSubmit(); }}>
		<!-- Title -->
		<div class="form-group">
			<label class="form-label" for="ncr-title">Title</label>
			<input
				id="ncr-title"
				type="text"
				class="form-input"
				placeholder="Describe the non-conformance..."
				bind:value={title}
				required
			/>
		</div>

		<!-- Severity -->
		<div class="form-group">
			<label class="form-label">Severity</label>
			<div class="toggle-group">
				<button
					type="button"
					class="toggle-btn"
					class:active={severity === 'minor'}
					class:minor={severity === 'minor'}
					onclick={() => severity = 'minor'}
				>
					Minor
				</button>
				<button
					type="button"
					class="toggle-btn"
					class:active={severity === 'major'}
					class:major={severity === 'major'}
					onclick={() => severity = 'major'}
				>
					Major
				</button>
				<button
					type="button"
					class="toggle-btn"
					class:active={severity === 'critical'}
					class:critical={severity === 'critical'}
					onclick={() => severity = 'critical'}
				>
					Critical
				</button>
			</div>
		</div>

		<!-- NCR Type -->
		<div class="form-group">
			<label class="form-label" for="ncr-type">Type</label>
			<select id="ncr-type" class="form-select" bind:value={ncr_type}>
				<option value="internal">Internal</option>
				<option value="supplier">Supplier</option>
				<option value="customer">Customer</option>
			</select>
		</div>

		<!-- Description -->
		<div class="form-group">
			<label class="form-label" for="ncr-desc">Description</label>
			<textarea
				id="ncr-desc"
				class="form-textarea"
				placeholder="Provide details about the non-conformance..."
				bind:value={description}
				rows={4}
			></textarea>
		</div>

		<!-- Submit -->
		<button
			type="submit"
			class="submit-btn"
			disabled={!canSubmit}
		>
			{#if submitting}
				<div class="btn-spinner"></div>
				Creating...
			{:else}
				Create NCR
			{/if}
		</button>
	</form>
</div>

<style>
	.form-page {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding-bottom: 32px;
	}

	.lot-banner {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 14px;
		background-color: color-mix(in oklch, var(--theme-warning) 12%, transparent);
		border: 1px solid color-mix(in oklch, var(--theme-warning) 30%, transparent);
		border-radius: 12px;
		font-size: 13px;
		color: var(--theme-foreground);
	}

	.error-banner {
		padding: 12px 14px;
		background-color: color-mix(in oklch, var(--theme-error) 12%, transparent);
		border: 1px solid color-mix(in oklch, var(--theme-error) 30%, transparent);
		border-radius: 12px;
		font-size: 13px;
		color: var(--theme-error);
	}

	form {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.form-label {
		font-size: 13px;
		font-weight: 600;
		color: var(--theme-muted-foreground);
		text-transform: uppercase;
		letter-spacing: 0.04em;
		padding-left: 2px;
	}

	.form-input,
	.form-select,
	.form-textarea {
		width: 100%;
		padding: 14px 16px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 14px;
		color: var(--theme-foreground);
		font-size: 16px;
		outline: none;
		transition: border-color 0.15s ease;
		-webkit-appearance: none;
		box-sizing: border-box;
	}

	.form-input:focus,
	.form-select:focus,
	.form-textarea:focus {
		border-color: var(--theme-primary);
		box-shadow: 0 0 0 3px color-mix(in oklch, var(--theme-primary) 15%, transparent);
	}

	.form-input::placeholder,
	.form-textarea::placeholder {
		color: var(--theme-muted-foreground);
	}

	.form-select {
		background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%23888' stroke-width='2'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
		background-repeat: no-repeat;
		background-position: right 14px center;
		padding-right: 40px;
	}

	.form-textarea {
		resize: vertical;
		min-height: 100px;
		font-family: inherit;
		line-height: 1.5;
	}

	.toggle-group {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
	}

	.toggle-btn {
		padding: 14px 12px;
		border-radius: 14px;
		font-size: 15px;
		font-weight: 600;
		border: 2px solid var(--theme-border);
		background-color: var(--theme-card);
		color: var(--theme-muted-foreground);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.toggle-btn:active {
		transform: scale(0.96);
	}

	.toggle-btn.active {
		border-width: 2px;
	}

	.toggle-btn.minor {
		background-color: color-mix(in oklch, var(--theme-warning) 15%, var(--theme-card));
		border-color: var(--theme-warning);
		color: var(--theme-foreground);
	}

	.toggle-btn.major {
		background-color: color-mix(in oklch, var(--theme-info) 15%, var(--theme-card));
		border-color: var(--theme-info);
		color: var(--theme-foreground);
	}

	.toggle-btn.critical {
		background-color: color-mix(in oklch, var(--theme-error) 15%, var(--theme-card));
		border-color: var(--theme-error);
		color: var(--theme-foreground);
	}

	.submit-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
		padding: 16px;
		border-radius: 14px;
		font-size: 16px;
		font-weight: 700;
		background-color: var(--theme-primary);
		color: var(--theme-primary-foreground);
		border: none;
		cursor: pointer;
		transition: all 0.15s ease;
		margin-top: 4px;
	}

	.submit-btn:active:not(:disabled) {
		transform: scale(0.98);
	}

	.submit-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-spinner {
		width: 18px;
		height: 18px;
		border: 2px solid color-mix(in oklch, var(--theme-primary-foreground) 30%, transparent);
		border-top-color: var(--theme-primary-foreground);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
