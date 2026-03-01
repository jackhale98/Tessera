<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui';
	import { AlertTriangle } from 'lucide-svelte';
	import type { CoverageStats } from '$lib/api/types';

	interface Props {
		title: string;
		stats: CoverageStats;
		icon?: typeof import('lucide-svelte').Icon;
		colorClass?: string;
		/** Optional warning message shown when there are uncovered items */
		gapWarning?: string;
	}

	let { title, stats, icon: Icon, colorClass = 'text-primary', gapWarning }: Props = $props();

	const percentage = $derived(stats.percentage.toFixed(1));
	const progressColor = $derived(
		stats.percentage >= 80 ? 'bg-green-500' :
		stats.percentage >= 50 ? 'bg-yellow-500' :
		'bg-red-500'
	);
	const gap = $derived(stats.total - stats.covered);
</script>

<Card>
	<CardHeader class="pb-2">
		<CardTitle class="flex items-center gap-2 text-sm font-medium">
			{#if Icon}
				<Icon class="h-4 w-4 {colorClass}" />
			{/if}
			{title}
		</CardTitle>
	</CardHeader>
	<CardContent>
		<div class="space-y-2">
			<div class="flex items-baseline justify-between">
				<span class="text-2xl font-bold">{percentage}%</span>
				<span class="text-sm text-muted-foreground">
					{stats.covered} / {stats.total}
				</span>
			</div>
			<div class="h-2 w-full overflow-hidden rounded-full bg-muted">
				<div
					class="h-full transition-all duration-500 {progressColor}"
					style="width: {stats.percentage}%"
				></div>
			</div>
			{#if gapWarning && gap > 0}
				<div class="flex items-center gap-2 rounded-lg bg-yellow-500/10 p-2 text-xs text-yellow-600 dark:text-yellow-400">
					<AlertTriangle class="h-3.5 w-3.5 shrink-0" />
					{gap} {gapWarning}
				</div>
			{/if}
		</div>
	</CardContent>
</Card>
