<script lang="ts">
	import { ArrowLeft } from 'lucide-svelte';
	import { goto } from '$app/navigation';

	interface Props {
		title: string;
		subtitle?: string;
		backHref?: string;
		transparent?: boolean;
		children?: import('svelte').Snippet;
	}

	let { title, subtitle, backHref, transparent = false, children }: Props = $props();

	function goBack() {
		if (backHref) {
			goto(backHref);
		} else {
			history.back();
		}
	}
</script>

<header class="mobile-header touch-none-select" class:transparent>
	<div class="header-inner">
		{#if backHref !== undefined}
			<button class="back-btn" onclick={goBack} aria-label="Go back">
				<ArrowLeft size={22} />
			</button>
		{/if}

		<div class="header-titles" class:has-back={backHref !== undefined}>
			<h1 class="header-title">{title}</h1>
			{#if subtitle}
				<p class="header-subtitle">{subtitle}</p>
			{/if}
		</div>

		{#if children}
			<div class="header-actions">
				{@render children()}
			</div>
		{/if}
	</div>
</header>

<style>
	.mobile-header {
		background-color: var(--theme-background);
		border-bottom: 1px solid var(--theme-border);
		position: sticky;
		top: 0;
		z-index: 40;
		backdrop-filter: blur(12px);
		background-color: color-mix(in oklch, var(--theme-background) 85%, transparent);
	}

	.mobile-header.transparent {
		background-color: transparent;
		border-bottom: none;
		backdrop-filter: none;
	}

	.header-inner {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 56px;
		padding: 0 16px;
	}

	.back-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: 10px;
		color: var(--theme-foreground);
		transition: background-color 0.15s ease;
		flex-shrink: 0;
		border: none;
		background: none;
		cursor: pointer;
	}

	.back-btn:active {
		background-color: var(--theme-accent);
	}

	.header-titles {
		flex: 1;
		min-width: 0;
	}

	.header-titles.has-back {
		text-align: center;
		padding-right: 36px;
	}

	.header-title {
		font-size: 17px;
		font-weight: 700;
		line-height: 1.3;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		letter-spacing: -0.01em;
	}

	.header-subtitle {
		font-size: 12px;
		color: var(--theme-muted-foreground);
		line-height: 1.2;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
	}
</style>
