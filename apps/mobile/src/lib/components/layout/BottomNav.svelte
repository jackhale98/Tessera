<script lang="ts">
	import { page } from '$app/stores';
	import { Home, Package, ShieldCheck, Search, Menu } from 'lucide-svelte';

	const tabs = [
		{ href: '/', label: 'Home', icon: Home, match: /^\/$/ },
		{ href: '/lots', label: 'Lots', icon: Package, match: /^\/lots/ },
		{ href: '/quality', label: 'Quality', icon: ShieldCheck, match: /^\/quality/ },
		{ href: '/browse', label: 'Browse', icon: Search, match: /^\/browse/ },
		{ href: '/more', label: 'More', icon: Menu, match: /^\/more/ }
	];

	function isActive(match: RegExp, pathname: string): boolean {
		return match.test(pathname);
	}
</script>

<nav class="bottom-nav touch-none-select touch-highlight">
	<div class="nav-inner">
		{#each tabs as tab}
			{@const active = isActive(tab.match, $page.url.pathname)}
			<a
				href={tab.href}
				class="nav-item"
				class:active
				aria-current={active ? 'page' : undefined}
			>
				<div class="icon-container" class:active>
					<tab.icon size={22} strokeWidth={active ? 2.5 : 1.8} />
				</div>
				<span class="nav-label" class:active>{tab.label}</span>
			</a>
		{/each}
	</div>
</nav>

<style>
	.bottom-nav {
		background-color: var(--theme-sidebar);
		border-top: 1px solid var(--theme-sidebar-border);
	}

	.nav-inner {
		display: flex;
		justify-content: space-around;
		align-items: center;
		height: 64px;
		max-width: 600px;
		margin: 0 auto;
		padding: 0 8px;
	}

	.nav-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 2px;
		min-width: 64px;
		min-height: 44px;
		padding: 4px 12px;
		text-decoration: none;
		color: var(--theme-muted-foreground);
		transition: color 0.15s ease;
		border-radius: 12px;
		position: relative;
	}

	.nav-item:active {
		opacity: 0.7;
	}

	.icon-container {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 28px;
		border-radius: 14px;
		transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.icon-container.active {
		background-color: var(--theme-primary);
		color: var(--theme-primary-foreground);
		box-shadow: 0 2px 8px color-mix(in oklch, var(--theme-primary) 40%, transparent);
	}

	.nav-item.active {
		color: var(--theme-primary);
	}

	.nav-label {
		font-size: 10px;
		font-weight: 500;
		letter-spacing: 0.02em;
		line-height: 1;
		transition: color 0.15s ease;
	}

	.nav-label.active {
		font-weight: 700;
		color: var(--theme-primary);
	}
</style>
