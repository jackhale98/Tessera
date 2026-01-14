<script lang="ts">
	import { goto } from '$app/navigation';
	import type { EntityPrefix } from '$lib/api/types';

	interface MatrixCell {
		row_id: string;
		col_id: string;
		link_types: string[];
	}

	interface DsmData {
		entity_ids: string[];
		entity_titles: Record<string, string>;
		entity_types: Record<string, string>;
		cells: MatrixCell[];
	}

	interface Props {
		data: DsmData;
		onSelectEntity?: (id: string) => void;
	}

	let { data, onSelectEntity }: Props = $props();

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

	function getEntityColor(type: string): string {
		const colors: Record<string, string> = {
			REQ: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
			RISK: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
			HAZ: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
			TEST: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
			RSLT: 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200',
			CMP: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
			ASM: 'bg-violet-100 text-violet-800 dark:bg-violet-900 dark:text-violet-200'
		};
		return colors[type] ?? 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200';
	}

	function handleEntityClick(id: string) {
		if (onSelectEntity) {
			onSelectEntity(id);
		} else {
			const prefix = id.split('-')[0] as EntityPrefix;
			const routes: Record<string, string> = {
				REQ: 'requirements',
				RISK: 'risks',
				HAZ: 'hazards',
				TEST: 'tests',
				CMP: 'components',
				ASM: 'assemblies'
			};
			goto(`/${routes[prefix] ?? 'entities'}/${id}`);
		}
	}

	function truncateId(id: string): string {
		const parts = id.split('-');
		if (parts.length === 2 && parts[1].length > 8) {
			return `${parts[0]}-${parts[1].slice(0, 4)}...`;
		}
		return id;
	}
</script>

{#if data.entity_ids.length === 0}
	<div class="flex h-64 items-center justify-center text-muted-foreground">
		No entities to display in matrix
	</div>
{:else}
	<div class="overflow-auto">
		<table class="min-w-full border-collapse text-xs">
			<thead>
				<tr>
					<th class="sticky left-0 z-10 border bg-background p-2"></th>
					{#each data.entity_ids as colId}
						<th class="border bg-muted/50 p-1">
							<button
								class="w-full rounded px-1 py-0.5 text-center hover:bg-muted {getEntityColor(data.entity_types[colId])}"
								onclick={() => handleEntityClick(colId)}
								title={data.entity_titles[colId]}
							>
								{truncateId(colId)}
							</button>
						</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each data.entity_ids as rowId}
					<tr>
						<th class="sticky left-0 z-10 border bg-muted/50 p-1 text-left">
							<button
								class="w-full rounded px-1 py-0.5 text-left hover:bg-muted {getEntityColor(data.entity_types[rowId])}"
								onclick={() => handleEntityClick(rowId)}
								title={data.entity_titles[rowId]}
							>
								{truncateId(rowId)}
							</button>
						</th>
						{#each data.entity_ids as colId}
							{@const cell = getCell(rowId, colId)}
							<td
								class="border p-1 text-center {rowId === colId ? 'bg-muted' : cell ? 'bg-primary/20 hover:bg-primary/30' : 'hover:bg-muted/30'}"
								title={cell ? cell.link_types.join(', ') : ''}
							>
								{#if rowId === colId}
									<span class="text-muted-foreground">-</span>
								{:else if cell}
									<span class="font-bold text-primary">{cell.link_types.length}</span>
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
		<div class="flex items-center gap-2">
			<div class="h-4 w-4 rounded bg-muted"></div>
			<span>Self (diagonal)</span>
		</div>
	</div>
{/if}
