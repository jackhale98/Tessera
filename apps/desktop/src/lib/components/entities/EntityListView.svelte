<script lang="ts">
	import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui';
	import { Badge } from '$lib/components/ui';
	import { Input } from '$lib/components/ui';
	import { cn } from '$lib/utils/cn';
	import { Search } from 'lucide-svelte';
	import FilterPanel from './FilterPanel.svelte';
	import type { EntityData, FilterState, EntityFilterConfig } from '$lib/api/types';

	interface Column {
		key: string;
		label: string;
		sortable?: boolean;
		render?: (value: unknown, entity: EntityData) => string;
		class?: string;
	}

	interface Props {
		entities: EntityData[];
		columns: Column[];
		loading?: boolean;
		searchPlaceholder?: string;
		onRowClick?: (entity: EntityData) => void;
		class?: string;
		// Filter integration
		filterConfig?: EntityFilterConfig;
		onFiltersChange?: (filters: FilterState) => void;
		initialFilters?: FilterState;
		// Whether to show the search bar (can be disabled if search is in filters)
		showSearch?: boolean;
	}

	let {
		entities,
		columns,
		loading = false,
		searchPlaceholder = 'Search...',
		onRowClick,
		class: className,
		filterConfig,
		onFiltersChange,
		initialFilters = {},
		showSearch = true
	}: Props = $props();

	let searchQuery = $state('');
	let sortColumn = $state<string | null>(null);
	let sortDirection = $state<'asc' | 'desc'>('asc');

	// Filter entities by search query
	const filteredEntities = $derived(() => {
		if (!searchQuery) return entities;
		const query = searchQuery.toLowerCase();
		return entities.filter(
			(e) =>
				e.title.toLowerCase().includes(query) ||
				e.id.toLowerCase().includes(query) ||
				e.tags.some((t) => t.toLowerCase().includes(query))
		);
	});

	// Sort entities
	const sortedEntities = $derived(() => {
		const filtered = filteredEntities();
		if (!sortColumn) return filtered;

		return [...filtered].sort((a, b) => {
			const aVal = getNestedValue(a, sortColumn!);
			const bVal = getNestedValue(b, sortColumn!);

			if (aVal === bVal) return 0;
			if (aVal === null || aVal === undefined) return 1;
			if (bVal === null || bVal === undefined) return -1;

			const comparison = String(aVal).localeCompare(String(bVal));
			return sortDirection === 'asc' ? comparison : -comparison;
		});
	});

	function getNestedValue(obj: EntityData, path: string): unknown {
		return path.split('.').reduce((current, key) => {
			if (current && typeof current === 'object') {
				return (current as Record<string, unknown>)[key];
			}
			return undefined;
		}, obj as unknown);
	}

	function handleSort(column: Column) {
		if (!column.sortable) return;

		if (sortColumn === column.key) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortColumn = column.key;
			sortDirection = 'asc';
		}
	}

	function handleRowClick(entity: EntityData) {
		if (onRowClick) {
			onRowClick(entity);
		}
	}

	function handleFiltersChange(filters: FilterState) {
		if (onFiltersChange) {
			onFiltersChange(filters);
		}
	}

	function getStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (status) {
			case 'approved':
			case 'released':
				return 'default';
			case 'review':
				return 'secondary';
			case 'obsolete':
				return 'destructive';
			default:
				return 'outline';
		}
	}

	function formatValue(column: Column, entity: EntityData): string {
		const value = getNestedValue(entity, column.key);

		if (column.render) {
			return column.render(value, entity);
		}

		if (value === null || value === undefined) {
			return '-';
		}

		if (Array.isArray(value)) {
			return value.join(', ');
		}

		return String(value);
	}
</script>

<div class={cn('space-y-4', className)}>
	<!-- Filter Panel -->
	{#if filterConfig}
		<FilterPanel
			fields={filterConfig.fields}
			quickFilters={filterConfig.quickFilters}
			onFiltersChange={handleFiltersChange}
			initialFilters={initialFilters}
			collapsible={true}
			defaultExpanded={false}
		/>
	{/if}

	<!-- Search and count bar -->
	<div class="flex items-center gap-4">
		{#if showSearch}
			<div class="relative max-w-sm flex-1">
				<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
				<Input
					type="search"
					placeholder={searchPlaceholder}
					bind:value={searchQuery}
					class="pl-9"
				/>
			</div>
		{/if}
		<div class="text-sm text-muted-foreground">
			{sortedEntities().length} of {entities.length} items
		</div>
	</div>

	<!-- Table -->
	<div class="rounded-md border">
		<Table>
			<TableHeader>
				<TableRow>
					{#each columns as column}
						<TableHead
							class={cn(column.sortable && 'cursor-pointer select-none hover:bg-muted/50', column.class)}
							onclick={() => handleSort(column)}
						>
							<div class="flex items-center gap-1">
								{column.label}
								{#if column.sortable && sortColumn === column.key}
									<span class="text-xs">
										{sortDirection === 'asc' ? '\u2191' : '\u2193'}
									</span>
								{/if}
							</div>
						</TableHead>
					{/each}
				</TableRow>
			</TableHeader>
			<TableBody>
				{#if loading}
					<TableRow>
						<TableCell colspan={columns.length} class="h-24 text-center">
							<div class="flex items-center justify-center gap-2">
								<div class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
								Loading...
							</div>
						</TableCell>
					</TableRow>
				{:else if sortedEntities().length === 0}
					<TableRow>
						<TableCell colspan={columns.length} class="h-24 text-center text-muted-foreground">
							No results found
						</TableCell>
					</TableRow>
				{:else}
					{#each sortedEntities() as entity (entity.id)}
						<TableRow
							class={cn(onRowClick && 'cursor-pointer')}
							onclick={() => handleRowClick(entity)}
						>
							{#each columns as column}
								<TableCell class={column.class}>
									{#if column.key === 'status'}
										<Badge variant={getStatusVariant(entity.status)}>
											{entity.status}
										</Badge>
									{:else if column.key === 'tags'}
										<div class="flex flex-wrap gap-1">
											{#each entity.tags.slice(0, 3) as tag}
												<Badge variant="outline" class="text-xs">
													{tag}
												</Badge>
											{/each}
											{#if entity.tags.length > 3}
												<Badge variant="outline" class="text-xs">
													+{entity.tags.length - 3}
												</Badge>
											{/if}
										</div>
									{:else}
										{formatValue(column, entity)}
									{/if}
								</TableCell>
							{/each}
						</TableRow>
					{/each}
				{/if}
			</TableBody>
		</Table>
	</div>
</div>
