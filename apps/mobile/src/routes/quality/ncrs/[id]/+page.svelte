<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { ncrs } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { StatusBadge } from '$lib/components/common/index.js';
	import { ArrowRight, ShieldAlert } from 'lucide-svelte';

	let ncr = $state<Record<string, unknown> | null>(null);
	let loading = $state(true);
	let advancing = $state(false);

	let ncrId = $derived($page.params.id);

	const statusFlow = ['open', 'containment', 'investigation', 'disposition', 'closed'];

	let currentStatusIndex = $derived(
		ncr ? statusFlow.indexOf(ncr.ncr_status as string) : -1
	);
	let nextStatus = $derived(
		currentStatusIndex >= 0 && currentStatusIndex < statusFlow.length - 1
			? statusFlow[currentStatusIndex + 1]
			: null
	);

	onMount(async () => {
		await loadNcr();
	});

	async function loadNcr() {
		loading = true;
		try {
			const data = await ncrs.get(ncrId);
			ncr = data as Record<string, unknown>;
		} finally {
			loading = false;
		}
	}

	async function handleAdvanceStatus() {
		if (advancing || !nextStatus) return;
		advancing = true;
		try {
			await ncrs.advanceStatus(ncrId);
			await loadNcr();
		} catch (e) {
			console.error('Failed to advance status:', e);
		} finally {
			advancing = false;
		}
	}

	function formatDate(val: unknown): string {
		if (!val) return '';
		try {
			return new Date(val as string).toLocaleDateString();
		} catch {
			return String(val);
		}
	}

	function capitalize(s: string): string {
		return s.charAt(0).toUpperCase() + s.slice(1).replace(/_/g, ' ');
	}
</script>

<MobileHeader
	title={ncr?.title as string ?? 'Loading...'}
	backHref="/quality/ncrs"
/>

