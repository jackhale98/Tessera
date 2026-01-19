<script lang="ts">
	import { Select as SelectPrimitive } from 'bits-ui';

	interface Props {
		value?: string;
		onValueChange?: (value: string) => void;
		disabled?: boolean;
		children?: import('svelte').Snippet;
	}

	let { value = $bindable(), onValueChange, disabled = false, children, ...restProps }: Props = $props();

	function handleValueChange(newValue: string | undefined) {
		if (newValue !== undefined) {
			value = newValue;
			onValueChange?.(newValue);
		}
	}
</script>

<SelectPrimitive.Root bind:value {disabled} onValueChange={handleValueChange} type="single" {...restProps}>
	{#if children}
		{@render children()}
	{/if}
</SelectPrimitive.Root>
