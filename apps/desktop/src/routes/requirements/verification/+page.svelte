<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '$lib/components/ui';
	import { requirements } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import type { VerificationMatrixResponse, VerificationMatrixRow, TestWithResults } from '$lib/api/tauri';
	import {
		Download,
		RefreshCw,
		CheckCircle,
		XCircle,
		AlertTriangle,
		Clock,
		FileText,
		FlaskConical,
		ClipboardList,
		ChevronDown,
		ChevronRight
	} from 'lucide-svelte';

	let matrixData = $state<VerificationMatrixResponse | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let expandedRows = $state<Set<string>>(new Set());

	// Filter state
	let statusFilter = $state<string>('all');
	let typeFilter = $state<string>('all');

	const filteredRows = $derived(() => {
		if (!matrixData) return [];

		let rows = matrixData.rows;

		// Filter by verification status
		if (statusFilter !== 'all') {
			rows = rows.filter(r => r.verification_status === statusFilter);
		}

		// Filter by requirement type
		if (typeFilter !== 'all') {
			rows = rows.filter(r => r.req_type === typeFilter);
		}

		return rows;
	});

	async function loadData() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			matrixData = await requirements.getVerificationMatrix();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load verification matrix:', e);
		} finally {
			loading = false;
		}
	}

	function toggleRow(id: string) {
		const newExpanded = new Set(expandedRows);
		if (newExpanded.has(id)) {
			newExpanded.delete(id);
		} else {
			newExpanded.add(id);
		}
		expandedRows = newExpanded;
	}

	function handleEntityClick(e: Event, id: string) {
		e.stopPropagation();
		if (!id) return;

		const prefix = id.split('-')[0]?.toUpperCase();
		const routeMap: Record<string, string> = {
			REQ: '/requirements',
			TEST: '/verification/tests',
			RSLT: '/verification/results'
		};
		const route = routeMap[prefix] || '/entities';
		goto(`${route}/${id}`);
	}

	function getVerificationStatusBadge(status: string) {
		switch (status) {
			case 'verified':
				return { variant: 'default' as const, class: 'bg-green-500', icon: CheckCircle, label: 'Verified' };
			case 'partial':
				return { variant: 'secondary' as const, class: 'bg-yellow-500', icon: Clock, label: 'Partial' };
			case 'failed':
				return { variant: 'destructive' as const, class: '', icon: XCircle, label: 'Failed' };
			case 'not_tested':
			default:
				return { variant: 'outline' as const, class: '', icon: AlertTriangle, label: 'Not Tested' };
		}
	}

	function getVerdictBadge(verdict: string | undefined) {
		if (!verdict) return { variant: 'outline' as const, class: '', label: 'No Results' };
		switch (verdict) {
			case 'pass':
			case 'approved':
				return { variant: 'default' as const, class: 'bg-green-500', label: 'Pass' };
			case 'fail':
			case 'rejected':
				return { variant: 'destructive' as const, class: '', label: 'Fail' };
			case 'partial':
				return { variant: 'secondary' as const, class: 'bg-yellow-500', label: 'Partial' };
			default:
				return { variant: 'outline' as const, class: '', label: verdict };
		}
	}

	function exportCsv() {
		if (!matrixData) return;

		const headers = [
			'Requirement ID',
			'Requirement Title',
			'Type',
			'Level',
			'Priority',
			'Status',
			'Derived Requirements',
			'Tests',
			'Pass Count',
			'Fail Count',
			'Not Run',
			'Verification Status'
		];

		const rows = matrixData.rows.map(r => [
			r.requirement.id,
			`"${r.requirement.title.replace(/"/g, '""')}"`,
			r.req_type,
			r.level,
			r.priority,
			r.requirement.status,
			r.derived_requirements.map(d => d.id).join('; '),
			r.tests.map(t => `${t.id} (${t.latest_verdict || 'no results'})`).join('; '),
			r.pass_count,
			r.fail_count,
			r.not_run_count,
			r.verification_status
		]);

		const csv = [headers.join(','), ...rows.map(r => r.join(','))].join('\n');
		const blob = new Blob([csv], { type: 'text/csv' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `verification-matrix-${new Date().toISOString().split('T')[0]}.csv`;
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
			<h1 class="text-2xl font-bold">Requirements Verification Matrix</h1>
			<p class="text-muted-foreground">
				Full traceability from requirements through tests to results
			</p>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" onclick={loadData} disabled={loading}>
				<RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
				Refresh
			</Button>
			<Button variant="outline" onclick={exportCsv} disabled={!matrixData || matrixData.rows.length === 0}>
				<Download class="mr-2 h-4 w-4" />
				Export CSV
			</Button>
		</div>
	</div>

	<!-- Summary Cards -->
	{#if matrixData}
		<div class="grid gap-4 md:grid-cols-5">
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Total Requirements</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{matrixData.summary.total_requirements}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground flex items-center gap-2">
						<CheckCircle class="h-4 w-4 text-green-500" />
						Fully Verified
					</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-green-600">{matrixData.summary.fully_verified}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground flex items-center gap-2">
						<Clock class="h-4 w-4 text-yellow-500" />
						Partially Verified
					</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-yellow-600">{matrixData.summary.partially_verified}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground flex items-center gap-2">
						<AlertTriangle class="h-4 w-4 text-muted-foreground" />
						Not Tested
					</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-muted-foreground">{matrixData.summary.not_tested}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground flex items-center gap-2">
						<XCircle class="h-4 w-4 text-red-500" />
						Failed
					</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-red-600">{matrixData.summary.failed}</div>
				</CardContent>
			</Card>
		</div>

		<!-- Coverage Bar -->
		<Card>
			<CardContent class="pt-6">
				<div class="flex items-center justify-between mb-2">
					<span class="text-sm font-medium">Verification Coverage</span>
					<span class="text-sm font-bold">{matrixData.summary.verification_coverage.toFixed(1)}%</span>
				</div>
				<div class="h-3 w-full rounded-full bg-muted overflow-hidden">
					<div
						class="h-full bg-green-500 transition-all duration-500"
						style="width: {matrixData.summary.verification_coverage}%"
					></div>
				</div>
			</CardContent>
		</Card>
	{/if}

	<!-- Error display -->
	{#if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{/if}

	<!-- Filter Bar -->
	<div class="flex items-center gap-4">
		<div class="flex items-center gap-2">
			<label for="status-filter" class="text-sm font-medium">Status:</label>
			<select
				id="status-filter"
				bind:value={statusFilter}
				class="h-9 rounded-md border border-input bg-background px-3 py-1 text-sm"
			>
				<option value="all">All</option>
				<option value="verified">Verified</option>
				<option value="partial">Partial</option>
				<option value="not_tested">Not Tested</option>
				<option value="failed">Failed</option>
			</select>
		</div>
		<div class="flex items-center gap-2">
			<label for="type-filter" class="text-sm font-medium">Type:</label>
			<select
				id="type-filter"
				bind:value={typeFilter}
				class="h-9 rounded-md border border-input bg-background px-3 py-1 text-sm"
			>
				<option value="all">All</option>
				<option value="input">Input</option>
				<option value="output">Output</option>
			</select>
		</div>
		<div class="text-sm text-muted-foreground">
			{filteredRows().length} of {matrixData?.rows.length ?? 0} requirements
		</div>
	</div>

	<!-- Main content -->
	{#if !$isProjectOpen}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Open a project to view verification matrix</p>
			</CardContent>
		</Card>
	{:else if loading}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<div class="flex items-center gap-2">
					<div class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
					Loading verification matrix...
				</div>
			</CardContent>
		</Card>
	{:else if matrixData}
		<Card>
			<CardContent class="p-0">
				<div class="overflow-auto">
					<table class="w-full text-sm">
						<thead class="border-b bg-muted/50 sticky top-0">
							<tr>
								<th class="w-8 px-2 py-3"></th>
								<th class="whitespace-nowrap px-3 py-3 text-left font-medium">Requirement</th>
								<th class="whitespace-nowrap px-3 py-3 text-center font-medium w-20">Type</th>
								<th class="whitespace-nowrap px-3 py-3 text-center font-medium w-24">Level</th>
								<th class="whitespace-nowrap px-3 py-3 text-left font-medium">Derived Reqs</th>
								<th class="whitespace-nowrap px-3 py-3 text-left font-medium">Tests</th>
								<th class="whitespace-nowrap px-3 py-3 text-center font-medium w-32">Results</th>
								<th class="whitespace-nowrap px-3 py-3 text-center font-medium w-28">Status</th>
							</tr>
						</thead>
						<tbody class="divide-y">
							{#each filteredRows() as row (row.requirement.id)}
								{@const statusBadge = getVerificationStatusBadge(row.verification_status)}
								{@const isExpanded = expandedRows.has(row.requirement.id)}
								<tr
									class="cursor-pointer transition-colors hover:bg-muted/50"
									onclick={() => toggleRow(row.requirement.id)}
								>
									<!-- Expand Icon -->
									<td class="px-2 py-3 text-center">
										{#if row.tests.length > 0 || row.derived_requirements.length > 0}
											{#if isExpanded}
												<ChevronDown class="h-4 w-4 text-muted-foreground" />
											{:else}
												<ChevronRight class="h-4 w-4 text-muted-foreground" />
											{/if}
										{/if}
									</td>

									<!-- Requirement -->
									<td class="px-3 py-3">
										<div class="flex items-center gap-2">
											<FileText class="h-4 w-4 shrink-0 text-blue-500" />
											<div>
												<button
													class="font-medium text-left hover:text-primary hover:underline"
													onclick={(e) => handleEntityClick(e, row.requirement.id)}
												>
													{row.requirement.title}
												</button>
												<p class="font-mono text-xs text-muted-foreground">
													{row.requirement.id}
												</p>
											</div>
										</div>
									</td>

									<!-- Type -->
									<td class="px-3 py-3 text-center">
										<Badge variant="outline" class="capitalize text-xs">
											{row.req_type}
										</Badge>
									</td>

									<!-- Level -->
									<td class="px-3 py-3 text-center capitalize text-xs">
										{row.level}
									</td>

									<!-- Derived Requirements -->
									<td class="px-3 py-3">
										{#if row.derived_requirements.length > 0}
											<span class="text-xs">{row.derived_requirements.length} derived</span>
										{:else}
											<span class="text-xs text-muted-foreground">-</span>
										{/if}
									</td>

									<!-- Tests -->
									<td class="px-3 py-3">
										{#if row.tests.length > 0}
											<div class="flex items-center gap-2">
												<FlaskConical class="h-4 w-4 text-purple-500" />
												<span class="text-xs">{row.tests.length} test{row.tests.length > 1 ? 's' : ''}</span>
											</div>
										{:else}
											<span class="text-xs text-muted-foreground">No tests</span>
										{/if}
									</td>

									<!-- Results Summary -->
									<td class="px-3 py-3 text-center">
										{#if row.tests.length > 0}
											<div class="flex items-center justify-center gap-1 text-xs">
												{#if row.pass_count > 0}
													<span class="text-green-600">{row.pass_count}P</span>
												{/if}
												{#if row.fail_count > 0}
													<span class="text-red-600">{row.fail_count}F</span>
												{/if}
												{#if row.not_run_count > 0}
													<span class="text-muted-foreground">{row.not_run_count}NR</span>
												{/if}
											</div>
										{:else}
											<span class="text-xs text-muted-foreground">-</span>
										{/if}
									</td>

									<!-- Verification Status -->
									<td class="px-3 py-3 text-center">
										<Badge variant={statusBadge.variant} class="{statusBadge.class} text-xs">
											<svelte:component this={statusBadge.icon} class="h-3 w-3 mr-1" />
											{statusBadge.label}
										</Badge>
									</td>
								</tr>

								<!-- Expanded Details Row -->
								{#if isExpanded}
									<tr class="bg-muted/30">
										<td colspan="8" class="px-8 py-4">
											<div class="space-y-4">
												<!-- Derived Requirements -->
												{#if row.derived_requirements.length > 0}
													<div>
														<h4 class="text-xs font-semibold text-muted-foreground mb-2 flex items-center gap-2">
															<FileText class="h-3 w-3" />
															Derived Requirements ({row.derived_requirements.length})
														</h4>
														<div class="flex flex-wrap gap-2">
															{#each row.derived_requirements as derived}
																<button
																	class="text-xs px-2 py-1 rounded border hover:bg-muted/50 hover:text-primary"
																	onclick={(e) => handleEntityClick(e, derived.id)}
																>
																	{derived.title}
																	<Badge variant="outline" class="ml-1 text-xs capitalize">
																		{derived.status}
																	</Badge>
																</button>
															{/each}
														</div>
													</div>
												{/if}

												<!-- Tests with Results -->
												{#if row.tests.length > 0}
													<div>
														<h4 class="text-xs font-semibold text-muted-foreground mb-2 flex items-center gap-2">
															<FlaskConical class="h-3 w-3" />
															Tests & Results ({row.tests.length})
														</h4>
														<div class="space-y-2">
															{#each row.tests as test}
																{@const verdictBadge = getVerdictBadge(test.latest_verdict)}
																<div class="flex items-center justify-between rounded border p-2 bg-background">
																	<div class="flex items-center gap-2">
																		<button
																			class="text-sm font-medium hover:text-primary hover:underline"
																			onclick={(e) => handleEntityClick(e, test.id)}
																		>
																			{test.title}
																		</button>
																		<span class="text-xs text-muted-foreground font-mono">
																			{test.id}
																		</span>
																	</div>
																	<div class="flex items-center gap-2">
																		{#if test.results.length > 0}
																			<span class="text-xs text-muted-foreground">
																				{test.results.length} result{test.results.length > 1 ? 's' : ''}
																			</span>
																		{/if}
																		<Badge variant={verdictBadge.variant} class="{verdictBadge.class} text-xs">
																			{verdictBadge.label}
																		</Badge>
																	</div>
																</div>
																<!-- Show individual results if expanded -->
																{#if test.results.length > 0}
																	<div class="ml-6 space-y-1">
																		{#each test.results.slice(0, 3) as result}
																			{@const resultBadge = getVerdictBadge(result.verdict)}
																			<div class="flex items-center justify-between text-xs text-muted-foreground">
																				<button
																					class="hover:text-primary hover:underline font-mono"
																					onclick={(e) => handleEntityClick(e, result.id)}
																				>
																					{result.id}
																				</button>
																				<div class="flex items-center gap-2">
																					{#if result.executed_date}
																						<span>{new Date(result.executed_date).toLocaleDateString()}</span>
																					{/if}
																					<Badge variant={resultBadge.variant} class="{resultBadge.class} text-xs">
																						{resultBadge.label}
																					</Badge>
																				</div>
																			</div>
																		{/each}
																		{#if test.results.length > 3}
																			<p class="text-xs text-muted-foreground">
																				+{test.results.length - 3} more results
																			</p>
																		{/if}
																	</div>
																{/if}
															{/each}
														</div>
													</div>
												{:else}
													<p class="text-sm text-muted-foreground italic">
														No tests linked to this requirement. Consider adding verification tests.
													</p>
												{/if}
											</div>
										</td>
									</tr>
								{/if}
							{/each}
						</tbody>
					</table>
				</div>
			</CardContent>
		</Card>

		{#if filteredRows().length === 0}
			<div class="flex h-32 items-center justify-center text-muted-foreground">
				No requirements match the selected filters
			</div>
		{/if}
	{/if}
</div>

<!-- Legend -->
<div class="mt-4 rounded-lg bg-muted/50 p-3 text-xs text-muted-foreground">
	<strong>Legend:</strong>
	<CheckCircle class="inline h-3 w-3 text-green-500 ml-2" /> Verified (all tests pass) |
	<Clock class="inline h-3 w-3 text-yellow-500" /> Partial (some tests not run) |
	<XCircle class="inline h-3 w-3 text-red-500" /> Failed (at least one test failed) |
	<AlertTriangle class="inline h-3 w-3" /> Not Tested (no tests linked)
	<br />
	<strong class="mt-1 inline-block">Results:</strong>
	P = Pass | F = Fail | NR = Not Run
</div>
