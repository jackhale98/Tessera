<script lang="ts">
	import '../app.css';
	import { MainLayout } from '$lib/components/layout/index.js';
	import { onMount } from 'svelte';
	import { checkProjectState } from '$lib/stores/project.js';

	let { children } = $props();

	onMount(async () => {
		// Check if there's a project already open (e.g., from a previous session)
		await checkProjectState();
	});
</script>

<svelte:head>
	<title>TDT Desktop</title>
	<meta name="description" content="Tessera Design Toolkit - Engineering Artifact Management" />
</svelte:head>

<MainLayout>
	{@render children()}
</MainLayout>
