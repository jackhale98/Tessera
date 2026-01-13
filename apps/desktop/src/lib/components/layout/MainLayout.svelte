<script lang="ts">
	import Sidebar from './Sidebar.svelte';
	import Header from './Header.svelte';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';

	interface Props {
		children?: import('svelte').Snippet;
	}

	let { children }: Props = $props();
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
			<ScrollArea class="h-full" orientation="both">
				<div class="p-6">
					{@render children?.()}
				</div>
			</ScrollArea>
		</main>
	</div>
</div>
