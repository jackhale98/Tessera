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
		FolderOpen
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
				count: counts?.requirements ?? 0
			},
			{
				label: 'Risks',
				href: '/risks',
				icon: AlertTriangle,
				count: counts?.risks ?? 0
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
				href: '/components',
				icon: Box,
				count: (counts?.components ?? 0) + (counts?.assemblies ?? 0),
				children: [
					{ label: 'Components', href: '/components', icon: Box, count: counts?.components ?? 0 },
					{ label: 'Assemblies', href: '/assemblies', icon: Layers, count: counts?.assemblies ?? 0 }
				]
			},
			{
				label: 'Manufacturing',
				href: '/manufacturing',
				icon: Factory,
				count: (counts?.processes ?? 0) + (counts?.lots ?? 0)
			},
			{
				label: 'Quality',
				href: '/quality',
				icon: Shield,
				count: (counts?.ncrs ?? 0) + (counts?.capas ?? 0)
			},
			{
				label: 'Traceability',
				href: '/traceability',
				icon: Link2
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
				<span class="text-sm font-semibold text-sidebar-foreground">TDT Desktop</span>
				<span class="text-xs text-muted-foreground">Design Toolkit</span>
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
	<div class="border-t border-sidebar-border p-3">
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
