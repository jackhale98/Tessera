<script lang="ts">
	import { cn } from '$lib/utils/cn.js';
	import { badgeVariants, type BadgeVariant } from './index.js';
	import type { HTMLAttributes } from 'svelte/elements';

	interface Props extends HTMLAttributes<HTMLDivElement> {
		variant?: BadgeVariant;
		class?: string;
	}

	let { class: className, variant = 'default', children, ...restProps }: Props = $props();
</script>

<div class={cn(badgeVariants({ variant }), className)} {...restProps}>
	{@render children?.()}
</div>
