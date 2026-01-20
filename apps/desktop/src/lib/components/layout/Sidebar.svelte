<script lang="ts">
	import { cn } from '$lib/utils/cn.js';
	import { page } from '$app/stores';
	import { entityCounts, isProjectOpen } from '$lib/stores/project.js';
	import {
		FileText,
		AlertTriangle,
		FlaskConical,
		ClipboardCheck,
		Box,
		Layers,
		Settings,
		Factory,
		Shield,
		Link2,
		Home,
		FolderOpen,
		Ruler,
		CircleDot,
		Plug,
		Crosshair,
		Zap,
		GitBranch,
		BarChart3,
		Grid3X3,
		ClipboardList,
		Package,
		AlertOctagon,
		FileSpreadsheet,
		Building2,
		ShoppingCart,
		CheckSquare,
		List
	} from 'lucide-svelte';

	interface NavItem {
		label: string;
		href: string;
		icon: typeof Home;
		count?: number;
		children?: NavItem[];
	}

	// Build navigation items based on entity counts
	$effect(() => {
		navItems = buildNavItems($entityCounts);
	});

	function buildNavItems(counts: typeof $entityCounts): NavItem[] {
		return [
			{
				label: 'Dashboard',
				href: '/',
				icon: Home
			},
			{
				label: 'Requirements',
				href: '/requirements',
				icon: FileText,
				count: counts?.requirements ?? 0,
				children: [
					{ label: 'All Requirements', href: '/requirements', icon: List, count: counts?.requirements ?? 0 },
					{ label: 'Verification Matrix', href: '/requirements/verification', icon: CheckSquare }
				]
			},
			{
				label: 'Safety',
				href: '/risks',
				icon: AlertTriangle,
				count: (counts?.risks ?? 0) + (counts?.hazards ?? 0),
				children: [
					{ label: 'Hazards', href: '/hazards', icon: Zap, count: counts?.hazards ?? 0 },
					{ label: 'Risks', href: '/risks', icon: AlertTriangle, count: counts?.risks ?? 0 },
					{ label: 'FMEA Worksheet', href: '/risks/fmea', icon: ClipboardCheck },
					{ label: 'Analytics', href: '/risks/analytics', icon: BarChart3 }
				]
			},
			{
				label: 'Verification',
				href: '/verification',
				icon: FlaskConical,
				count: (counts?.tests ?? 0) + (counts?.results ?? 0),
				children: [
					{ label: 'Tests', href: '/verification/tests', icon: FlaskConical, count: counts?.tests ?? 0 },
					{ label: 'Results', href: '/verification/results', icon: ClipboardCheck, count: counts?.results ?? 0 }
				]
			},
			{
				label: 'BOM',
				href: '/assemblies',
				icon: Layers,
				count: (counts?.components ?? 0) + (counts?.assemblies ?? 0),
				children: [
					{ label: 'Assemblies', href: '/assemblies', icon: Layers, count: counts?.assemblies ?? 0 },
					{ label: 'Components', href: '/components', icon: Box, count: counts?.components ?? 0 }
				]
			},
			{
				label: 'Tolerances',
				href: '/tolerances',
				icon: Ruler,
				count: (counts?.features ?? 0) + (counts?.mates ?? 0) + (counts?.stackups ?? 0),
				children: [
					{ label: 'Features', href: '/features', icon: CircleDot, count: counts?.features ?? 0 },
					{ label: 'Mates', href: '/mates', icon: Plug, count: counts?.mates ?? 0 },
					{ label: 'Stackups', href: '/tolerances', icon: Ruler, count: counts?.stackups ?? 0 }
				]
			},
			{
				label: 'Manufacturing',
				href: '/manufacturing',
				icon: Factory,
				count: (counts?.processes ?? 0) + (counts?.controls ?? 0) + (counts?.work_instructions ?? 0) + (counts?.lots ?? 0) + (counts?.deviations ?? 0),
				children: [
					{ label: 'Processes', href: '/manufacturing/processes', icon: Factory, count: counts?.processes ?? 0 },
					{ label: 'Controls', href: '/controls', icon: Crosshair, count: counts?.controls ?? 0 },
					{ label: 'Work Instructions', href: '/manufacturing/work-instructions', icon: ClipboardList, count: counts?.work_instructions ?? 0 },
					{ label: 'Lots', href: '/manufacturing/lots', icon: Package, count: counts?.lots ?? 0 },
					{ label: 'Deviations', href: '/manufacturing/deviations', icon: AlertTriangle, count: counts?.deviations ?? 0 }
				]
			},
			{
				label: 'Quality',
				href: '/quality',
				icon: Shield,
				count: (counts?.ncrs ?? 0) + (counts?.capas ?? 0),
				children: [
					{ label: 'NCRs', href: '/quality/ncrs', icon: AlertOctagon, count: counts?.ncrs ?? 0 },
					{ label: 'CAPAs', href: '/quality/capas', icon: Shield, count: counts?.capas ?? 0 }
				]
			},
			{
				label: 'Procurement',
				href: '/procurement',
				icon: ShoppingCart,
				count: (counts?.quotes ?? 0) + (counts?.suppliers ?? 0),
				children: [
					{ label: 'Quotes', href: '/procurement/quotes', icon: FileSpreadsheet, count: counts?.quotes ?? 0 },
					{ label: 'Suppliers', href: '/procurement/suppliers', icon: Building2, count: counts?.suppliers ?? 0 }
				]
			},
			{
				label: 'Traceability',
				href: '/traceability',
				icon: Link2,
				children: [
					{ label: 'Trace Explorer', href: '/traceability', icon: GitBranch },
					{ label: 'Coverage', href: '/traceability/coverage', icon: BarChart3 },
					{ label: 'Matrix (DSM)', href: '/traceability/matrix', icon: Grid3X3 }
				]
			}
		];
	}

	let navItems: NavItem[] = $state(buildNavItems($entityCounts));
	let expandedItems: Set<string> = $state(new Set());

	function toggleExpanded(label: string) {
		if (expandedItems.has(label)) {
			expandedItems.delete(label);
		} else {
			expandedItems.add(label);
		}
		expandedItems = new Set(expandedItems);
	}

	function isActive(href: string): boolean {
		return $page.url.pathname === href || $page.url.pathname.startsWith(href + '/');
	}
