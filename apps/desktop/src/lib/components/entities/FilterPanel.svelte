<script lang="ts">
	import { Button } from '$lib/components/ui';
	import { Input } from '$lib/components/ui';
	import { Select } from '$lib/components/ui';
	import { Badge } from '$lib/components/ui';
	import { Label } from '$lib/components/ui';
	import { cn } from '$lib/utils/cn';
	import { X, Filter, RotateCcw, ChevronDown, ChevronUp } from 'lucide-svelte';
	import type {
		FilterFieldDefinition,
		FilterState,
		FilterValue,
		QuickFilter,
		OnFiltersChange
	} from '$lib/api/types';

	interface Props {
		fields: FilterFieldDefinition[];
		quickFilters?: QuickFilter[];
		onFiltersChange: OnFiltersChange;
		class?: string;
		// Initial filter state
		initialFilters?: FilterState;
		// Show/hide the filter panel
		collapsible?: boolean;
		defaultExpanded?: boolean;
	}

	let {
		fields,
		quickFilters = [],
		onFiltersChange,
		class: className,
		initialFilters = {},
		collapsible = true,
		defaultExpanded = false
	}: Props = $props();

	// Internal filter state
	let filterState = $state<FilterState>({ ...initialFilters });
	let isExpanded = $state(defaultExpanded);

	// Count active filters
	const activeFilterCount = $derived(() => {
		return Object.entries(filterState).filter(([_, value]) => {
			if (value === null || value === undefined || value === '') return false;
			if (Array.isArray(value) && value.length === 0) return false;
			return true;
		}).length;
	});

	// Notify parent when filters change
	function notifyChange() {
		// Clean up empty values before sending
		const cleanFilters: FilterState = {};
		for (const [key, value] of Object.entries(filterState)) {
			if (value === null || value === undefined || value === '') continue;
			if (Array.isArray(value) && value.length === 0) continue;
			cleanFilters[key] = value;
		}
		onFiltersChange(cleanFilters);
	}

	// Update a filter value
	function updateFilter(key: string, value: FilterValue) {
		filterState = { ...filterState, [key]: value };
		notifyChange();
	}

	// Clear a single filter
	function clearFilter(key: string) {
		const field = fields.find((f) => f.key === key);
		filterState = { ...filterState, [key]: field?.defaultValue ?? null };
		notifyChange();
	}

	// Clear all filters
	function clearAllFilters() {
		const newState: FilterState = {};
		for (const field of fields) {
			newState[field.key] = field.defaultValue ?? null;
		}
		filterState = newState;
		onFiltersChange({});
	}

	// Apply a quick filter
	function applyQuickFilter(quickFilter: QuickFilter) {
		filterState = { ...filterState, ...quickFilter.filters };
		notifyChange();
	}

	// Toggle multi-select value
	function toggleMultiSelectValue(key: string, value: string) {
		const current = (filterState[key] as string[]) || [];
		const newValues = current.includes(value)
			? current.filter((v) => v !== value)
			: [...current, value];
		updateFilter(key, newValues);
	}

	// Check if a multi-select value is selected
	function isMultiSelectValueSelected(key: string, value: string): boolean {
		const current = filterState[key];
		if (!Array.isArray(current)) return false;
		return current.includes(value);
	}

	// Get the display value for a filter (for showing active filters)
	function getFilterDisplayValue(field: FilterFieldDefinition): string | null {
		const value = filterState[field.key];
		if (value === null || value === undefined || value === '') return null;

		if (field.type === 'select' && field.options) {
			const option = field.options.find((o) => o.value === value);
			return option?.label ?? String(value);
		}

		if (field.type === 'multi-select' && Array.isArray(value)) {
			if (value.length === 0) return null;
			if (value.length === 1) {
				const option = field.options?.find((o) => o.value === value[0]);
				return option?.label ?? value[0];
			}
			return `${value.length} selected`;
		}

		if (field.type === 'boolean') {
			return value ? (field.trueLabel ?? 'Yes') : null;
		}

		if (field.type === 'number-range') {
			const range = value as { min?: number; max?: number };
			if (range.min !== undefined && range.max !== undefined) {
				return `${range.min} - ${range.max}`;
			} else if (range.min !== undefined) {
				return `≥ ${range.min}`;
			} else if (range.max !== undefined) {
				return `≤ ${range.max}`;
			}
			return null;
		}

		return String(value);
	}
</script>

