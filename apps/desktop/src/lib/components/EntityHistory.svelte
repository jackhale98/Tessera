<script lang="ts">
	import { versionControl, type WorkflowHistory, type WorkflowEvent, type GitCommitInfo } from '$lib/api';
	import { cn } from '$lib/utils/cn.js';
	import {
		History,
		GitCommit,
		User,
		Calendar,
		CheckCircle2,
		XCircle,
		Clock,
		Tag,
		ChevronDown,
		ChevronRight,
		FileText,
		Shield,
		AlertCircle,
		Loader2
	} from 'lucide-svelte';

	interface Props {
		entityId?: string;
		class?: string;
	}

	let { entityId, class: className }: Props = $props();

	let workflowHistory: WorkflowHistory | null = $state(null);
	let gitHistory: GitCommitInfo[] = $state([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let expandedCommits = $state<Set<string>>(new Set());
	let commitDiffs = $state<Map<string, string>>(new Map());
	let loadingDiffs = $state<Set<string>>(new Set());

	$effect(() => {
		if (entityId) {
			loadHistory();
		}
	});

	async function loadHistory() {
		if (!entityId) {
			loading = false;
			return;
		}

		loading = true;
		error = null;

		try {
			const [workflow, commits] = await Promise.all([
				versionControl.getEntityWorkflowHistory(entityId).catch(() => null),
				versionControl.getEntityHistory(entityId, 20).catch(() => [])
			]);

			workflowHistory = workflow;
			gitHistory = commits;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load history';
		} finally {
			loading = false;
		}
	}

	async function toggleCommitDiff(hash: string) {
		if (expandedCommits.has(hash)) {
			expandedCommits.delete(hash);
			expandedCommits = new Set(expandedCommits);
		} else {
			expandedCommits.add(hash);
			expandedCommits = new Set(expandedCommits);

			// Load diff if not already loaded
			if (!commitDiffs.has(hash) && entityId) {
				loadingDiffs.add(hash);
				loadingDiffs = new Set(loadingDiffs);

				try {
					const diff = await versionControl.getEntityDiff(entityId, hash);
					commitDiffs.set(hash, diff);
					commitDiffs = new Map(commitDiffs);
				} catch {
					commitDiffs.set(hash, 'Failed to load diff');
					commitDiffs = new Map(commitDiffs);
				} finally {
					loadingDiffs.delete(hash);
					loadingDiffs = new Set(loadingDiffs);
				}
			}
		}
	}

	function getEventIcon(eventType: string) {
		switch (eventType) {
			case 'created':
				return FileText;
			case 'approved':
				return CheckCircle2;
			case 'released':
				return Tag;
			case 'rejected':
				return XCircle;
			default:
				return Clock;
		}
	}

	function getEventColor(eventType: string) {
		switch (eventType) {
			case 'created':
				return 'text-blue-500';
			case 'approved':
				return 'text-green-500';
			case 'released':
				return 'text-purple-500';
			case 'rejected':
				return 'text-red-500';
			default:
				return 'text-muted-foreground';
		}
	}

	function formatDate(dateStr: string): string {
		try {
			const date = new Date(dateStr);
			return date.toLocaleDateString(undefined, {
				year: 'numeric',
				month: 'short',
				day: 'numeric',
				hour: '2-digit',
				minute: '2-digit'
			});
		} catch {
			return dateStr;
		}
	}

	function formatShortDate(dateStr: string): string {
		try {
			const date = new Date(dateStr);
			return date.toLocaleDateString(undefined, {
				month: 'short',
				day: 'numeric'
			});
		} catch {
			return dateStr;
		}
	}
</script>

<div class={cn('space-y-6', className)}>
	{#if loading}
		<div class="flex items-center justify-center py-8">
			<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
		</div>
	{:else if error}
		<div class="flex items-center gap-2 text-destructive py-4">
			<AlertCircle class="h-4 w-4" />
			<span class="text-sm">{error}</span>
		</div>
	{:else}
		<!-- Workflow Timeline -->
		{#if workflowHistory && workflowHistory.events.length > 0}
			<div class="space-y-3">
				<div class="flex items-center gap-2">
					<History class="h-4 w-4 text-muted-foreground" />
					<h3 class="text-sm font-medium">Workflow Timeline</h3>
				</div>

				<div class="border rounded-lg p-4 bg-card">
					<!-- Current Status -->
					<div class="flex items-center gap-4 mb-4 pb-4 border-b">
						<div class="flex items-center gap-2">
							<span class="text-xs text-muted-foreground">Current:</span>
							<span class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium bg-primary/10 text-primary">
								{workflowHistory.current_status}
							</span>
						</div>
						{#if workflowHistory.revision}
							<div class="flex items-center gap-2">
								<span class="text-xs text-muted-foreground">Revision:</span>
								<span class="text-xs font-medium">{workflowHistory.revision}</span>
							</div>
						{/if}
					</div>

					<!-- Timeline -->
					<div class="space-y-4">
						{#each workflowHistory.events as event, i}
							{@const EventIcon = getEventIcon(event.event_type)}
							<div class="flex gap-3">
								<!-- Timeline line and dot -->
								<div class="flex flex-col items-center">
									<div class={cn('rounded-full p-1', getEventColor(event.event_type))}>
										<EventIcon class="h-3 w-3" />
									</div>
									{#if i < workflowHistory.events.length - 1}
										<div class="w-px flex-1 bg-border mt-1"></div>
									{/if}
								</div>

								<!-- Event content -->
								<div class="flex-1 pb-4">
									<div class="flex items-center gap-2 text-sm">
										<span class="font-medium capitalize">{event.event_type}</span>
										<span class="text-muted-foreground">by</span>
										<span class="font-medium">{event.actor}</span>
										{#if event.role}
											<span class="text-xs text-muted-foreground">({event.role})</span>
										{/if}
									</div>
									<div class="flex items-center gap-2 text-xs text-muted-foreground mt-0.5">
										<Calendar class="h-3 w-3" />
										<span>{formatDate(event.timestamp)}</span>
										{#if event.signature_verified}
											<div class="flex items-center gap-1 text-green-600">
												<Shield class="h-3 w-3" />
												<span>Verified</span>
											</div>
										{/if}
									</div>
									{#if event.comment}
										<p class="mt-1 text-sm text-muted-foreground italic">"{event.comment}"</p>
									{/if}
								</div>
							</div>
						{/each}
					</div>

					<!-- Tags -->
					{#if workflowHistory.tags.length > 0}
						<div class="flex items-center gap-2 mt-4 pt-4 border-t flex-wrap">
							<Tag class="h-3 w-3 text-muted-foreground" />
							{#each workflowHistory.tags as tag}
								<span class="inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium bg-secondary text-secondary-foreground">
									{tag}
								</span>
							{/each}
						</div>
					{/if}
				</div>
			</div>
		{/if}

		<!-- Git History -->
		{#if gitHistory.length > 0}
			<div class="space-y-3">
				<div class="flex items-center gap-2">
					<GitCommit class="h-4 w-4 text-muted-foreground" />
					<h3 class="text-sm font-medium">Git History</h3>
				</div>

				<div class="border rounded-lg divide-y bg-card">
					{#each gitHistory as commit}
						<div class="p-3">
							<!-- Commit header -->
							<button
								onclick={() => toggleCommitDiff(commit.hash)}
								class="flex items-start gap-3 w-full text-left hover:bg-accent/50 -m-3 p-3 rounded-lg transition-colors"
							>
								<div class="flex-shrink-0 mt-0.5">
									{#if expandedCommits.has(commit.hash)}
										<ChevronDown class="h-4 w-4 text-muted-foreground" />
									{:else}
										<ChevronRight class="h-4 w-4 text-muted-foreground" />
									{/if}
								</div>
								<div class="flex-1 min-w-0">
									<div class="flex items-center gap-2">
										<code class="text-xs font-mono text-primary">{commit.short_hash}</code>
										{#if commit.is_signed}
											<span title="Signed commit"><Shield class="h-3 w-3 text-green-600" /></span>
										{/if}
									</div>
									<p class="text-sm mt-0.5 truncate">{commit.message}</p>
									<div class="flex items-center gap-3 text-xs text-muted-foreground mt-1">
										<div class="flex items-center gap-1">
											<User class="h-3 w-3" />
											<span>{commit.author}</span>
										</div>
										<div class="flex items-center gap-1">
											<Calendar class="h-3 w-3" />
											<span>{formatShortDate(commit.date)}</span>
										</div>
									</div>
								</div>
							</button>

							<!-- Commit diff (expanded) -->
							{#if expandedCommits.has(commit.hash)}
								<div class="mt-3 ml-7">
									{#if loadingDiffs.has(commit.hash)}
										<div class="flex items-center gap-2 text-sm text-muted-foreground py-2">
											<Loader2 class="h-4 w-4 animate-spin" />
											<span>Loading diff...</span>
										</div>
									{:else if commitDiffs.has(commit.hash)}
										{@const diff = commitDiffs.get(commit.hash)}
										{#if diff && diff !== 'No changes found'}
											<pre class="text-xs font-mono bg-muted p-3 rounded-md overflow-x-auto max-h-64 overflow-y-auto">{diff}</pre>
										{:else}
											<p class="text-sm text-muted-foreground py-2">No changes in this commit for this entity.</p>
										{/if}
									{/if}
								</div>
							{/if}
						</div>
					{/each}
				</div>

				{#if gitHistory.length >= 20}
					<p class="text-xs text-muted-foreground text-center">
						Showing last 20 commits. <a href="/version-control" class="text-primary hover:underline">View all in Version Control</a>
					</p>
				{/if}
			</div>
		{/if}

		<!-- Empty state -->
		{#if (!workflowHistory || workflowHistory.events.length === 0) && gitHistory.length === 0}
			<div class="flex flex-col items-center justify-center py-8 text-center">
				<History class="h-12 w-12 text-muted-foreground/50 mb-3" />
				<p class="text-sm text-muted-foreground">No history available</p>
				<p class="text-xs text-muted-foreground/70 mt-1">This entity may not have been committed yet</p>
			</div>
		{/if}
	{/if}
</div>