{#if loading}
	<div class="loading-container">
		<div class="loading-spinner"></div>
	</div>
{:else if ncr}
	<div class="detail-page">
		<!-- Status card -->
		<div class="status-card">
			<div class="status-row">
				<span class="status-label">Status</span>
				<StatusBadge status={ncr.ncr_status as string} />
			</div>
			{#if ncr.ncr_status !== 'closed'}
				<div class="status-progress">
					{#each statusFlow as step, i}
						<div
							class="status-dot"
							class:completed={i <= currentStatusIndex}
							class:current={i === currentStatusIndex}
						></div>
						{#if i < statusFlow.length - 1}
							<div
								class="status-line"
								class:completed={i < currentStatusIndex}
							></div>
						{/if}
					{/each}
				</div>
				<div class="status-labels">
					{#each statusFlow as step}
						<span class="status-step-label">{capitalize(step)}</span>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Severity indicator -->
		{#if ncr.severity}
			<div class="severity-card severity-{ncr.severity}">
				<ShieldAlert size={20} />
				<div class="severity-text">
					<span class="severity-level">{capitalize(ncr.severity as string)} Severity</span>
					<span class="severity-desc">
						{#if ncr.severity === 'critical'}
							Requires immediate containment action
						{:else if ncr.severity === 'major'}
							Requires investigation and corrective action
						{:else}
							Monitor and document for trending
						{/if}
					</span>
				</div>
			</div>
		{/if}

		<!-- Info grid -->
		<section class="section">
			<h2 class="section-title">Details</h2>
			<div class="info-grid">
				{#if ncr.ncr_type}
					<div class="info-item">
						<span class="info-label">Type</span>
						<span class="info-value">{capitalize(ncr.ncr_type as string)}</span>
					</div>
				{/if}
				{#if ncr.category}
					<div class="info-item">
						<span class="info-label">Category</span>
						<span class="info-value">{capitalize(ncr.category as string)}</span>
					</div>
				{/if}
				{#if ncr.author}
					<div class="info-item">
						<span class="info-label">Author</span>
						<span class="info-value">{ncr.author}</span>
					</div>
				{/if}
				{#if ncr.created}
					<div class="info-item">
						<span class="info-label">Created</span>
						<span class="info-value">{formatDate(ncr.created)}</span>
					</div>
				{/if}
				{#if ncr.ncr_number}
					<div class="info-item">
						<span class="info-label">NCR Number</span>
						<span class="info-value">{ncr.ncr_number}</span>
					</div>
				{/if}
			</div>
		</section>

		<!-- Description -->
		{#if ncr.description}
			<section class="section">
				<h2 class="section-title">Description</h2>
				<div class="description-card">
					<p>{ncr.description}</p>
				</div>
			</section>
		{/if}

		<!-- Advance status button -->
		{#if nextStatus}
			<button
				class="advance-btn"
				onclick={handleAdvanceStatus}
				disabled={advancing}
			>
				{#if advancing}
					<div class="btn-spinner"></div>
					Advancing...
				{:else}
					<ArrowRight size={18} />
					Advance to {capitalize(nextStatus)}
				{/if}
			</button>
		{/if}
	</div>
{/if}

<style>
	.detail-page {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 20px;
		padding-bottom: 32px;
	}

	.status-card {
		background: linear-gradient(135deg, var(--theme-card), color-mix(in oklch, var(--theme-primary) 5%, var(--theme-card)));
		border: 1px solid var(--theme-border);
		border-radius: 16px;
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.status-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.status-label {
		font-size: 13px;
		font-weight: 600;
		color: var(--theme-muted-foreground);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.status-progress {
		display: flex;
		align-items: center;
		gap: 0;
		padding: 0 4px;
	}

	.status-dot {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background-color: var(--theme-muted);
		flex-shrink: 0;
		transition: all 0.3s ease;
	}

	.status-dot.completed {
		background-color: var(--theme-primary);
	}

	.status-dot.current {
		box-shadow: 0 0 0 3px color-mix(in oklch, var(--theme-primary) 25%, transparent);
	}

	.status-line {
		flex: 1;
		height: 3px;
		background-color: var(--theme-muted);
		transition: background-color 0.3s ease;
	}

	.status-line.completed {
		background-color: var(--theme-primary);
	}

	.status-labels {
		display: flex;
		justify-content: space-between;
		padding: 0 0;
	}

	.status-step-label {
		font-size: 9px;
		font-weight: 600;
		color: var(--theme-muted-foreground);
		text-align: center;
		width: 0;
		flex: 1;
	}

	.status-step-label:first-child { text-align: left; }
	.status-step-label:last-child { text-align: right; }

	.severity-card {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 16px;
		border-radius: 14px;
		border: 1px solid;
	}

	.severity-card.severity-minor {
		background-color: color-mix(in oklch, var(--theme-warning) 10%, transparent);
		border-color: color-mix(in oklch, var(--theme-warning) 30%, transparent);
		color: var(--theme-warning);
	}

	.severity-card.severity-major {
		background-color: color-mix(in oklch, var(--theme-info) 10%, transparent);
		border-color: color-mix(in oklch, var(--theme-info) 30%, transparent);
		color: var(--theme-info);
	}

	.severity-card.severity-critical {
		background-color: color-mix(in oklch, var(--theme-error) 10%, transparent);
		border-color: color-mix(in oklch, var(--theme-error) 30%, transparent);
		color: var(--theme-error);
	}

	.severity-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.severity-level {
		font-size: 15px;
		font-weight: 700;
	}

	.severity-desc {
		font-size: 12px;
		opacity: 0.8;
	}

	.section { display: flex; flex-direction: column; gap: 10px; }
	.section-title { font-size: 16px; font-weight: 700; padding: 0 4px; }

	.info-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 8px;
	}

	.info-item {
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 12px;
		padding: 12px 14px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.info-label { font-size: 11px; font-weight: 600; color: var(--theme-muted-foreground); text-transform: uppercase; letter-spacing: 0.05em; }
	.info-value { font-size: 15px; font-weight: 600; }

	.description-card {
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 14px;
		padding: 16px;
	}

	.description-card p {
		font-size: 14px;
		line-height: 1.6;
		color: var(--theme-foreground);
		margin: 0;
		white-space: pre-wrap;
	}

	.advance-btn {
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
	}

	.advance-btn:active:not(:disabled) {
		transform: scale(0.98);
	}

	.advance-btn:disabled {
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

	.loading-container { display: flex; justify-content: center; padding: 64px 16px; }
	.loading-spinner { width: 32px; height: 32px; border: 3px solid var(--theme-border); border-top-color: var(--theme-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
	@keyframes spin { to { transform: rotate(360deg); } }
</style>
