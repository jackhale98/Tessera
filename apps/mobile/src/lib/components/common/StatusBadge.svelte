<script lang="ts">
	import { Badge } from '$lib/components/ui';
	import { cn } from '$lib/utils/cn';

	interface Props {
		status: string;
		class?: string;
	}

	let { status, class: className }: Props = $props();

	const statusConfig: Record<string, { variant: 'default' | 'secondary' | 'destructive' | 'outline'; label: string }> = {
		draft: { variant: 'outline', label: 'Draft' },
		review: { variant: 'secondary', label: 'Review' },
		approved: { variant: 'default', label: 'Approved' },
		released: { variant: 'default', label: 'Released' },
		obsolete: { variant: 'destructive', label: 'Obsolete' }
	};

	const config = $derived(statusConfig[status.toLowerCase()] ?? { variant: 'outline', label: status });
</script>

<Badge variant={config.variant} class={cn('capitalize', className)}>
	{config.label}
</Badge>