<div class={cn('space-y-3', className)}>
	<!-- Filter header with toggle and active count -->
	{#if collapsible}
		<div class="flex items-center justify-between">
			<Button
				variant="ghost"
				size="sm"
				class="gap-2"
				onclick={() => (isExpanded = !isExpanded)}
			>
				<Filter class="h-4 w-4" />
				Filters
				{#if activeFilterCount() > 0}
					<Badge variant="secondary" class="ml-1 h-5 px-1.5 text-xs">
						{activeFilterCount()}
					</Badge>
				{/if}
				{#if isExpanded}
					<ChevronUp class="h-4 w-4" />
				{:else}
					<ChevronDown class="h-4 w-4" />
				{/if}
			</Button>

			{#if activeFilterCount() > 0}
				<Button variant="ghost" size="sm" class="gap-1 text-muted-foreground" onclick={clearAllFilters}>
					<RotateCcw class="h-3 w-3" />
					Clear all
				</Button>
			{/if}
		</div>
	{/if}

	<!-- Active filters as chips (always visible when there are active filters) -->
	{#if activeFilterCount() > 0 && !isExpanded}
		<div class="flex flex-wrap gap-2">
			{#each fields as field}
				{@const displayValue = getFilterDisplayValue(field)}
				{#if displayValue}
					<Badge variant="secondary" class="gap-1 pr-1">
						<span class="text-muted-foreground">{field.label}:</span>
						{displayValue}
						<button
							class="ml-1 rounded-full p-0.5 hover:bg-muted"
							onclick={() => clearFilter(field.key)}
						>
							<X class="h-3 w-3" />
						</button>
					</Badge>
				{/if}
			{/each}
		</div>
	{/if}

	<!-- Expanded filter panel -->
	{#if isExpanded || !collapsible}
		<div class="rounded-lg border bg-muted/30 p-4">
			<!-- Quick filters -->
			{#if quickFilters.length > 0}
				<div class="mb-4">
					<Label class="mb-2 block text-xs text-muted-foreground">Quick Filters</Label>
					<div class="flex flex-wrap gap-2">
						{#each quickFilters as qf}
							<Button
								variant="outline"
								size="sm"
								onclick={() => applyQuickFilter(qf)}
							>
								{qf.label}
							</Button>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Filter fields grid -->
			<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				{#each fields as field}
					<div class="space-y-1.5">
						<Label for={field.key} class="text-xs font-medium">{field.label}</Label>

						{#if field.type === 'select'}
							<Select
								id={field.key}
								value={filterState[field.key] as string ?? ''}
								onchange={(e) => updateFilter(field.key, e.currentTarget.value || null)}
							>
								<option value="">{field.placeholder ?? 'All'}</option>
								{#each field.options ?? [] as option}
									<option value={option.value}>
										{option.label}
										{#if option.count !== undefined}
											({option.count})
										{/if}
									</option>
								{/each}
							</Select>

						{:else if field.type === 'multi-select'}
							<div class="flex flex-wrap gap-1.5 rounded-md border bg-background p-2">
								{#each field.options ?? [] as option}
									<button
										type="button"
										class={cn(
											'rounded-md px-2 py-1 text-xs transition-colors',
											isMultiSelectValueSelected(field.key, option.value)
												? 'bg-primary text-primary-foreground'
												: 'bg-muted hover:bg-muted/80'
										)}
										onclick={() => toggleMultiSelectValue(field.key, option.value)}
									>
										{option.label}
										{#if option.count !== undefined}
											<span class="ml-1 opacity-70">({option.count})</span>
										{/if}
									</button>
								{/each}
							</div>

						{:else if field.type === 'text'}
							<Input
								id={field.key}
								type="text"
								placeholder={field.placeholder}
								value={filterState[field.key] as string ?? ''}
								oninput={(e) => updateFilter(field.key, e.currentTarget.value || null)}
							/>

						{:else if field.type === 'number'}
							<Input
								id={field.key}
								type="number"
								placeholder={field.placeholder}
								min={field.min}
								max={field.max}
								step={field.step}
								value={filterState[field.key] as number ?? ''}
								oninput={(e) => updateFilter(field.key, e.currentTarget.valueAsNumber || null)}
							/>

						{:else if field.type === 'number-range'}
							{@const rangeValue = (filterState[field.key] as { min?: number; max?: number }) ?? {}}
							<div class="flex items-center gap-2">
								<Input
									type="number"
									placeholder="Min"
									min={field.min}
									max={field.max}
									step={field.step}
									value={rangeValue.min ?? ''}
									class="w-full"
									oninput={(e) =>
										updateFilter(field.key, {
											...rangeValue,
											min: e.currentTarget.valueAsNumber || undefined
										})
									}
								/>
								<span class="text-muted-foreground">-</span>
								<Input
									type="number"
									placeholder="Max"
									min={field.min}
									max={field.max}
									step={field.step}
									value={rangeValue.max ?? ''}
									class="w-full"
									oninput={(e) =>
										updateFilter(field.key, {
											...rangeValue,
											max: e.currentTarget.valueAsNumber || undefined
										})
									}
								/>
							</div>

						{:else if field.type === 'boolean'}
							<div class="flex items-center gap-2">
								<input
									id={field.key}
									type="checkbox"
									class="h-4 w-4 rounded border-input"
									checked={filterState[field.key] === true}
									onchange={(e) => updateFilter(field.key, e.currentTarget.checked || null)}
								/>
								<Label for={field.key} class="text-sm font-normal">
									{field.trueLabel ?? 'Yes'}
								</Label>
							</div>

						{:else if field.type === 'date'}
							<Input
								id={field.key}
								type="date"
								value={filterState[field.key] as string ?? ''}
								oninput={(e) => updateFilter(field.key, e.currentTarget.value || null)}
							/>

						{:else if field.type === 'date-range'}
							{@const dateValue = (filterState[field.key] as { start?: string; end?: string }) ?? {}}
							<div class="flex items-center gap-2">
								<Input
									type="date"
									value={dateValue.start ?? ''}
									class="w-full"
									oninput={(e) =>
										updateFilter(field.key, {
											...dateValue,
											start: e.currentTarget.value || undefined
										})
									}
								/>
								<span class="text-muted-foreground">-</span>
								<Input
									type="date"
									value={dateValue.end ?? ''}
									class="w-full"
									oninput={(e) =>
										updateFilter(field.key, {
											...dateValue,
											end: e.currentTarget.value || undefined
										})
									}
								/>
							</div>
						{/if}
					</div>
				{/each}
			</div>

			<!-- Clear all button in expanded view -->
			{#if activeFilterCount() > 0}
				<div class="mt-4 flex justify-end">
					<Button variant="outline" size="sm" class="gap-1" onclick={clearAllFilters}>
						<RotateCcw class="h-3 w-3" />
						Clear all filters
					</Button>
				</div>
			{/if}
		</div>
	{/if}
</div>
