<script lang="ts">
	import { Input, Badge } from '$lib/components/ui';
	import { Search, X, Loader2, ChevronDown } from 'lucide-svelte';
	import { entities } from '$lib/api/tauri.js';

	interface EntitySearchResult {
		id: string;
		title: string;
		status: string;
		prefix: string;
	}

	interface Props {
		/** Entity types to search (e.g., ['CTRL', 'TEST']) */
		entityTypes: string[];
		/** Placeholder text for the search input */
		placeholder?: string;
		/** Currently selected entity ID */
		value?: string;
		/** Callback when an entity is selected */
		onSelect: (entity: EntitySearchResult) => void;
		/** Optional callback when selection is cleared */
		onClear?: () => void;
		/** Whether the picker is disabled */
		disabled?: boolean;
		/** Label to show above the picker */
		label?: string;
	}

	let {
		entityTypes,
		placeholder = 'Search entities...',
		value = '',
		onSelect,
		onClear,
		disabled = false,
		label
	}: Props = $props();

	let searchQuery = $state('');
	let results = $state<EntitySearchResult[]>([]);
	let loading = $state(false);
	let isOpen = $state(false);
	let selectedEntity = $state<EntitySearchResult | null>(null);
	let highlightedIndex = $state(-1);
	let inputElement = $state<HTMLInputElement | null>(null);
	let dropdownElement = $state<HTMLDivElement | null>(null);

	// Debounce timer
	let debounceTimer: ReturnType<typeof setTimeout>;

	async function searchEntities(query: string) {
		if (!query.trim() && !isOpen) {
			results = [];
			return;
		}

		loading = true;
		try {
			const searchResults = await entities.search({
				entity_types: entityTypes,
				search: query.trim() || null,
				limit: 20
			}) as EntitySearchResult[];
			results = searchResults;
		} catch (e) {
			console.error('Entity search failed:', e);
			results = [];
		} finally {
			loading = false;
		}
	}

	function handleInputChange(e: Event) {
		const target = e.target as HTMLInputElement;
		searchQuery = target.value;
		highlightedIndex = -1;

		// Debounce search
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			searchEntities(searchQuery);
		}, 200);
	}

	function handleFocus() {
		isOpen = true;
		// Load initial results if empty
		if (results.length === 0) {
			searchEntities(searchQuery);
		}
	}

	function handleBlur(e: FocusEvent) {
		// Delay closing to allow click on dropdown items
		setTimeout(() => {
			const relatedTarget = e.relatedTarget as HTMLElement;
			if (!dropdownElement?.contains(relatedTarget)) {
				isOpen = false;
			}
		}, 150);
	}

	function handleSelect(entity: EntitySearchResult) {
		selectedEntity = entity;
		searchQuery = '';
		isOpen = false;
		results = [];
		onSelect(entity);
	}

	function handleClear() {
		selectedEntity = null;
		searchQuery = '';
		results = [];
		onClear?.();
		inputElement?.focus();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!isOpen) {
			if (e.key === 'ArrowDown' || e.key === 'Enter') {
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
				isOpen = false;
				break;
		}
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'approved':
			case 'released':
				return 'bg-green-500/20 text-green-400';
			case 'review':
				return 'bg-yellow-500/20 text-yellow-400';
			case 'draft':
				return 'bg-blue-500/20 text-blue-400';
			case 'obsolete':
				return 'bg-red-500/20 text-red-400';
			default:
				return 'bg-muted text-muted-foreground';
		}
	}

	// Initialize with existing value if provided
	$effect(() => {
		if (value && !selectedEntity) {
			// Try to look up the entity by ID
			(entities.search({
				entity_types: entityTypes,
				search: value,
				limit: 1
			}) as Promise<EntitySearchResult[]>).then((results) => {
				if (results.length > 0 && results[0].id === value) {
					selectedEntity = results[0];
				}
			});
		}
	});
</script>

<div class="relative">
	{#if label}
		<label for="entity-picker-input" class="text-sm font-medium mb-2 block">{label}</label>
	{/if}

	{#if selectedEntity}
		<!-- Selected entity display -->
		<div
			class="flex items-center justify-between rounded-md border border-input bg-background px-3 py-2"
		>
			<div class="flex items-center gap-2 min-w-0">
				<Badge variant="outline" class="shrink-0 font-mono text-xs">
					{selectedEntity.prefix}
				</Badge>
				<span class="truncate text-sm">{selectedEntity.title}</span>
				<span class="text-xs text-muted-foreground font-mono truncate">
					{selectedEntity.id}
				</span>
			</div>
			<button
				type="button"
				class="ml-2 shrink-0 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none"
				onclick={handleClear}
				{disabled}
			>
				<X class="h-4 w-4" />
				<span class="sr-only">Clear selection</span>
			</button>
		</div>
	{:else}
		<!-- Search input -->
		<div class="relative">
			<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
			<input
				bind:this={inputElement}
				id="entity-picker-input"
				type="text"
				class="flex h-10 w-full rounded-md border border-input bg-background pl-9 pr-10 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
				{placeholder}
				value={searchQuery}
				oninput={handleInputChange}
				onfocus={handleFocus}
				onblur={handleBlur}
				onkeydown={handleKeydown}
				{disabled}
			/>
			<div class="absolute right-3 top-1/2 -translate-y-1/2">
				{#if loading}
					<Loader2 class="h-4 w-4 animate-spin text-muted-foreground" />
				{:else}
					<ChevronDown class="h-4 w-4 text-muted-foreground" />
				{/if}
			</div>
		</div>

		<!-- Dropdown -->
		{#if isOpen}
			<div
				bind:this={dropdownElement}
				class="absolute z-50 mt-1 w-full rounded-md border bg-popover shadow-md"
			>
				{#if results.length === 0}
					<div class="px-3 py-6 text-center text-sm text-muted-foreground">
						{#if loading}
							Searching...
						{:else if searchQuery}
							No entities found matching "{searchQuery}"
						{:else}
							Type to search or browse {entityTypes.join(', ')} entities
						{/if}
					</div>
				{:else}
					<ul class="max-h-60 overflow-auto py-1">
						{#each results as entity, i (entity.id)}
							<li>
								<button
									type="button"
									class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition-colors hover:bg-accent {highlightedIndex ===
									i
										? 'bg-accent'
										: ''}"
									onclick={() => handleSelect(entity)}
									onmouseenter={() => (highlightedIndex = i)}
								>
									<Badge variant="outline" class="shrink-0 font-mono text-xs">
										{entity.prefix}
									</Badge>
									<div class="min-w-0 flex-1">
										<div class="truncate font-medium">{entity.title}</div>
										<div class="truncate text-xs text-muted-foreground font-mono">
											{entity.id}
										</div>
									</div>
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
	{/if}
</div>
