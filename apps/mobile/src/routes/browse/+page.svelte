<script lang="ts">
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { entityCounts } from '$lib/stores/project.js';
	import { getEntityColorSolid } from '$lib/config/entities.js';
	import {
		FileText, AlertTriangle, FlaskConical, ClipboardCheck, Cpu, Layers,
		Cog, Gauge, BookOpen, Package, GitBranch, ShieldAlert, ShieldCheck
	} from 'lucide-svelte';

	const entityTypes = [
		{ prefix: 'REQ', label: 'Requirements', countKey: 'requirements', icon: FileText },
		{ prefix: 'RISK', label: 'Risks', countKey: 'risks', icon: AlertTriangle },
		{ prefix: 'TEST', label: 'Tests', countKey: 'tests', icon: FlaskConical },
		{ prefix: 'RSLT', label: 'Results', countKey: 'results', icon: ClipboardCheck },
		{ prefix: 'CMP', label: 'Components', countKey: 'components', icon: Cpu },
		{ prefix: 'ASM', label: 'Assemblies', countKey: 'assemblies', icon: Layers },
		{ prefix: 'PROC', label: 'Processes', countKey: 'processes', icon: Cog },
		{ prefix: 'CTRL', label: 'Controls', countKey: 'controls', icon: Gauge },
		{ prefix: 'WORK', label: 'Work Instructions', countKey: 'work_instructions', icon: BookOpen },
		{ prefix: 'LOT', label: 'Lots', countKey: 'lots', icon: Package },
		{ prefix: 'DEV', label: 'Deviations', countKey: 'deviations', icon: GitBranch },
		{ prefix: 'NCR', label: 'NCRs', countKey: 'ncrs', icon: ShieldAlert },
		{ prefix: 'CAPA', label: 'CAPAs', countKey: 'capas', icon: ShieldCheck }
	];

	function getCount(countKey: string): number {
		const counts = $entityCounts;
		if (!counts) return 0;
		return (counts as Record<string, number>)[countKey] ?? 0;
	}
</script>

<MobileHeader title="Browse" />

<div class="browse-page">
	<div class="type-grid">
		{#each entityTypes as type}
			{@const color = getEntityColorSolid(type.prefix)}
			<a href="/browse/{type.prefix.toLowerCase()}" class="type-card">
				<div class="type-icon" style="background-color: {color}">
					<type.icon size={20} />
				</div>
				<span class="type-name">{type.label}</span>
				<span class="type-count">{getCount(type.countKey)}</span>
			</a>
		{/each}
	</div>
</div>

<style>
	.browse-page {
		padding: 16px;
		padding-bottom: 32px;
	}

	.type-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 10px;
	}

	.type-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 20px 12px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 16px;
		text-decoration: none;
		color: inherit;
		transition: all 0.15s ease;
		-webkit-tap-highlight-color: transparent;
	}

	.type-card:active {
		transform: scale(0.96);
		background-color: var(--theme-accent);
	}

	.type-icon {
		width: 44px;
		height: 44px;
		border-radius: 14px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: white;
	}

	.type-name {
		font-size: 13px;
		font-weight: 600;
		text-align: center;
		line-height: 1.3;
	}

	.type-count {
		font-size: 20px;
		font-weight: 800;
		letter-spacing: -0.02em;
		line-height: 1;
		color: var(--theme-muted-foreground);
	}
</style>
