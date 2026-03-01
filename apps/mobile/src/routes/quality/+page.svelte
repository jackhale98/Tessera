<script lang="ts">
	import { onMount } from 'svelte';
	import { ncrs, capas } from '$lib/api/tauri.js';
	import type { NcrStats, CapaStats } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { ShieldAlert, ShieldCheck, ChevronRight, AlertTriangle, Clock } from 'lucide-svelte';

	let ncrStats = $state<NcrStats | null>(null);
	let capaStats = $state<CapaStats | null>(null);
	let loading = $state(true);

	onMount(async () => {
		try {
			const [n, c] = await Promise.all([ncrs.getStats(), capas.getStats()]);
			ncrStats = n;
			capaStats = c;
		} finally {
			loading = false;
		}
	});
</script>

<MobileHeader title="Quality" />

<div class="quality-page">
	{#if loading}
		<div class="loading-state">
			<div class="loading-spinner"></div>
		</div>
	{:else}
		<!-- Summary cards -->
		<div class="summary-grid">
			<div class="summary-card ncr">
				<div class="summary-icon">
					<ShieldAlert size={22} />
				</div>
				<span class="summary-value">{ncrStats?.open ?? 0}</span>
				<span class="summary-label">Open NCRs</span>
			</div>
			<div class="summary-card capa">
				<div class="summary-icon">
					<ShieldCheck size={22} />
				</div>
				<span class="summary-value">{capaStats?.open ?? 0}</span>
				<span class="summary-label">Open CAPAs</span>
			</div>
			<div class="summary-card critical">
				<div class="summary-icon">
					<AlertTriangle size={22} />
				</div>
				<span class="summary-value">{ncrStats?.by_severity?.critical ?? 0}</span>
				<span class="summary-label">Critical NCRs</span>
			</div>
			<div class="summary-card overdue">
				<div class="summary-icon">
					<Clock size={22} />
				</div>
				<span class="summary-value">{capaStats?.overdue ?? 0}</span>
				<span class="summary-label">Overdue CAPAs</span>
			</div>
		</div>

		<!-- Navigation links -->
		<div class="nav-links">
			<a href="/quality/ncrs" class="nav-link">
				<div class="nav-link-left">
					<div class="nav-link-icon ncr-bg">
						<ShieldAlert size={20} />
					</div>
					<div class="nav-link-text">
						<span class="nav-link-title">Non-Conformance Reports</span>
						<span class="nav-link-count">{ncrStats?.total ?? 0} total</span>
					</div>
				</div>
				<ChevronRight size={18} class="nav-chevron" />
			</a>

			<a href="/quality/capas" class="nav-link">
				<div class="nav-link-left">
					<div class="nav-link-icon capa-bg">
						<ShieldCheck size={20} />
					</div>
					<div class="nav-link-text">
						<span class="nav-link-title">CAPAs</span>
						<span class="nav-link-count">{capaStats?.total ?? 0} total</span>
					</div>
				</div>
				<ChevronRight size={18} class="nav-chevron" />
			</a>
		</div>

		<!-- NCR by severity breakdown -->
		{#if ncrStats}
			<section class="breakdown-section">
				<h3 class="breakdown-title">NCR Severity</h3>
				<div class="breakdown-bars">
					{#each [
						{ label: 'Minor', value: ncrStats.by_severity.minor, color: 'var(--theme-warning)' },
						{ label: 'Major', value: ncrStats.by_severity.major, color: 'var(--theme-info)' },
						{ label: 'Critical', value: ncrStats.by_severity.critical, color: 'var(--theme-error)' }
					] as bar}
						<div class="bar-row">
							<span class="bar-label">{bar.label}</span>
							<div class="bar-track">
								<div
									class="bar-fill"
									style="width: {ncrStats.total > 0 ? (bar.value / ncrStats.total) * 100 : 0}%; background-color: {bar.color}"
								></div>
							</div>
							<span class="bar-value">{bar.value}</span>
						</div>
					{/each}
				</div>
			</section>
		{/if}
	{/if}
</div>

<style>
	.quality-page {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 20px;
		padding-bottom: 32px;
	}

	.summary-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 10px;
	}

	.summary-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 18px 12px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 16px;
	}

	.summary-icon {
		width: 40px;
		height: 40px;
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.summary-card.ncr .summary-icon { background-color: color-mix(in oklch, var(--theme-error) 15%, transparent); color: var(--theme-error); }
	.summary-card.capa .summary-icon { background-color: color-mix(in oklch, var(--theme-info) 15%, transparent); color: var(--theme-info); }
	.summary-card.critical .summary-icon { background-color: color-mix(in oklch, var(--theme-warning) 15%, transparent); color: var(--theme-warning); }
	.summary-card.overdue .summary-icon { background-color: color-mix(in oklch, var(--theme-destructive) 15%, transparent); color: var(--theme-destructive); }

	.summary-value { font-size: 28px; font-weight: 800; letter-spacing: -0.02em; line-height: 1; }
	.summary-label { font-size: 11px; font-weight: 500; color: var(--theme-muted-foreground); text-transform: uppercase; letter-spacing: 0.04em; }

	.nav-links { display: flex; flex-direction: column; gap: 8px; }

	.nav-link {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 14px 16px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 14px;
		text-decoration: none;
		color: inherit;
		transition: all 0.15s ease;
	}

	.nav-link:active { transform: scale(0.98); background-color: var(--theme-accent); }

	.nav-link-left { display: flex; align-items: center; gap: 12px; }

	.nav-link-icon {
		width: 40px;
		height: 40px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.ncr-bg { background-color: color-mix(in oklch, var(--theme-error) 15%, transparent); color: var(--theme-error); }
	.capa-bg { background-color: color-mix(in oklch, var(--theme-info) 15%, transparent); color: var(--theme-info); }

	.nav-link-text { display: flex; flex-direction: column; gap: 2px; }
	.nav-link-title { font-size: 15px; font-weight: 600; }
	.nav-link-count { font-size: 12px; color: var(--theme-muted-foreground); }
	:global(.nav-chevron) { color: var(--theme-muted-foreground); }

	.breakdown-section { display: flex; flex-direction: column; gap: 12px; }
	.breakdown-title { font-size: 15px; font-weight: 700; padding: 0 4px; }
	.breakdown-bars { display: flex; flex-direction: column; gap: 10px; padding: 16px; background-color: var(--theme-card); border: 1px solid var(--theme-border); border-radius: 14px; }

	.bar-row { display: flex; align-items: center; gap: 10px; }
	.bar-label { font-size: 13px; font-weight: 500; width: 56px; flex-shrink: 0; }
	.bar-track { flex: 1; height: 8px; background-color: var(--theme-muted); border-radius: 4px; overflow: hidden; }
	.bar-fill { height: 100%; border-radius: 4px; transition: width 0.5s ease; min-width: 2px; }
	.bar-value { font-size: 13px; font-weight: 700; width: 28px; text-align: right; font-variant-numeric: tabular-nums; }

	.loading-state { display: flex; justify-content: center; padding: 64px; }
	.loading-spinner { width: 32px; height: 32px; border: 3px solid var(--theme-border); border-top-color: var(--theme-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
	@keyframes spin { to { transform: rotate(360deg); } }
</style>
