<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { projectName, projectPath, isProjectOpen, isLoading, totalEntities } from '$lib/stores/project.js';
	import { theme, type Theme } from '$lib/stores/theme.js';
	import { open } from '@tauri-apps/plugin-dialog';
	import { openProject, initProject, closeProject, refreshProject } from '$lib/stores/project.js';
	import {
		FolderOpen,
		FolderPlus,
		X,
		RefreshCw,
		Search,
		Command,
		Sun,
		Moon,
		Monitor
	} from 'lucide-svelte';

	function getThemeLabel(currentTheme: Theme): string {
		switch (currentTheme) {
			case 'light':
				return 'Light mode';
			case 'dark':
				return 'Dark mode';
			case 'system':
				return 'System theme';
		}
	}

	let searchQuery = $state('');

	async function handleOpenProject() {
		const selected = await open({
			directory: true,
			multiple: false,
			title: 'Open Tessera Project'
		});

		if (selected && typeof selected === 'string') {
			await openProject(selected);
		}
	}

	async function handleInitProject() {
		const selected = await open({
			directory: true,
			multiple: false,
			title: 'Select Directory for New Project'
		});

		if (selected && typeof selected === 'string') {
			await initProject(selected);
		}
	}

	async function handleCloseProject() {
		await closeProject();
	}

	async function handleRefresh() {
		await refreshProject();
	}

	function handleSearch(e: Event) {
		// TODO: Implement global search
		e.preventDefault();
		console.log('Search:', searchQuery);
	}
</script>

<header class="flex h-14 items-center justify-between border-b border-border bg-background px-4">
	<!-- Left section: Project info or actions -->
	<div class="flex items-center gap-4">
		{#if $isProjectOpen}
			<div class="flex items-center gap-3">
				<div class="flex flex-col">
					<span class="text-sm font-medium">{$projectName}</span>
					<span class="text-xs text-muted-foreground truncate max-w-[200px]" title={$projectPath}>
						{$projectPath}
					</span>
				</div>
				<div class="h-6 w-px bg-border"></div>
				<span class="text-xs text-muted-foreground">{$totalEntities} entities</span>
			</div>
		{:else}
			<span class="text-sm text-muted-foreground">No project open</span>
		{/if}
	</div>

	<!-- Center section: Search -->
	<div class="flex-1 max-w-md mx-4">
		<form onsubmit={handleSearch} class="relative">
			<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
			<Input
				type="search"
				placeholder="Search entities..."
				class="pl-9 pr-12 h-9 bg-muted/50"
				bind:value={searchQuery}
				disabled={!$isProjectOpen}
			/>
			<kbd class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 hidden h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium opacity-100 sm:flex">
				<Command class="h-3 w-3" />K
			</kbd>
		</form>
	</div>

	<!-- Right section: Actions -->
	<div class="flex items-center gap-2">
		<!-- Theme toggle -->
		<Button
			variant="ghost"
			size="icon"
			onclick={() => theme.toggle()}
			title={getThemeLabel($theme)}
		>
			{#if $theme === 'light'}
				<Sun class="h-4 w-4" />
			{:else if $theme === 'dark'}
				<Moon class="h-4 w-4" />
			{:else}
				<Monitor class="h-4 w-4" />
			{/if}
		</Button>

		{#if $isProjectOpen}
			<Button
				variant="ghost"
				size="icon"
				onclick={handleRefresh}
				disabled={$isLoading}
				title="Refresh project"
			>
				<RefreshCw class="h-4 w-4 {$isLoading ? 'animate-spin' : ''}" />
			</Button>
			<Button
				variant="ghost"
				size="icon"
				onclick={handleCloseProject}
				disabled={$isLoading}
				title="Close project"
			>
				<X class="h-4 w-4" />
			</Button>
		{:else}
			<Button
				variant="outline"
				size="sm"
				onclick={handleOpenProject}
				disabled={$isLoading}
			>
				<FolderOpen class="h-4 w-4 mr-2" />
				Open Project
			</Button>
			<Button
				variant="default"
				size="sm"
				onclick={handleInitProject}
				disabled={$isLoading}
			>
				<FolderPlus class="h-4 w-4 mr-2" />
				New Project
			</Button>
		{/if}
	</div>
</header>
