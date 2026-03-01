<script lang="ts">
	interface Props {
		placeholder?: string;
		class?: string;
	}

	let { placeholder = 'Select...', class: className }: Props = $props();
</script>

<!-- SelectValue is handled by bits-ui Trigger internally, this is just a placeholder display -->
<span class={className}>{placeholder}</span>
