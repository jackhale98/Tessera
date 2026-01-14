<script lang="ts">
	import { Card, CardContent } from '$lib/components/ui';
	import type { RiskStats } from '$lib/api/tauri';
	import { AlertTriangle, Shield, TrendingUp, AlertCircle } from 'lucide-svelte';

	interface Props {
		stats: RiskStats;
	}

	let { stats }: Props = $props();
</script>

<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
	<!-- Total Risks -->
	<Card>
		<CardContent class="pt-6">
			<div class="flex items-center justify-between">
				<div>
					<p class="text-sm text-muted-foreground">Total Risks</p>
					<p class="text-3xl font-bold">{stats.total}</p>
				</div>
				<div class="flex h-12 w-12 items-center justify-center rounded-full bg-blue-100 dark:bg-blue-900/30">
					<AlertTriangle class="h-6 w-6 text-blue-600 dark:text-blue-400" />
				</div>
			</div>
		</CardContent>
	</Card>

	<!-- High Priority -->
	<Card>
		<CardContent class="pt-6">
			<div class="flex items-center justify-between">
				<div>
					<p class="text-sm text-muted-foreground">High Priority</p>
					<p class="text-3xl font-bold text-red-500">{stats.high_priority_count}</p>
				</div>
				<div class="flex h-12 w-12 items-center justify-center rounded-full bg-red-100 dark:bg-red-900/30">
					<AlertCircle class="h-6 w-6 text-red-600 dark:text-red-400" />
				</div>
			</div>
		</CardContent>
	</Card>

	<!-- Unmitigated -->
	<Card>
		<CardContent class="pt-6">
			<div class="flex items-center justify-between">
				<div>
					<p class="text-sm text-muted-foreground">Unmitigated</p>
					<p class="text-3xl font-bold text-orange-500">{stats.unmitigated_count}</p>
				</div>
				<div class="flex h-12 w-12 items-center justify-center rounded-full bg-orange-100 dark:bg-orange-900/30">
					<Shield class="h-6 w-6 text-orange-600 dark:text-orange-400" />
				</div>
			</div>
		</CardContent>
	</Card>

	<!-- Average RPN -->
	<Card>
		<CardContent class="pt-6">
			<div class="flex items-center justify-between">
				<div>
					<p class="text-sm text-muted-foreground">Avg RPN</p>
					<p class="text-3xl font-bold">{stats.average_rpn?.toFixed(0) ?? '-'}</p>
				</div>
				<div class="flex h-12 w-12 items-center justify-center rounded-full bg-purple-100 dark:bg-purple-900/30">
					<TrendingUp class="h-6 w-6 text-purple-600 dark:text-purple-400" />
				</div>
			</div>
		</CardContent>
	</Card>
</div>

<!-- Distribution by Level -->
{#if Object.keys(stats.by_level).length > 0}
	<Card class="mt-4">
		<CardContent class="pt-6">
			<h3 class="mb-4 font-medium">Risk Level Distribution</h3>
			<div class="flex h-8 w-full overflow-hidden rounded-full">
				{#each Object.entries(stats.by_level) as [level, count]}
					{@const percentage = (count / stats.total) * 100}
					{@const colors: Record<string, string> = {
						low: 'bg-green-500',
						medium: 'bg-yellow-500',
						high: 'bg-orange-500',
						critical: 'bg-red-500'
					}}
					{#if percentage > 0}
						<div
							class="{colors[level] ?? 'bg-gray-500'} flex items-center justify-center text-xs font-medium text-white"
							style="width: {percentage}%"
							title="{level}: {count} ({percentage.toFixed(1)}%)"
						>
							{#if percentage > 10}
								{count}
							{/if}
						</div>
					{/if}
				{/each}
			</div>
			<div class="mt-3 flex flex-wrap gap-4 text-sm">
				{#each Object.entries(stats.by_level) as [level, count]}
					{@const colors: Record<string, string> = {
						low: 'bg-green-500',
						medium: 'bg-yellow-500',
						high: 'bg-orange-500',
						critical: 'bg-red-500'
					}}
					<div class="flex items-center gap-2">
						<div class="h-3 w-3 rounded-full {colors[level] ?? 'bg-gray-500'}"></div>
						<span class="capitalize">{level}: {count}</span>
					</div>
				{/each}
			</div>
		</CardContent>
	</Card>
{/if}
