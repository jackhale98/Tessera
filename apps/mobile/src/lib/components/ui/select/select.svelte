<script lang="ts">
	import { cn } from '$lib/utils/cn.js';
	import type { HTMLSelectAttributes } from 'svelte/elements';
	import { ChevronDown } from 'lucide-svelte';

	interface Props extends HTMLSelectAttributes {
		class?: string;
		value?: string | number | null;
	}

	let { class: className, value = $bindable(), children, ...restProps }: Props = $props();
</script>

<div class="relative">
	<select
		class={cn(
			'flex h-9 w-full appearance-none rounded-md border border-input bg-transparent px-3 py-1 pr-8 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50',
			className
		)}
		bind:value
		{...restProps}
	>
		{@render children?.()}
	</select>
	<ChevronDown class="pointer-events-none absolute right-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
</div>
