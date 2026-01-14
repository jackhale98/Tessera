<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui';
	import { StatusBadge } from '$lib/components/common';
	import { ArrowLeft, Edit, Trash2 } from 'lucide-svelte';

	interface Props {
		id: string;
		title: string;
		status: string;
		subtitle?: string;
		backHref: string;
		backLabel?: string;
		onEdit?: () => void;
		onDelete?: () => void;
	}

	let {
		id,
		title,
		status,
		subtitle,
		backHref,
		backLabel = 'Back',
		onEdit,
		onDelete
	}: Props = $props();
</script>

<div class="space-y-4">
	<!-- Back navigation -->
	<Button variant="ghost" size="sm" onclick={() => goto(backHref)}>
		<ArrowLeft class="mr-2 h-4 w-4" />
		{backLabel}
	</Button>

	<!-- Header -->
	<div class="flex items-start justify-between">
		<div class="space-y-1">
			<div class="flex items-center gap-3">
				<h1 class="text-2xl font-bold">{title}</h1>
				<StatusBadge {status} />
			</div>
			<p class="font-mono text-sm text-muted-foreground">{id}</p>
			{#if subtitle}
				<p class="text-muted-foreground">{subtitle}</p>
			{/if}
		</div>

		<div class="flex items-center gap-2">
			{#if onEdit}
				<Button variant="outline" size="sm" onclick={onEdit}>
					<Edit class="mr-2 h-4 w-4" />
					Edit
				</Button>
			{/if}
			{#if onDelete}
				<Button variant="outline" size="sm" class="text-destructive" onclick={onDelete}>
					<Trash2 class="mr-2 h-4 w-4" />
					Delete
				</Button>
			{/if}
		</div>
	</div>
</div>
