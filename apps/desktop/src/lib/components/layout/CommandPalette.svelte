<script lang="ts">
	import { goto } from '$app/navigation';
	import { Dialog, Input } from '$lib/components/ui';
	import { isProjectOpen } from '$lib/stores/project';
	import {
		Search,
		FileText,
		AlertTriangle,
		FlaskConical,
		Box,
		Layers,
		Factory,
		Shield,
		Link2,
		Home,
		Settings,
		Plus,
		Zap,
		BarChart3,
		ClipboardCheck,
		Ruler
	} from 'lucide-svelte';

	interface CommandItem {
		id: string;
		label: string;
		description?: string;
		icon: typeof Home;
		action: () => void;
		keywords?: string[];
		category: 'navigation' | 'action' | 'entity';
		shortcut?: string;
	}

	let open = $state(false);
	let searchQuery = $state('');
	let selectedIndex = $state(0);

	const commands: CommandItem[] = [
		// Navigation
		{ id: 'home', label: 'Go to Dashboard', icon: Home, action: () => goto('/'), category: 'navigation', keywords: ['home', 'dashboard'], shortcut: '⌘1' },
		{ id: 'requirements', label: 'Go to Requirements', icon: FileText, action: () => goto('/requirements'), category: 'navigation', shortcut: '⌘2' },
		{ id: 'risks', label: 'Go to Risks', icon: AlertTriangle, action: () => goto('/risks'), category: 'navigation', keywords: ['fmea', 'safety'], shortcut: '⌘3' },
		{ id: 'fmea', label: 'Go to FMEA Worksheet', icon: ClipboardCheck, action: () => goto('/risks/fmea'), category: 'navigation' },
		{ id: 'analytics', label: 'Go to Risk Analytics', icon: BarChart3, action: () => goto('/risks/analytics'), category: 'navigation' },
		{ id: 'hazards', label: 'Go to Hazards', icon: Zap, action: () => goto('/hazards'), category: 'navigation' },
		{ id: 'tests', label: 'Go to Tests', icon: FlaskConical, action: () => goto('/verification/tests'), category: 'navigation', keywords: ['verification'], shortcut: '⌘4' },
		{ id: 'results', label: 'Go to Test Results', icon: ClipboardCheck, action: () => goto('/verification/results'), category: 'navigation' },
		{ id: 'components', label: 'Go to Components', icon: Box, action: () => goto('/components'), category: 'navigation', keywords: ['bom'], shortcut: '⌘5' },
		{ id: 'assemblies', label: 'Go to Assemblies', icon: Layers, action: () => goto('/assemblies'), category: 'navigation', keywords: ['bom'] },
		{ id: 'tolerances', label: 'Go to Tolerances', icon: Ruler, action: () => goto('/tolerances'), category: 'navigation', keywords: ['stackup'] },
		{ id: 'processes', label: 'Go to Processes', icon: Factory, action: () => goto('/manufacturing/processes'), category: 'navigation', keywords: ['manufacturing'] },
		{ id: 'quality', label: 'Go to Quality', icon: Shield, action: () => goto('/quality'), category: 'navigation', keywords: ['ncr', 'capa'] },
		{ id: 'traceability', label: 'Go to Traceability', icon: Link2, action: () => goto('/traceability'), category: 'navigation', keywords: ['trace', 'coverage'], shortcut: '⌘6' },
		{ id: 'settings', label: 'Go to Settings', icon: Settings, action: () => goto('/settings'), category: 'navigation', keywords: ['preferences', 'config'] },

		// Actions
		{ id: 'new-req', label: 'Create Requirement', description: 'New requirement', icon: Plus, action: () => goto('/requirements/new'), category: 'action' },
		{ id: 'new-risk', label: 'Create Risk', description: 'New risk', icon: Plus, action: () => goto('/risks/new'), category: 'action' },
		{ id: 'new-test', label: 'Create Test', description: 'New test', icon: Plus, action: () => goto('/verification/tests/new'), category: 'action' },
		{ id: 'new-component', label: 'Create Component', description: 'New component', icon: Plus, action: () => goto('/components/new'), category: 'action' },
		{ id: 'new-assembly', label: 'Create Assembly', description: 'New assembly', icon: Plus, action: () => goto('/assemblies/new'), category: 'action' }
	];

	const filteredCommands = $derived(() => {
		if (!searchQuery) return commands;
		const query = searchQuery.toLowerCase();
		return commands.filter(cmd => {
			const matchLabel = cmd.label.toLowerCase().includes(query);
			const matchDesc = cmd.description?.toLowerCase().includes(query);
			const matchKeywords = cmd.keywords?.some(k => k.includes(query));
			return matchLabel || matchDesc || matchKeywords;
		});
	});

	// Quick navigation shortcuts
	const quickNavShortcuts: Record<string, string> = {
		'1': '/',              // Dashboard
		'2': '/requirements',  // Requirements
		'3': '/risks',         // Risks
		'4': '/verification/tests', // Tests
		'5': '/components',    // Components
		'6': '/traceability',  // Traceability
	};

	function handleKeydown(e: KeyboardEvent) {
		// Open with Cmd+K or Ctrl+K
		if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
			e.preventDefault();
			open = true;
			searchQuery = '';
			selectedIndex = 0;
			return;
		}

		// Quick navigation with Cmd+1 through Cmd+6
		if ((e.metaKey || e.ctrlKey) && e.key in quickNavShortcuts && !open) {
			e.preventDefault();
			goto(quickNavShortcuts[e.key]);
			return;
		}

		if (!open) return;

		const cmds = filteredCommands();

		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, cmds.length - 1);
				break;
			case 'ArrowUp':
				e.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				break;
			case 'Enter':
				e.preventDefault();
				if (cmds[selectedIndex]) {
					executeCommand(cmds[selectedIndex]);
				}
				break;
		}
	}

	function executeCommand(cmd: CommandItem) {
		open = false;
		searchQuery = '';
		cmd.action();
	}

	// Reset selection when filter changes
	$effect(() => {
		filteredCommands();
		selectedIndex = 0;
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<Dialog bind:open class="overflow-hidden p-0">
	<!-- Search input -->
	<div class="flex items-center border-b px-4">
		<Search class="h-4 w-4 text-muted-foreground" />
		<input
			type="text"
			placeholder="Type a command or search..."
			bind:value={searchQuery}
			class="flex-1 bg-transparent px-3 py-4 text-sm outline-none placeholder:text-muted-foreground"
		/>
		<kbd class="hidden rounded bg-muted px-2 py-1 text-xs text-muted-foreground sm:inline-block">
			ESC
		</kbd>
	</div>

	<!-- Command list -->
	<div class="max-h-80 overflow-y-auto p-2">
		{#if filteredCommands().length === 0}
			<div class="py-8 text-center text-sm text-muted-foreground">
				No commands found
			</div>
		{:else}
			{#each filteredCommands() as cmd, index (cmd.id)}
				<button
					class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors {index === selectedIndex ? 'bg-accent text-accent-foreground' : 'text-foreground hover:bg-muted'}"
					onclick={() => executeCommand(cmd)}
					onmouseenter={() => selectedIndex = index}
				>
					<cmd.icon class="h-4 w-4 text-muted-foreground" />
					<div class="flex-1 text-left">
						<div class="font-medium">{cmd.label}</div>
						{#if cmd.description}
							<div class="text-xs text-muted-foreground">{cmd.description}</div>
						{/if}
					</div>
					{#if cmd.shortcut}
						<kbd class="hidden rounded bg-muted px-2 py-0.5 text-xs text-muted-foreground sm:inline-block">
							{cmd.shortcut}
						</kbd>
					{:else if cmd.category === 'action'}
						<span class="rounded bg-primary/10 px-2 py-0.5 text-xs text-primary">Action</span>
					{/if}
				</button>
			{/each}
		{/if}
	</div>

	<!-- Footer hint -->
	<div class="flex items-center justify-between border-t px-4 py-2 text-xs text-muted-foreground">
		<div class="flex items-center gap-2">
			<kbd class="rounded bg-muted px-1.5 py-0.5">↑↓</kbd>
			<span>Navigate</span>
		</div>
		<div class="flex items-center gap-2">
			<kbd class="rounded bg-muted px-1.5 py-0.5">↵</kbd>
			<span>Select</span>
		</div>
	</div>
</Dialog>
