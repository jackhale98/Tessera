<script lang="ts">
	import { onMount } from 'svelte';
	import {
		Card,
		CardHeader,
		CardTitle,
		CardDescription,
		CardContent,
		CardFooter
	} from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import { isProjectOpen } from '$lib/stores/project.js';
	import { versionControl } from '$lib/api/index.js';
	import type {
		GitStatusInfo,
		UncommittedFile,
		VcGitUserInfo,
		GitCommitInfo,
		BranchInfo,
		TagInfo,
		CommitResult
	} from '$lib/api/index.js';
	import {
		GitBranch,
		GitCommit,
		Tag,
		RefreshCw,
		Check,
		X,
		Upload,
		Download,
		Plus,
		FileEdit,
		FilePlus,
		FileMinus,
		FileQuestion,
		Shield,
		AlertCircle,
		Clock,
		Code
	} from 'lucide-svelte';

	// State
	let loading = $state(true);
	let error = $state<string | null>(null);
	let successMessage = $state<string | null>(null);
	let activeTab = $state<'uncommitted' | 'commits' | 'branches' | 'tags'>('uncommitted');

	// Git data
	let gitStatus = $state<GitStatusInfo | null>(null);
	let gitUser = $state<VcGitUserInfo | null>(null);
	let recentCommits = $state<GitCommitInfo[]>([]);
	let branches = $state<BranchInfo[]>([]);
	let tags = $state<TagInfo[]>([]);

	// Commit form
	let selectedFiles = $state<Set<string>>(new Set());
	let commitMessage = $state('');
	let signCommit = $state(false);
	let committing = $state(false);
	let pushing = $state(false);
	let pulling = $state(false);

	// Branch form
	let newBranchName = $state('');
	let checkoutAfterCreate = $state(true);
	let creatingBranch = $state(false);

	// Tag filter
	let tagFilter = $state('');

	// Diff viewer
	let expandedDiffs = $state<Record<string, string>>({});
	let loadingDiffs = $state<Set<string>>(new Set());

	async function loadData() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			const [status, user, commits, branchList, tagList] = await Promise.all([
				versionControl.getStatus(),
				versionControl.getUser(),
				versionControl.getRecentCommits(50),
				versionControl.listBranches(),
				versionControl.listTags()
			]);

			gitStatus = status;
			gitUser = user;
			recentCommits = commits;
			branches = branchList;
			tags = tagList;

			// Select all uncommitted files by default
			if (status.uncommitted_files.length > 0) {
				selectedFiles = new Set(status.uncommitted_files.map((f) => f.path));
			}

			// Auto-enable signing if configured
			if (user?.signing_configured) {
				signCommit = true;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load version control data';
		} finally {
			loading = false;
		}
	}

	async function handleRefresh() {
		await loadData();
		showSuccess('Refreshed');
	}

	async function handleStageAndCommit() {
		if (!commitMessage.trim()) {
			error = 'Please enter a commit message';
			return;
		}

		if (selectedFiles.size === 0) {
			error = 'Please select files to commit';
			return;
		}

		committing = true;
		error = null;

		try {
			// Stage selected files
			await versionControl.stageFiles(Array.from(selectedFiles));

			// Commit
			const result = await versionControl.commit(commitMessage.trim(), signCommit);

			showSuccess(`Committed: ${result.hash.substring(0, 7)} (${result.files_changed} files)`);
			commitMessage = '';
			selectedFiles.clear();

			// Refresh data
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to commit';
		} finally {
			committing = false;
		}
	}

	async function handlePush() {
		pushing = true;
		error = null;

		try {
			const result = await versionControl.push(undefined, true);
			showSuccess(`Pushed to ${result.branch}`);
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to push';
		} finally {
			pushing = false;
		}
	}

	async function handlePull() {
		pulling = true;
		error = null;

		try {
			await versionControl.pull();
			showSuccess('Pulled latest changes');
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to pull';
		} finally {
			pulling = false;
		}
	}

	async function handleCheckoutBranch(branch: string) {
		try {
			await versionControl.checkoutBranch(branch);
			showSuccess(`Checked out ${branch}`);
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to checkout branch';
		}
	}

	async function handleCreateBranch() {
		if (!newBranchName.trim()) {
			error = 'Please enter a branch name';
			return;
		}

		creatingBranch = true;
		error = null;

		try {
			await versionControl.createBranch(newBranchName.trim(), checkoutAfterCreate);
			showSuccess(`Created branch ${newBranchName}`);
			newBranchName = '';
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create branch';
		} finally {
			creatingBranch = false;
		}
	}

	function toggleFileSelection(path: string) {
		const newSet = new Set(selectedFiles);
		if (newSet.has(path)) {
			newSet.delete(path);
		} else {
			newSet.add(path);
		}
		selectedFiles = newSet;
	}

	function selectAllFiles() {
		if (gitStatus?.uncommitted_files) {
			selectedFiles = new Set(gitStatus.uncommitted_files.map((f) => f.path));
		}
	}

	function deselectAllFiles() {
		selectedFiles = new Set();
	}

	async function toggleDiff(path: string) {
		if (expandedDiffs[path] !== undefined) {
			const { [path]: _, ...rest } = expandedDiffs;
			expandedDiffs = rest;
			return;
		}

		const newLoading = new Set(loadingDiffs);
		newLoading.add(path);
		loadingDiffs = newLoading;

		try {
			const diff = await versionControl.getUncommittedFileDiff(path);
			expandedDiffs = { ...expandedDiffs, [path]: diff };
		} catch (e) {
			console.error('Failed to load diff:', e);
		} finally {
			const newLoading2 = new Set(loadingDiffs);
			newLoading2.delete(path);
			loadingDiffs = newLoading2;
		}
	}

	function showSuccess(message: string) {
		successMessage = message;
		setTimeout(() => {
			successMessage = null;
		}, 3000);
	}

	function getStatusIcon(status: string) {
		switch (status) {
			case 'modified':
				return FileEdit;
			case 'added':
				return FilePlus;
			case 'deleted':
				return FileMinus;
			case 'untracked':
				return FileQuestion;
			default:
				return FileEdit;
		}
	}

	function getStatusColor(status: string) {
		switch (status) {
			case 'modified':
				return 'text-yellow-500';
			case 'added':
				return 'text-green-500';
			case 'deleted':
				return 'text-red-500';
			case 'untracked':
				return 'text-gray-500';
			default:
				return 'text-gray-500';
		}
	}

	function formatDate(dateStr: string) {
		try {
			const date = new Date(dateStr);
			return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
		} catch {
			return dateStr;
		}
	}

	$effect(() => {
		if ($isProjectOpen) {
			loadData();
		}
	});
