<script lang="ts">
	import { Badge } from '$lib/components/ui';
	import { cn } from '$lib/utils/cn';

	interface Props {
		priority: string;
		class?: string;
	}

	let { priority, class: className }: Props = $props();

	const priorityConfig: Record<string, { variant: 'default' | 'secondary' | 'destructive' | 'outline'; label: string }> = {
		low: { variant: 'outline', label: 'Low' },
		medium: { variant: 'secondary', label: 'Medium' },
		high: { variant: 'default', label: 'High' },
		critical: { variant: 'destructive', label: 'Critical' }
	};

	const config = $derived(priorityConfig[priority.toLowerCase()] ?? { variant: 'outline', label: priority });
</script>

<Badge variant={config.variant} class={cn('capitalize', className)}>
	{config.label}
</Badge>
