<script lang="ts">
	import { cn } from '$lib/utils/cn.js';

	interface Props {
		open: boolean;
		onClose?: () => void;
		class?: string;
		children: import('svelte').Snippet;
	}

	let { open = $bindable(), onClose, class: className, children }: Props = $props();

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			open = false;
			onClose?.();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			open = false;
			onClose?.();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-start justify-center bg-black/50 backdrop-blur-sm pt-[15vh]"
		onclick={handleBackdropClick}
	>
		<div
			class={cn(
				'w-full max-w-lg rounded-lg border bg-background shadow-lg animate-in fade-in-0 zoom-in-95',
				className
			)}
		>
			{@render children()}
		</div>
	</div>
{/if}
