<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Button } from '$lib/components/ui';
	import { FmeaTable, RiskStatsCards } from '$lib/components/risk';
	import { risks } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import type { RiskSummary, RiskStats } from '$lib/api/tauri';
	import { Download, Plus, RefreshCw } from 'lucide-svelte';

	let risksData = $state<RiskSummary[]>([]);
	let stats = $state<RiskStats | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function loadData() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			const [risksResult, statsResult] = await Promise.all([
				risks.list(),
				risks.getStats()
			]);

			risksData = risksResult.items;
			stats = statsResult;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load risks:', e);
		} finally {
			loading = false;
		}
	}

	function handleRowClick(risk: RiskSummary) {
		goto(`/risks/${risk.id}`);
	}

	function exportCsv() {
		const headers = ['ID', 'Title', 'Type', 'Failure Mode', 'Severity', 'Occurrence', 'Detection', 'RPN', 'Risk Level', 'Status', 'Mitigations'];
		const rows = risksData.map(r => [
			r.id,
			`"${r.title.replace(/"/g, '""')}"`,
			r.risk_type,
			`"${(r.failure_mode || '').replace(/"/g, '""')}"`,
			r.severity ?? '',
			r.occurrence ?? '',
			r.detection ?? '',
			r.rpn ?? '',
			r.risk_level ?? '',
			r.status,
			r.mitigation_count
		]);

		const csv = [headers.join(','), ...rows.map(r => r.join(','))].join('\n');
		const blob = new Blob([csv], { type: 'text/csv' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `fmea-worksheet-${new Date().toISOString().split('T')[0]}.csv`;
		a.click();
		URL.revokeObjectURL(url);
	}

	onMount(() => {
		loadData();
	});

	$effect(() => {
		if ($isProjectOpen) {
			loadData();
		}
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">FMEA Worksheet</h1>
			<p class="text-muted-foreground">Failure Mode and Effects Analysis</p>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" onclick={loadData} disabled={loading}>
				<RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
				Refresh
			</Button>
			<Button variant="outline" onclick={exportCsv} disabled={risksData.length === 0}>
				<Download class="mr-2 h-4 w-4" />
				Export CSV
			</Button>
			<Button onclick={() => goto('/risks/new')}>
				<Plus class="mr-2 h-4 w-4" />
				New Risk
			</Button>
		</div>
	</div>

	<!-- Stats -->
	{#if stats}
		<RiskStatsCards {stats} />
	{/if}

	<!-- Error display -->
	{#if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{/if}

	<!-- FMEA Table -->
	{#if !$isProjectOpen}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Open a project to view FMEA worksheet</p>
			</CardContent>
		</Card>
	{:else if loading}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<div class="flex items-center gap-2">
					<div class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
					Loading FMEA data...
				</div>
			</CardContent>
		</Card>
	{:else}
		<Card>
			<CardHeader>
				<CardTitle>FMEA Register ({risksData.length} risks)</CardTitle>
			</CardHeader>
			<CardContent>
				<FmeaTable risks={risksData} onRowClick={handleRowClick} />
			</CardContent>
		</Card>
	{/if}
</div>
