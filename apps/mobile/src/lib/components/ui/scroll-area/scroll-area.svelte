<script lang="ts">
	import { cn } from '$lib/utils/cn.js';
	import type { HTMLAttributes } from 'svelte/elements';

	interface Props extends HTMLAttributes<HTMLDivElement> {
		class?: string;
		orientation?: 'vertical' | 'horizontal' | 'both';
	}

	let { class: className, orientation = 'vertical', children, ...restProps }: Props = $props();
</script>

<div
	class={cn(
		'relative',
		orientation === 'vertical' && 'overflow-y-auto overflow-x-hidden',
		orientation === 'horizontal' && 'overflow-x-auto overflow-y-hidden',
		orientation === 'both' && 'overflow-auto',
		className
	)}
	{...restProps}
>
	{@render children?.()}
</div>
