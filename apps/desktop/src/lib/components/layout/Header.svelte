<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { projectName, projectPath, isProjectOpen, isLoading, totalEntities } from '$lib/stores/project.js';
	import { theme, type Theme } from '$lib/stores/theme.js';
	import { open } from '@tauri-apps/plugin-dialog';
	import { openProject, initProject, closeProject, refreshProject } from '$lib/stores/project.js';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { ALL_ENTITY_TYPES, getEntityRoute, getStatusColor } from '$lib/config/entities';
	import {
		FolderOpen,
		FolderPlus,
		X,
		RefreshCw,
		Search,
		Command,
		Sun,
		Moon,
		Monitor,
		Loader2
	} from 'lucide-svelte';

	interface EntitySearchResult {
		id: string;
		title: string;
		status: string;
		prefix: string;
	}

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
	let results = $state<EntitySearchResult[]>([]);
	let loading = $state(false);
	let isOpen = $state(false);
	let highlightedIndex = $state(-1);
	let inputElement = $state<HTMLInputElement | null>(null);
	let dropdownElement = $state<HTMLDivElement | null>(null);
	let debounceTimer: ReturnType<typeof setTimeout>;
	let dropdownStyle = $state('');

	function updateDropdownPosition() {
		if (!inputElement) return;
		const rect = inputElement.getBoundingClientRect();
		dropdownStyle = `position: fixed; top: ${rect.bottom + 4}px; left: ${rect.left}px; width: ${rect.width}px;`;
	}

	async function searchEntities(query: string) {
		if (!query.trim() && !isOpen) {
			results = [];
			return;
		}

		loading = true;
		try {
			const searchResults = await invoke<EntitySearchResult[]>('search_entities', {
				params: {
					entity_types: ALL_ENTITY_TYPES,
					search: query.trim() || null,
					limit: 20
				}
			});
			results = searchResults;
		} catch (e) {
			console.error('Entity search failed:', e);
			results = [];
		} finally {
			loading = false;
		}
	}

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		searchQuery = target.value;
		highlightedIndex = -1;

		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			searchEntities(searchQuery);
		}, 200);
	}

	function handleFocus() {
		isOpen = true;
		updateDropdownPosition();
		if (results.length === 0) {
			searchEntities(searchQuery);
		}
	}

	function handleBlur(e: FocusEvent) {
		setTimeout(() => {
			const relatedTarget = e.relatedTarget as HTMLElement;
			if (!dropdownElement?.contains(relatedTarget)) {
				isOpen = false;
			}
		}, 150);
	}

	function handleSelect(entity: EntitySearchResult) {
		const route = getEntityRoute(entity.prefix, entity.id);
		searchQuery = '';
		isOpen = false;
		results = [];
		highlightedIndex = -1;
		goto(route);
	}

	function handleSearchKeydown(e: KeyboardEvent) {
		if (!isOpen) {
			if (e.key === 'ArrowDown' || e.key === 'Enter') {
				e.preventDefault();
				isOpen = true;
				searchEntities(searchQuery);
			}
			return;
		}

		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				highlightedIndex = Math.min(highlightedIndex + 1, results.length - 1);
				break;
			case 'ArrowUp':
				e.preventDefault();
				highlightedIndex = Math.max(highlightedIndex - 1, 0);
				break;
			case 'Enter':
				e.preventDefault();
				if (highlightedIndex >= 0 && results[highlightedIndex]) {
					handleSelect(results[highlightedIndex]);
				}
				break;
			case 'Escape':
				e.preventDefault();
				isOpen = false;
				inputElement?.blur();
				break;
		}
	}

	function handleGlobalKeydown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key === 'k') {
			e.preventDefault();
			if ($isProjectOpen) {
				inputElement?.focus();
			}
		}
	}

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
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

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
	<div class="flex-1 max-w-md mx-4 relative">
		<div class="relative">
			<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
			<input
				bind:this={inputElement}
				type="text"
				placeholder="Search entities..."
				class="flex h-9 w-full rounded-md border border-input bg-muted/50 pl-9 pr-12 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
				value={searchQuery}
				oninput={handleInput}
				onfocus={handleFocus}
				onblur={handleBlur}
				onkeydown={handleSearchKeydown}
				disabled={!$isProjectOpen}
			/>
			<div class="absolute right-3 top-1/2 -translate-y-1/2">
				{#if loading}
					<Loader2 class="h-4 w-4 animate-spin text-muted-foreground" />
				{:else}
					<kbd class="pointer-events-none hidden h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium opacity-100 sm:flex">
						<Command class="h-3 w-3" />K
					</kbd>
				{/if}
			</div>
		</div>

		<!-- Dropdown -->
		{#if isOpen && $isProjectOpen}
			<div
				bind:this={dropdownElement}
				class="z-50 rounded-md border bg-popover shadow-md"
				style={dropdownStyle}
			>
				{#if results.length === 0}
					<div class="px-3 py-6 text-center text-sm text-muted-foreground">
						{#if loading}
							Searching...
						{:else if searchQuery}
							No entities found matching "{searchQuery}"
						{:else}
							Type to search across all entities
						{/if}
					</div>
				{:else}
					<ul class="max-h-72 overflow-auto py-1">
						{#each results as entity, i (entity.id)}
							<li>
								<button
									type="button"
									class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition-colors hover:bg-accent {highlightedIndex === i ? 'bg-accent' : ''}"
									onclick={() => handleSelect(entity)}
									onmouseenter={() => (highlightedIndex = i)}
								>
									<Badge variant="outline" class="shrink-0 font-mono text-xs">
										{entity.prefix}
									</Badge>
									<div class="min-w-0 flex-1">
										<div class="truncate font-medium">{entity.title}</div>
									</div>
									<span class="shrink-0 truncate text-xs text-muted-foreground font-mono max-w-[80px]">
										{entity.id.length > 12 ? entity.id.slice(-8) : entity.id}
									</span>
									<Badge class="shrink-0 text-xs {getStatusColor(entity.status)}">
										{entity.status}
									</Badge>
								</button>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		{/if}
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
