<script lang="ts">
	import { cn } from '$lib/utils/cn.js';

	interface Props {
		class?: string;
		children: import('svelte').Snippet;
	}

	let { class: className, children }: Props = $props();
</script>

<h2 class={cn('text-lg font-semibold leading-none tracking-tight', className)}>
	{@render children()}
</h2>
