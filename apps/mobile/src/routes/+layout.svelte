<script lang="ts">
	import '../app.css';
	import { MobileLayout } from '$lib/components/layout/index.js';
	import { onMount } from 'svelte';
	import { checkProjectState } from '$lib/stores/project.js';

	let { children } = $props();

	onMount(async () => {
		await checkProjectState();
	});
</script>

<svelte:head>
	<title>Tessera</title>
	<meta name="description" content="Tessera - Engineering Artifact Management" />
</svelte:head>

<MobileLayout>
	{@render children()}
</MobileLayout>
