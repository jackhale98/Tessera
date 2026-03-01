<script lang="ts">
	import { goto } from '$app/navigation';
	import { getEntityColorMuted, getEntityRoute, truncateEntityId } from '$lib/config/entities';
	import type { EntityPrefix } from '$lib/api/types';

	interface MatrixCell {
		row_id: string;
		col_id: string;
		link_types: string[];
	}

	// Support both square (DSM) and rectangular (DMM) matrices
	interface DsmData {
		entity_ids: string[]; // For square matrix (deprecated for DMM)
		row_entity_ids?: string[]; // For rectangular matrix
		col_entity_ids?: string[]; // For rectangular matrix
		entity_titles: Record<string, string>;
		entity_types: Record<string, string>;
		cells: MatrixCell[];
	}

	interface Props {
		data: DsmData;
		onSelectEntity?: (id: string) => void;
	}

	let { data, onSelectEntity }: Props = $props();

	// Get row and column IDs - support both square and rectangular matrices
	const rowIds = $derived(data.row_entity_ids ?? data.entity_ids);
	const colIds = $derived(data.col_entity_ids ?? data.entity_ids);

	// Determine if this is a rectangular matrix (DMM mode)
	const isRectangular = $derived(
		data.row_entity_ids !== undefined && data.col_entity_ids !== undefined
	);

	// Create a lookup for quick cell access
	const cellLookup = $derived(() => {
		const lookup = new Map<string, MatrixCell>();
		for (const cell of data.cells) {
			lookup.set(`${cell.row_id}:${cell.col_id}`, cell);
		}
		return lookup;
	});

	function getCell(rowId: string, colId: string): MatrixCell | undefined {
		return cellLookup().get(`${rowId}:${colId}`);
	}

	function handleEntityClick(id: string) {
		if (onSelectEntity) {
			onSelectEntity(id);
		} else {
			const prefix = id.split('-')[0] as EntityPrefix;
			goto(getEntityRoute(prefix, id));
		}
	}
</script>

{#if rowIds.length === 0 || colIds.length === 0}
	<div class="flex h-64 items-center justify-center text-muted-foreground">
		No entities to display in matrix
	</div>
{:else}
	<div class="overflow-auto">
		<table class="min-w-full border-collapse text-xs">
			<thead>
				<tr>
					<th class="sticky left-0 z-10 border bg-background p-2"></th>
					{#each colIds as colId}
						<th class="border bg-muted/50 p-1">
							<button
								class="w-full rounded px-1 py-0.5 text-center hover:bg-muted {getEntityColorMuted(data.entity_types[colId])}"
								onclick={() => handleEntityClick(colId)}
								title={data.entity_titles[colId]}
							>
								{truncateEntityId(colId)}
							</button>
						</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each rowIds as rowId}
					<tr>
						<th class="sticky left-0 z-10 border bg-muted/50 p-1 text-left">
							<button
								class="w-full rounded px-1 py-0.5 text-left hover:bg-muted {getEntityColorMuted(data.entity_types[rowId])}"
								onclick={() => handleEntityClick(rowId)}
								title={data.entity_titles[rowId]}
							>
								{truncateEntityId(rowId)}
							</button>
						</th>
						{#each colIds as colId}
							{@const cell = getCell(rowId, colId)}
							{@const isDiagonal = !isRectangular && rowId === colId}
							<td
								class="border p-1 text-center {isDiagonal ? 'bg-muted' : cell ? 'bg-primary/20 hover:bg-primary/30' : 'hover:bg-muted/30'}"
								title={cell ? cell.link_types.join(', ') : ''}
							>
								{#if isDiagonal}
									<span class="text-muted-foreground">-</span>
								{:else if cell}
									<span class="font-bold text-primary">X</span>
								{/if}
							</td>
						{/each}
					</tr>
				{/each}
			</tbody>
		</table>
	</div>

	<div class="mt-4 flex items-center gap-4 text-sm text-muted-foreground">
		<div class="flex items-center gap-2">
			<div class="h-4 w-4 rounded bg-primary/20"></div>
			<span>Has relationship</span>
		</div>
		{#if !isRectangular}
			<div class="flex items-center gap-2">
				<div class="h-4 w-4 rounded bg-muted"></div>
				<span>Self (diagonal)</span>
			</div>
		{/if}
	</div>
{/if}
