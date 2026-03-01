<script lang="ts">
	import { cn } from '$lib/utils/cn.js';
	import type { HTMLTdAttributes } from 'svelte/elements';

	interface Props extends HTMLTdAttributes {
		class?: string;
		colspan?: number;
	}

	let { class: className, colspan, children, ...restProps }: Props = $props();
</script>

<td
	class={cn('p-2 align-middle [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]', className)}
	{colspan}
	{...restProps}
>
	{@render children?.()}
</td>
