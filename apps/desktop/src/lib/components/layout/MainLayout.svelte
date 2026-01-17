<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import Sidebar from './Sidebar.svelte';
	import Header from './Header.svelte';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';

	interface Props {
		children?: import('svelte').Snippet;
	}

	let { children }: Props = $props();

	let scrollContainer = $state<HTMLElement | null>(null);

	// Reset scroll position to top when navigating between pages
	afterNavigate(() => {
		if (scrollContainer) {
			scrollContainer.scrollTo({ top: 0, left: 0 });
		}
	});
</script>

<div class="flex h-screen w-screen overflow-hidden bg-background">
	<!-- Sidebar -->
	<Sidebar />

	<!-- Main content area -->
	<div class="flex flex-1 flex-col overflow-hidden">
		<!-- Header -->
		<Header />

		<!-- Page content -->
		<main class="flex-1 overflow-hidden">
			<div
				bind:this={scrollContainer}
				class="h-full overflow-auto"
			>
				<div class="p-6">
					{@render children?.()}
				</div>
			</div>
		</main>
	</div>
</div>
