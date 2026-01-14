<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '$lib/components/ui';
	import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui';
	import { Input } from '$lib/components/ui';
	import { components } from '$lib/api';
	import { isProjectOpen } from '$lib/stores/project';
	import type { ComponentSummary, ComponentStats, BomCostSummary } from '$lib/api/tauri';

	let componentsData = $state<ComponentSummary[]>([]);
	let stats = $state<ComponentStats | null>(null);
	let costSummary = $state<BomCostSummary | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let searchQuery = $state('');

	const filteredComponents = $derived(() => {
		if (!searchQuery) return componentsData;
		const query = searchQuery.toLowerCase();
		return componentsData.filter(
			(c) =>
				c.title.toLowerCase().includes(query) ||
				c.id.toLowerCase().includes(query) ||
				c.part_number.toLowerCase().includes(query)
		);
	});

	async function loadData() {
		if (!$isProjectOpen) return;

		loading = true;
		error = null;

		try {
			const [componentsResult, statsResult, costResult] = await Promise.all([
				components.list(),
				components.getStats(),
				components.getBomCostSummary()
			]);

			componentsData = componentsResult.items;
			stats = statsResult;
			costSummary = costResult;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load components:', e);
		} finally {
			loading = false;
		}
	}

	function handleRowClick(component: ComponentSummary) {
		goto(`/components/${component.id}`);
	}

	function formatCurrency(value: number | null | undefined): string {
		if (value == null) return '-';
		return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(value);
	}

	function formatMass(value: number | null | undefined): string {
		if (value == null) return '-';
		if (value < 1) return `${(value * 1000).toFixed(0)} g`;
		return `${value.toFixed(2)} kg`;
	}

	onMount(() => {
		loadData();
	});

	$effect(() => {
		if ($isProjectOpen) {
			loadData();
		}
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Bill of Materials</h1>
			<p class="text-muted-foreground">Component and assembly management</p>
		</div>
		<Button onclick={() => goto('/components/new')}>New Component</Button>
	</div>

	<!-- Stats cards -->
	{#if stats && costSummary}
		<div class="grid gap-4 md:grid-cols-5">
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Total Components</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.total}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Make</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.make_count}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Buy</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{stats.buy_count}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Total Cost</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{formatCurrency(costSummary.total_cost)}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="pb-2">
					<CardTitle class="text-sm font-medium text-muted-foreground">Total Mass</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{formatMass(stats.total_mass)}</div>
				</CardContent>
			</Card>
		</div>
	{/if}

	<!-- Error display -->
	{#if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{/if}

	<!-- Components table -->
	{#if !$isProjectOpen}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Open a project to view components</p>
			</CardContent>
		</Card>
	{:else}
		<Card>
			<CardHeader>
				<div class="flex items-center justify-between">
					<CardTitle>Component List</CardTitle>
					<Input
						type="search"
						placeholder="Search components..."
						bind:value={searchQuery}
						class="max-w-sm"
					/>
				</div>
			</CardHeader>
			<CardContent>
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead class="w-40">Part Number</TableHead>
							<TableHead>Title</TableHead>
							<TableHead class="w-24">Category</TableHead>
							<TableHead class="w-20">Make/Buy</TableHead>
							<TableHead class="w-28 text-right">Unit Cost</TableHead>
							<TableHead class="w-24 text-right">Mass</TableHead>
							<TableHead class="w-24">Status</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#if loading}
							<TableRow>
								<TableCell colspan={7} class="h-24 text-center">
									<div class="flex items-center justify-center gap-2">
										<div class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
										Loading...
									</div>
								</TableCell>
							</TableRow>
						{:else if filteredComponents().length === 0}
							<TableRow>
								<TableCell colspan={7} class="h-24 text-center text-muted-foreground">
									No components found
								</TableCell>
							</TableRow>
						{:else}
							{#each filteredComponents() as component (component.id)}
								<TableRow class="cursor-pointer" onclick={() => handleRowClick(component)}>
									<TableCell class="font-mono text-xs">{component.part_number}</TableCell>
									<TableCell>{component.title}</TableCell>
									<TableCell class="capitalize">{component.category}</TableCell>
									<TableCell>
										<Badge variant={component.make_buy === 'make' ? 'default' : 'secondary'}>
											{component.make_buy}
										</Badge>
									</TableCell>
									<TableCell class="text-right">{formatCurrency(component.unit_cost)}</TableCell>
									<TableCell class="text-right">{formatMass(component.mass_kg)}</TableCell>
									<TableCell>
										<Badge variant="outline" class="capitalize">{component.status}</Badge>
									</TableCell>
								</TableRow>
							{/each}
						{/if}
					</TableBody>
				</Table>
			</CardContent>
		</Card>
	{/if}
</div>