</script>

<div class="container mx-auto p-6 max-w-6xl">
	<div class="flex items-center justify-between mb-6">
		<div>
			<h1 class="text-2xl font-bold flex items-center gap-2">
				<GitBranch class="h-6 w-6" />
				Version Control
			</h1>
			<p class="text-muted-foreground">Manage git operations and view history</p>
		</div>
		<Button variant="outline" onclick={handleRefresh} disabled={loading}>
			<RefreshCw class="h-4 w-4 mr-2 {loading ? 'animate-spin' : ''}" />
			Refresh
		</Button>
	</div>

	{#if error}
		<div class="bg-destructive/10 text-destructive p-4 rounded-md mb-4 flex items-center gap-2">
			<AlertCircle class="h-5 w-5" />
			{error}
		</div>
	{/if}

	{#if successMessage}
		<div class="bg-green-500/10 text-green-600 p-4 rounded-md mb-4 flex items-center gap-2">
			<Check class="h-5 w-5" />
			{successMessage}
		</div>
	{/if}

	{#if !$isProjectOpen}
		<Card>
			<CardContent class="py-8 text-center text-muted-foreground">
				Open a project to access version control features.
			</CardContent>
		</Card>
	{:else if loading}
		<Card>
			<CardContent class="py-8 text-center text-muted-foreground">
				Loading version control data...
			</CardContent>
		</Card>
	{:else if gitStatus && !gitStatus.is_repo}
		<Card>
			<CardContent class="py-8 text-center text-muted-foreground">
				This project is not a git repository.
			</CardContent>
		</Card>
	{:else}
		<!-- Repository Status -->
		<Card class="mb-6">
			<CardContent class="py-4">
				<div class="flex flex-wrap items-center gap-6">
					<div class="flex items-center gap-2">
						<GitBranch class="h-4 w-4 text-muted-foreground" />
						<span class="font-medium">Branch:</span>
						<Badge variant={gitStatus?.is_main_branch ? 'default' : 'secondary'}>
							{gitStatus?.current_branch || 'unknown'}
						</Badge>
					</div>

					<div class="flex items-center gap-2">
						<span class="font-medium">Status:</span>
						{#if gitStatus?.is_clean}
							<Badge variant="outline" class="text-green-600">
								<Check class="h-3 w-3 mr-1" />
								Clean
							</Badge>
						{:else}
							<Badge variant="outline" class="text-yellow-600">
								{gitStatus?.uncommitted_files.length || 0} uncommitted
							</Badge>
						{/if}
					</div>

					{#if gitUser?.name}
						<div class="flex items-center gap-2">
							<span class="font-medium">User:</span>
							<span class="text-sm">{gitUser.name}</span>
							{#if gitUser.email}
								<span class="text-sm text-muted-foreground">&lt;{gitUser.email}&gt;</span>
							{/if}
						</div>
					{/if}

					{#if gitUser?.signing_configured}
						<Badge variant="outline" class="text-green-600">
							<Shield class="h-3 w-3 mr-1" />
							GPG Signing
						</Badge>
					{/if}
				</div>
			</CardContent>
		</Card>

		<!-- Tabs -->
		<div class="flex gap-2 mb-4 border-b pb-2">
			<Button
				variant={activeTab === 'uncommitted' ? 'default' : 'ghost'}
				size="sm"
				onclick={() => (activeTab = 'uncommitted')}
			>
				Uncommitted
				{#if gitStatus && gitStatus.uncommitted_files.length > 0}
					<Badge variant="secondary" class="ml-2">{gitStatus.uncommitted_files.length}</Badge>
				{/if}
			</Button>
			<Button
				variant={activeTab === 'commits' ? 'default' : 'ghost'}
				size="sm"
				onclick={() => (activeTab = 'commits')}
			>
				Recent Commits
			</Button>
			<Button
				variant={activeTab === 'branches' ? 'default' : 'ghost'}
				size="sm"
				onclick={() => (activeTab = 'branches')}
			>
				Branches
				<Badge variant="secondary" class="ml-2">{branches.length}</Badge>
			</Button>
			<Button
				variant={activeTab === 'tags' ? 'default' : 'ghost'}
				size="sm"
				onclick={() => (activeTab = 'tags')}
			>
				Tags
				<Badge variant="secondary" class="ml-2">{tags.length}</Badge>
			</Button>
		</div>

		<!-- Tab Content -->
		{#if activeTab === 'uncommitted'}
			<Card>
				<CardHeader>
					<CardTitle class="text-lg">Uncommitted Changes</CardTitle>
					<CardDescription>
						Stage and commit changes to version control
					</CardDescription>
				</CardHeader>
				<CardContent>
					{#if gitStatus?.uncommitted_files.length === 0}
						<p class="text-muted-foreground text-center py-4">
							No uncommitted changes. Working directory is clean.
						</p>
					{:else}
						<!-- File list -->
						<div class="mb-4">
							<div class="flex items-center justify-between mb-2">
								<span class="text-sm font-medium">Files</span>
								<div class="flex gap-2">
									<Button variant="link" size="sm" onclick={selectAllFiles}>
										Select All
									</Button>
									<Button variant="link" size="sm" onclick={deselectAllFiles}>
										Deselect All
									</Button>
								</div>
							</div>
							<div class="border rounded-md max-h-96 overflow-y-auto">
								{#each gitStatus?.uncommitted_files || [] as file}
									{@const StatusIcon = getStatusIcon(file.status)}
									<div class="border-b last:border-b-0">
										<div class="flex items-center">
											<button
												type="button"
												class="flex-1 px-3 py-2 flex items-center gap-3 hover:bg-accent cursor-pointer text-left"
												onclick={() => toggleFileSelection(file.path)}
											>
												<input
													type="checkbox"
													checked={selectedFiles.has(file.path)}
													class="h-4 w-4"
													onclick={(e) => e.stopPropagation()}
													onchange={() => toggleFileSelection(file.path)}
												/>
												<StatusIcon class="h-4 w-4 {getStatusColor(file.status)}" />
												<div class="flex-1 min-w-0">
													<div class="truncate text-sm">{file.path}</div>
													{#if file.entity_id}
														<div class="text-xs text-muted-foreground truncate">
															{file.entity_id}
															{#if file.entity_title}
																: {file.entity_title}
															{/if}
														</div>
													{/if}
												</div>
												<Badge variant="outline" class="capitalize text-xs">
													{file.status}
												</Badge>
											</button>
											<button
												type="button"
												class="px-2 py-2 hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
												title="View diff"
												onclick={(e) => { e.stopPropagation(); toggleDiff(file.path); }}
											>
												{#if loadingDiffs.has(file.path)}
													<RefreshCw class="h-4 w-4 animate-spin" />
												{:else}
													<Code class="h-4 w-4" />
												{/if}
											</button>
										</div>
										{#if expandedDiffs[file.path] !== undefined}
											<div class="px-3 pb-3">
												<pre class="text-xs font-mono bg-muted rounded-md p-3 overflow-x-auto max-h-64 overflow-y-auto">{#each expandedDiffs[file.path].split('\n') as line}{#if line.startsWith('+') && !line.startsWith('+++')}<span class="text-green-500">{line}</span>{:else if line.startsWith('-') && !line.startsWith('---')}<span class="text-red-500">{line}</span>{:else if line.startsWith('@@')}<span class="text-blue-400">{line}</span>{:else}{line}{/if}
{/each}</pre>
											</div>
										{/if}
									</div>
								{/each}
							</div>
						</div>

						<!-- Commit form -->
						<Separator class="my-4" />

						<div class="space-y-4">
							<div>
								<Label for="commit-message">Commit Message</Label>
								<Input
									id="commit-message"
									bind:value={commitMessage}
									placeholder="Describe your changes..."
									class="mt-1"
								/>
							</div>

							{#if gitUser?.signing_configured}
								<label class="flex items-center gap-2 cursor-pointer">
									<input type="checkbox" bind:checked={signCommit} class="h-4 w-4" />
									<Shield class="h-4 w-4 text-green-600" />
									<span class="text-sm">Sign commit with GPG</span>
								</label>
							{/if}

							<div class="flex gap-2">
								<Button
									onclick={handleStageAndCommit}
									disabled={committing || selectedFiles.size === 0 || !commitMessage.trim()}
								>
									{#if committing}
										<RefreshCw class="h-4 w-4 mr-2 animate-spin" />
									{:else}
										<GitCommit class="h-4 w-4 mr-2" />
									{/if}
									Commit ({selectedFiles.size} files)
								</Button>

								<Button variant="outline" onclick={handlePush} disabled={pushing}>
									{#if pushing}
										<RefreshCw class="h-4 w-4 mr-2 animate-spin" />
									{:else}
										<Upload class="h-4 w-4 mr-2" />
									{/if}
									Push
								</Button>

								<Button variant="outline" onclick={handlePull} disabled={pulling}>
									{#if pulling}
										<RefreshCw class="h-4 w-4 mr-2 animate-spin" />
									{:else}
										<Download class="h-4 w-4 mr-2" />
									{/if}
									Pull
								</Button>
							</div>
						</div>
					{/if}
				</CardContent>
			</Card>
		{:else if activeTab === 'commits'}
			<Card>
				<CardHeader>
					<CardTitle class="text-lg">Recent Commits</CardTitle>
					<CardDescription>
						Last {recentCommits.length} commits in the repository
					</CardDescription>
				</CardHeader>
				<CardContent>
					{#if recentCommits.length === 0}
						<p class="text-muted-foreground text-center py-4">
							No commits yet.
						</p>
					{:else}
						<div class="space-y-2">
							{#each recentCommits as commit}
								<div class="border rounded-md p-3 hover:bg-accent/50">
									<div class="flex items-start justify-between gap-2">
										<div class="flex-1 min-w-0">
											<div class="font-medium truncate">{commit.message}</div>
											<div class="text-sm text-muted-foreground flex items-center gap-2 mt-1">
												<code class="text-xs bg-muted px-1.5 py-0.5 rounded">
													{commit.short_hash}
												</code>
												<span>{commit.author}</span>
												<span class="flex items-center gap-1">
													<Clock class="h-3 w-3" />
													{formatDate(commit.date)}
												</span>
											</div>
										</div>
										{#if commit.is_signed}
											<Badge variant="outline" class="text-green-600 shrink-0">
												<Shield class="h-3 w-3 mr-1" />
												Signed
											</Badge>
										{/if}
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</CardContent>
			</Card>
		{:else if activeTab === 'branches'}
			<Card>
				<CardHeader>
					<CardTitle class="text-lg">Branches</CardTitle>
					<CardDescription>
						Local and remote branches
					</CardDescription>
				</CardHeader>
				<CardContent>
					<!-- Create branch form -->
					<div class="mb-4 p-4 border rounded-md bg-muted/30">
						<div class="flex items-end gap-2">
							<div class="flex-1">
								<Label for="new-branch">Create New Branch</Label>
								<Input
									id="new-branch"
									bind:value={newBranchName}
									placeholder="feature/my-feature"
									class="mt-1"
								/>
							</div>
							<label class="flex items-center gap-2 cursor-pointer whitespace-nowrap px-2">
								<input type="checkbox" bind:checked={checkoutAfterCreate} class="h-4 w-4" />
								<span class="text-sm">Checkout</span>
							</label>
							<Button onclick={handleCreateBranch} disabled={creatingBranch || !newBranchName.trim()}>
								{#if creatingBranch}
									<RefreshCw class="h-4 w-4 mr-2 animate-spin" />
								{:else}
									<Plus class="h-4 w-4 mr-2" />
								{/if}
								Create
							</Button>
						</div>
					</div>

					<Separator class="my-4" />

					<!-- Branch list -->
					<div class="space-y-2">
						{#each branches.filter((b) => !b.is_remote) as branch}
							<div class="border rounded-md p-3 flex items-center justify-between hover:bg-accent/50">
								<div class="flex items-center gap-2">
									<GitBranch class="h-4 w-4 text-muted-foreground" />
									<span class="font-medium">{branch.name}</span>
									{#if branch.is_current}
										<Badge>Current</Badge>
									{/if}
								</div>
								<div class="flex items-center gap-2">
									{#if branch.last_message}
										<span class="text-sm text-muted-foreground truncate max-w-xs">
											{branch.last_message}
										</span>
									{/if}
									{#if !branch.is_current}
										<Button
											variant="outline"
											size="sm"
											onclick={() => handleCheckoutBranch(branch.name)}
										>
											Checkout
										</Button>
									{/if}
								</div>
							</div>
						{/each}

						{#if branches.filter((b) => b.is_remote).length > 0}
							<Separator class="my-4" />
							<h4 class="text-sm font-medium text-muted-foreground mb-2">Remote Branches</h4>
							{#each branches.filter((b) => b.is_remote) as branch}
								<div class="border rounded-md p-3 flex items-center justify-between hover:bg-accent/50">
									<div class="flex items-center gap-2">
										<GitBranch class="h-4 w-4 text-muted-foreground" />
										<span class="text-muted-foreground">{branch.name}</span>
									</div>
									<Button
										variant="outline"
										size="sm"
										onclick={() => handleCheckoutBranch(branch.name.replace(/^origin\//, ''))}
									>
										Checkout
									</Button>
								</div>
							{/each}
						{/if}
					</div>
				</CardContent>
			</Card>
		{:else if activeTab === 'tags'}
			<Card>
				<CardHeader>
					<CardTitle class="text-lg">Tags</CardTitle>
					<CardDescription>
						Git tags including approval and release tags
					</CardDescription>
				</CardHeader>
				<CardContent>
					<div class="mb-4">
						<Input
							bind:value={tagFilter}
							placeholder="Filter tags (e.g., approve/*, release/*)"
						/>
					</div>

					{#if tags.length === 0}
						<p class="text-muted-foreground text-center py-4">
							No tags in this repository.
						</p>
					{:else}
						<div class="space-y-2">
							{#each tags.filter((t) => !tagFilter || t.name.includes(tagFilter)) as tag}
								<div class="border rounded-md p-3 hover:bg-accent/50">
									<div class="flex items-start justify-between gap-2">
										<div class="flex items-center gap-2">
											<Tag class="h-4 w-4 text-muted-foreground" />
											<span class="font-medium">{tag.name}</span>
										</div>
										{#if tag.commit}
											<code class="text-xs bg-muted px-1.5 py-0.5 rounded">
												{tag.commit}
											</code>
										{/if}
									</div>
									{#if tag.message || tag.tagger}
										<div class="mt-1 text-sm text-muted-foreground">
											{#if tag.message}
												<span>{tag.message}</span>
											{/if}
											{#if tag.tagger}
												<span class="ml-2">by {tag.tagger}</span>
											{/if}
											{#if tag.date}
												<span class="ml-2">{formatDate(tag.date)}</span>
											{/if}
										</div>
									{/if}
								</div>
							{/each}
						</div>
					{/if}
				</CardContent>
			</Card>
		{/if}
	{/if}
</div>
