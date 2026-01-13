<script lang="ts">
	import { cn } from '$lib/utils/cn.js';
	import type { HTMLAttributes } from 'svelte/elements';

	interface Props extends HTMLAttributes<HTMLTableRowElement> {
		class?: string;
	}

	let { class: className, children, ...restProps }: Props = $props();
</script>

<tr
	class={cn(
		'border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted',
		className
	)}
	{...restProps}
>
	{@render children?.()}
</tr>
