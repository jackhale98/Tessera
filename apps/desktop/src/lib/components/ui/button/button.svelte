<script lang="ts">
	import { cn } from '$lib/utils/cn.js';
	import { buttonVariants, type ButtonVariant, type ButtonSize } from './index.js';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	interface Props extends HTMLButtonAttributes {
		variant?: ButtonVariant;
		size?: ButtonSize;
		class?: string;
	}

	let {
		class: className,
		variant = 'default',
		size = 'default',
		children,
		...restProps
	}: Props = $props();
</script>

<button class={cn(buttonVariants({ variant, size }), className)} {...restProps}>
	{@render children?.()}
</button>
