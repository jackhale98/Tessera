<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { deviations } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { StatusBadge } from '$lib/components/common/index.js';
	import { truncateEntityId } from '$lib/config/entities.js';
	import { CheckCircle, Play, XCircle } from 'lucide-svelte';

	let dev = $state<Record<string, unknown> | null>(null);
	let loading = $state(true);
	let actionLoading = $state(false);

	let devId = $derived($page.params.id);

	onMount(async () => {
		await loadDeviation();
	});

	async function loadDeviation() {
		loading = true;
		try {
			dev = (await deviations.get(devId)) as Record<string, unknown>;
		} finally {
			loading = false;
		}
	}

	async function handleApprove() {
		actionLoading = true;
		try {
			await deviations.approve(devId, {
				approved_by: 'mobile-user',
				authorization_level: 'engineering',
				activate: true
			});
			await loadDeviation();
		} finally {
			actionLoading = false;
		}
	}

	async function handleActivate() {
		actionLoading = true;
		try {
			await deviations.activate(devId);
			await loadDeviation();
		} finally {
			actionLoading = false;
		}
	}

	async function handleClose() {
		actionLoading = true;
		try {
			await deviations.close(devId, 'Closed via mobile app');
			await loadDeviation();
		} finally {
			actionLoading = false;
		}
	}

	function formatFieldLabel(key: string): string {
		return key.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
	}
</script>

<MobileHeader
	title={dev?.title as string ?? 'Loading...'}
	backHref="/more/deviations"
/>

{#if loading}
	<div class="loading-container">
		<div class="loading-spinner"></div>
	</div>
{:else if dev}
	<div class="dev-detail">
		<!-- Status Card -->
		<div class="status-card">
			<div class="status-row">
				<span class="dev-id">{truncateEntityId(dev.id as string)}</span>
				<StatusBadge status={dev.dev_status as string ?? dev.status as string} />
			</div>
			<h2 class="dev-title">{dev.title}</h2>
		</div>

		<!-- Actions -->
		{#if dev.dev_status === 'pending'}
			<div class="actions">
				<button class="action-btn success" onclick={handleApprove} disabled={actionLoading}>
					<CheckCircle size={16} /> Approve & Activate
				</button>
			</div>
		{:else if dev.dev_status === 'approved'}
			<div class="actions">
				<button class="action-btn primary" onclick={handleActivate} disabled={actionLoading}>
					<Play size={16} /> Activate
				</button>
				<button class="action-btn destructive" onclick={handleClose} disabled={actionLoading}>
					<XCircle size={16} /> Close
				</button>
			</div>
		{:else if dev.dev_status === 'active'}
			<div class="actions">
				<button class="action-btn destructive" onclick={handleClose} disabled={actionLoading}>
					<XCircle size={16} /> Close Deviation
				</button>
			</div>
		{/if}

		<!-- Info Grid -->
		<section class="section">
			<h3 class="section-title">Details</h3>
			<div class="info-grid">
				{#if dev.deviation_type}
					<div class="info-item">
						<span class="info-label">Type</span>
						<span class="info-value">{formatFieldLabel(dev.deviation_type as string)}</span>
					</div>
				{/if}
				{#if dev.category}
					<div class="info-item">
						<span class="info-label">Category</span>
						<span class="info-value">{formatFieldLabel(dev.category as string)}</span>
					</div>
				{/if}
				{#if dev.risk_level || (dev.risk as Record<string, unknown>)?.level}
					<div class="info-item">
						<span class="info-label">Risk Level</span>
						<span class="info-value">{formatFieldLabel((dev.risk_level as string) ?? ((dev.risk as Record<string, unknown>)?.level as string) ?? '')}</span>
					</div>
				{/if}
				{#if dev.author}
					<div class="info-item">
						<span class="info-label">Author</span>
						<span class="info-value">{dev.author}</span>
					</div>
				{/if}
				{#if dev.created}
					<div class="info-item">
						<span class="info-label">Created</span>
						<span class="info-value">{new Date(dev.created as string).toLocaleDateString()}</span>
					</div>
				{/if}
				{#if dev.effective_date}
					<div class="info-item">
						<span class="info-label">Effective</span>
						<span class="info-value">{new Date(dev.effective_date as string).toLocaleDateString()}</span>
					</div>
				{/if}
				{#if dev.expiration_date}
					<div class="info-item">
						<span class="info-label">Expires</span>
						<span class="info-value">{new Date(dev.expiration_date as string).toLocaleDateString()}</span>
					</div>
				{/if}
				{#if dev.approved_by || (dev.approval as Record<string, unknown>)?.approved_by}
					<div class="info-item">
						<span class="info-label">Approved By</span>
						<span class="info-value">{(dev.approved_by as string) ?? ((dev.approval as Record<string, unknown>)?.approved_by as string)}</span>
					</div>
				{/if}
			</div>
		</section>

		<!-- Description -->
		{#if dev.description}
			<section class="section">
				<h3 class="section-title">Description</h3>
				<div class="description-card">
					<p>{dev.description}</p>
				</div>
			</section>
		{/if}

		<!-- Notes -->
		{#if dev.notes}
			<section class="section">
				<h3 class="section-title">Notes</h3>
				<div class="description-card">
					<p>{dev.notes}</p>
				</div>
			</section>
		{/if}
	</div>
{/if}

<style>
	.dev-detail {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 20px;
		padding-bottom: 32px;
	}

	.status-card {
		background: linear-gradient(135deg, var(--theme-card), color-mix(in oklch, var(--theme-warning) 5%, var(--theme-card)));
		border: 1px solid var(--theme-border);
		border-radius: 16px;
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.status-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.dev-id {
		font-size: 12px;
		font-weight: 600;
		font-family: var(--font-mono);
		color: var(--theme-muted-foreground);
		letter-spacing: 0.03em;
		text-transform: uppercase;
	}

	.dev-title {
		font-size: 18px;
		font-weight: 700;
		line-height: 1.35;
		letter-spacing: -0.01em;
	}

	.actions {
		display: flex;
		gap: 8px;
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		flex: 1;
		padding: 12px 16px;
		border-radius: 12px;
		font-size: 14px;
		font-weight: 600;
		border: 1px solid var(--theme-border);
		background-color: var(--theme-card);
		color: var(--theme-foreground);
		cursor: pointer;
		transition: transform 0.1s ease;
		-webkit-tap-highlight-color: transparent;
	}

	.action-btn:active { transform: scale(0.95); }
	.action-btn:disabled { opacity: 0.5; }
	.action-btn.primary { background-color: var(--theme-primary); color: var(--theme-primary-foreground); border-color: var(--theme-primary); }
	.action-btn.success { background-color: var(--theme-success); color: white; border-color: var(--theme-success); }
	.action-btn.destructive { background-color: color-mix(in oklch, var(--theme-error) 15%, transparent); color: var(--theme-error); border-color: var(--theme-error); }

	.section { display: flex; flex-direction: column; gap: 10px; }
	.section-title { font-size: 15px; font-weight: 700; padding: 0 4px; }

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
	.info-value { font-size: 14px; font-weight: 600; }

	.description-card {
		padding: 16px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 14px;
		font-size: 14px;
		line-height: 1.6;
		color: var(--theme-foreground);
	}

	.loading-container { display: flex; justify-content: center; padding: 64px 16px; }
	.loading-spinner { width: 32px; height: 32px; border: 3px solid var(--theme-border); border-top-color: var(--theme-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
	@keyframes spin { to { transform: rotate(360deg); } }
</style>