</script>

<aside class="flex h-full w-64 flex-col bg-sidebar border-r border-sidebar-border">
	<!-- Logo / Brand -->
	<div class="flex h-14 items-center border-b border-sidebar-border px-4">
		<div class="flex items-center gap-2">
			<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">
				<span class="text-sm font-bold text-primary-foreground">T</span>
			</div>
			<div class="flex flex-col">
				<span class="text-sm font-semibold text-sidebar-foreground">Tessera</span>
				<span class="text-xs text-muted-foreground">Engineering Artifacts</span>
			</div>
		</div>
	</div>

	<!-- Navigation -->
	<nav class="flex-1 overflow-y-auto p-3">
		{#if !$isProjectOpen}
			<div class="flex flex-col items-center justify-center py-8 text-center">
				<FolderOpen class="h-12 w-12 text-muted-foreground/50 mb-3" />
				<p class="text-sm text-muted-foreground">No project open</p>
				<p class="text-xs text-muted-foreground/70 mt-1">Open or create a project to get started</p>
			</div>
		{:else}
			<ul class="space-y-1">
				{#each navItems as item}
					<li>
						{#if item.children}
							<button
								onclick={() => toggleExpanded(item.label)}
								class={cn(
									'flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm font-medium transition-colors',
									isActive(item.href)
										? 'bg-sidebar-accent text-sidebar-accent-foreground'
										: 'text-sidebar-foreground hover:bg-sidebar-accent/50'
								)}
							>
								<div class="flex items-center gap-3">
									<item.icon class="h-4 w-4" />
									<span>{item.label}</span>
								</div>
								<div class="flex items-center gap-2">
									{#if item.count !== undefined && item.count > 0}
										<span class="text-xs text-muted-foreground">{item.count}</span>
									{/if}
									<svg
										class={cn('h-4 w-4 transition-transform', expandedItems.has(item.label) && 'rotate-90')}
										fill="none"
										stroke="currentColor"
										viewBox="0 0 24 24"
									>
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
									</svg>
								</div>
							</button>
							{#if expandedItems.has(item.label)}
								<ul class="ml-4 mt-1 space-y-1 border-l border-sidebar-border pl-3">
									{#each item.children as child}
										<li>
											<a
												href={child.href}
												class={cn(
													'flex items-center justify-between rounded-lg px-3 py-1.5 text-sm transition-colors',
													isActive(child.href)
														? 'bg-sidebar-accent text-sidebar-accent-foreground'
														: 'text-muted-foreground hover:bg-sidebar-accent/50 hover:text-sidebar-foreground'
												)}
											>
												<span>{child.label}</span>
												{#if child.count !== undefined && child.count > 0}
													<span class="text-xs">{child.count}</span>
												{/if}
											</a>
										</li>
									{/each}
								</ul>
							{/if}
						{:else}
							<a
								href={item.href}
								class={cn(
									'flex items-center justify-between rounded-lg px-3 py-2 text-sm font-medium transition-colors',
									isActive(item.href)
										? 'bg-sidebar-accent text-sidebar-accent-foreground'
										: 'text-sidebar-foreground hover:bg-sidebar-accent/50'
								)}
							>
								<div class="flex items-center gap-3">
									<item.icon class="h-4 w-4" />
									<span>{item.label}</span>
								</div>
								{#if item.count !== undefined && item.count > 0}
									<span class="text-xs text-muted-foreground">{item.count}</span>
								{/if}
							</a>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	</nav>

	<!-- Footer -->
	<div class="border-t border-sidebar-border p-3 space-y-1">
		{#if $isProjectOpen}
			<a
				href="/version-control"
				class={cn(
					'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
					isActive('/version-control')
						? 'bg-sidebar-accent text-sidebar-accent-foreground'
						: 'text-sidebar-foreground hover:bg-sidebar-accent/50'
				)}
			>
				<GitBranch class="h-4 w-4" />
				<span>Version Control</span>
			</a>
		{/if}
		<a
			href="/settings"
			class={cn(
				'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
				isActive('/settings')
					? 'bg-sidebar-accent text-sidebar-accent-foreground'
					: 'text-sidebar-foreground hover:bg-sidebar-accent/50'
			)}
		>
			<Settings class="h-4 w-4" />
			<span>Settings</span>
		</a>
	</div>
</aside>
