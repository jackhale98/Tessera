<script lang="ts">
	import { goto } from '$app/navigation';
	import { closeProject } from '$lib/stores/project.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { GitBranch, Network, Info, ChevronRight, LogOut } from 'lucide-svelte';

	const menuItems = [
		{
			href: '/more/deviations',
			icon: GitBranch,
			title: 'Deviations',
			description: 'Manage deviation requests and approvals',
			color: 'var(--theme-warning)'
		},
		{
			href: '/more/traceability',
			icon: Network,
			title: 'Traceability',
			description: 'Trace entity links and dependencies',
			color: 'var(--theme-info)'
		}
	];

	let closing = $state(false);

	async function handleCloseProject() {
		closing = true;
		try {
			await closeProject();
			goto('/project');
		} catch (e) {
			console.error('Failed to close project:', e);
		} finally {
			closing = false;
		}
	}
</script>

<MobileHeader title="More" />

<div class="more-page">
	<!-- Menu Items -->
	<div class="menu-list">
		{#each menuItems as item}
			<a href={item.href} class="menu-item">
				<div class="menu-icon" style="background-color: color-mix(in oklch, {item.color} 15%, transparent); color: {item.color}">
					<item.icon size={20} />
				</div>
				<div class="menu-text">
					<span class="menu-title">{item.title}</span>
					<span class="menu-desc">{item.description}</span>
				</div>
				<ChevronRight size={18} class="menu-chevron" />
			</a>
		{/each}
	</div>

	<!-- About Section -->
	<div class="about-card">
		<div class="about-icon">
			<Info size={20} />
		</div>
		<div class="about-text">
			<span class="about-title">About</span>
			<span class="about-version">Tessera Mobile v0.1.0</span>
		</div>
	</div>

	<!-- Close Project -->
	<button class="close-btn" onclick={handleCloseProject} disabled={closing}>
		<LogOut size={18} />
		<span>Close Project</span>
	</button>
</div>

<style>
	.more-page {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 20px;
		padding-bottom: 32px;
	}

	.menu-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.menu-item {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 16px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 16px;
		text-decoration: none;
		color: inherit;
		transition: all 0.15s ease;
		-webkit-tap-highlight-color: transparent;
	}

	.menu-item:active {
		transform: scale(0.98);
		background-color: var(--theme-accent);
	}

	.menu-icon {
		width: 44px;
		height: 44px;
		border-radius: 14px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.menu-text {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.menu-title {
		font-size: 16px;
		font-weight: 600;
	}

	.menu-desc {
		font-size: 13px;
		color: var(--theme-muted-foreground);
		line-height: 1.3;
	}

	:global(.menu-chevron) {
		color: var(--theme-muted-foreground);
		flex-shrink: 0;
	}

	.about-card {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 16px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 16px;
	}

	.about-icon {
		width: 44px;
		height: 44px;
		border-radius: 14px;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--theme-muted);
		color: var(--theme-muted-foreground);
		flex-shrink: 0;
	}

	.about-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.about-title {
		font-size: 16px;
		font-weight: 600;
	}

	.about-version {
		font-size: 13px;
		color: var(--theme-muted-foreground);
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
		padding: 14px 16px;
		border-radius: 14px;
		font-size: 15px;
		font-weight: 600;
		border: 1px solid var(--theme-error);
		background-color: color-mix(in oklch, var(--theme-error) 10%, transparent);
		color: var(--theme-error);
		cursor: pointer;
		transition: all 0.15s ease;
		-webkit-tap-highlight-color: transparent;
	}

	.close-btn:active {
		transform: scale(0.98);
		background-color: color-mix(in oklch, var(--theme-error) 20%, transparent);
	}

	.close-btn:disabled {
		opacity: 0.5;
	}
</style>
