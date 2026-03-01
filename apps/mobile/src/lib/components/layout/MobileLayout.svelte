<script lang="ts">
	import BottomNav from './BottomNav.svelte';
	import { isProjectOpen } from '$lib/stores/project.js';
	import { page } from '$app/stores';

	let { children } = $props();

	// Hide bottom nav on project select screen when no project is open
	let showNav = $derived($isProjectOpen || $page.url.pathname !== '/project');
</script>

<div class="mobile-layout">
	<!-- Safe area top spacer -->
	<div class="safe-top bg-background"></div>

	<!-- Scrollable content area -->
	<main class="mobile-content">
		{@render children()}
	</main>

	<!-- Bottom navigation -->
	{#if showNav}
		<BottomNav />
	{/if}

	<!-- Safe area bottom spacer -->
	<div class="safe-bottom bg-sidebar"></div>
</div>

<style>
	.mobile-layout {
		display: flex;
		flex-direction: column;
		height: 100dvh;
		overflow: hidden;
	}

	.mobile-content {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		overscroll-behavior-y: contain;
		-webkit-overflow-scrolling: touch;
	}
</style>
