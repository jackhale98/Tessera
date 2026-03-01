<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { capas } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { StatusBadge } from '$lib/components/common/index.js';
	import { ArrowRight, CheckCircle } from 'lucide-svelte';

	let capa = $state<Record<string, unknown> | null>(null);
	let loading = $state(true);
	let advancing = $state(false);
	let verifying = $state(false);

	let capaId = $derived($page.params.id);

	const statusFlow = ['initiation', 'investigation', 'implementation', 'verification', 'closed'];

	let currentStatusIndex = $derived(
		capa ? statusFlow.indexOf(capa.capa_status as string) : -1
	);
	let nextStatus = $derived(
		currentStatusIndex >= 0 && currentStatusIndex < statusFlow.length - 1
			? statusFlow[currentStatusIndex + 1]
			: null
	);
	let canVerifyEffectiveness = $derived(
		capa?.capa_status === 'verification' && !capa?.effectiveness_verified
	);

	onMount(async () => {
		await loadCapa();
	});

	async function loadCapa() {
		loading = true;
		try {
			const data = await capas.get(capaId);
			capa = data as Record<string, unknown>;
		} finally {
			loading = false;
		}
	}

	async function handleAdvanceStatus() {
		if (advancing || !nextStatus) return;
		advancing = true;
		try {
			await capas.advanceStatus(capaId);
			await loadCapa();
		} catch (e) {
			console.error('Failed to advance status:', e);
		} finally {
			advancing = false;
		}
	}

	async function handleVerifyEffectiveness() {
		if (verifying) return;
		verifying = true;
		try {
			await capas.verifyEffectiveness(capaId, {
				effective: true,
				verified_by: 'mobile-user'
			});
			await loadCapa();
		} catch (e) {
			console.error('Failed to verify effectiveness:', e);
		} finally {
			verifying = false;
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

	function isDueDateOverdue(dueDate: unknown, status: unknown): boolean {
		if (!dueDate || status === 'closed') return false;
		return new Date(dueDate as string) < new Date();
	}
</script>

<MobileHeader
	title={capa?.title as string ?? 'Loading...'}
	backHref="/quality/capas"
/>

{#if loading}
	<div class="loading-container">
		<div class="loading-spinner"></div>
	</div>
{:else if capa}
	<div class="detail-page">
		<!-- Status card -->
		<div class="status-card">
			<div class="status-row">
				<span class="status-label">Status</span>
				<StatusBadge status={capa.capa_status as string} />
			</div>
			{#if capa.capa_status !== 'closed'}
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

		<!-- Due date warning -->
		{#if capa.due_date && isDueDateOverdue(capa.due_date, capa.capa_status)}
			<div class="overdue-banner">
				<span class="overdue-text">Overdue</span>
				<span class="overdue-date">Due {formatDate(capa.due_date)}</span>
			</div>
		{/if}

		<!-- Effectiveness badge -->
		{#if capa.effectiveness_verified}
			<div class="effectiveness-card">
				<CheckCircle size={20} />
				<div class="effectiveness-text">
					<span class="effectiveness-title">Effectiveness Verified</span>
					<span class="effectiveness-desc">Corrective actions confirmed effective</span>
				</div>
			</div>
		{/if}

		<!-- Info grid -->
		<section class="section">
			<h2 class="section-title">Details</h2>
			<div class="info-grid">
				{#if capa.capa_type}
					<div class="info-item">
						<span class="info-label">Type</span>
						<span class="info-value">{capitalize(capa.capa_type as string)}</span>
					</div>
				{/if}
				{#if capa.capa_number}
					<div class="info-item">
						<span class="info-label">CAPA Number</span>
						<span class="info-value">{capa.capa_number}</span>
					</div>
				{/if}
				{#if capa.due_date}
					<div class="info-item">
						<span class="info-label">Due Date</span>
						<span class="info-value" class:overdue-value={isDueDateOverdue(capa.due_date, capa.capa_status)}>
							{formatDate(capa.due_date)}
						</span>
					</div>
				{/if}
				{#if capa.author}
					<div class="info-item">
						<span class="info-label">Author</span>
						<span class="info-value">{capa.author}</span>
					</div>
				{/if}
				{#if capa.created}
					<div class="info-item">
						<span class="info-label">Created</span>
						<span class="info-value">{formatDate(capa.created)}</span>
					</div>
				{/if}
				{#if capa.source_ncr}
					<div class="info-item">
						<span class="info-label">Source NCR</span>
						<span class="info-value source-link">{capa.source_ncr}</span>
					</div>
				{/if}
			</div>
		</section>

		<!-- Description -->
		{#if capa.description}
			<section class="section">
				<h2 class="section-title">Description</h2>
				<div class="description-card">
					<p>{capa.description}</p>
				</div>
			</section>
		{/if}

		<!-- Action buttons -->
		<div class="action-buttons">
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

			{#if canVerifyEffectiveness}
				<button
					class="verify-btn"
					onclick={handleVerifyEffectiveness}
					disabled={verifying}
				>
					{#if verifying}
						<div class="btn-spinner verify"></div>
						Verifying...
					{:else}
						<CheckCircle size={18} />
						Verify Effectiveness
					{/if}
				</button>
			{/if}
		</div>
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

	.overdue-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 14px 16px;
		background-color: color-mix(in oklch, var(--theme-error) 10%, transparent);
		border: 1px solid color-mix(in oklch, var(--theme-error) 30%, transparent);
		border-radius: 14px;
	}

	.overdue-text {
		font-size: 14px;
		font-weight: 700;
		color: var(--theme-error);
	}

	.overdue-date {
		font-size: 13px;
		color: var(--theme-error);
		opacity: 0.8;
	}

	.effectiveness-card {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 16px;
		background-color: color-mix(in oklch, var(--theme-success) 10%, transparent);
		border: 1px solid color-mix(in oklch, var(--theme-success) 30%, transparent);
		border-radius: 14px;
		color: var(--theme-success);
	}

	.effectiveness-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.effectiveness-title {
		font-size: 15px;
		font-weight: 700;
	}

	.effectiveness-desc {
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
	.info-value.overdue-value { color: var(--theme-error); }
	.source-link { font-family: var(--font-mono); font-size: 12px; word-break: break-all; }

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

	.action-buttons {
		display: flex;
		flex-direction: column;
		gap: 10px;
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

	.verify-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
		padding: 16px;
		border-radius: 14px;
		font-size: 16px;
		font-weight: 700;
		background-color: var(--theme-success);
		color: white;
		border: none;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.verify-btn:active:not(:disabled) {
		transform: scale(0.98);
	}

	.verify-btn:disabled {
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

	.btn-spinner.verify {
		border-color: color-mix(in oklch, white 30%, transparent);
		border-top-color: white;
	}

	.loading-container { display: flex; justify-content: center; padding: 64px 16px; }
	.loading-spinner { width: 32px; height: 32px; border: 3px solid var(--theme-border); border-top-color: var(--theme-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
	@keyframes spin { to { transform: rotate(360deg); } }
</style>
