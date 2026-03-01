<script lang="ts">
	import StatusBadge from './StatusBadge.svelte';
	import { getEntityColorSolid, truncateEntityId } from '$lib/config/entities.js';

	interface Props {
		id: string;
		title: string;
		subtitle?: string;
		status?: string;
		prefix?: string;
		href?: string;
		meta?: string;
		onclick?: () => void;
	}

	let { id, title, subtitle, status, prefix, href, meta, onclick }: Props = $props();

	let entityPrefix = $derived(prefix || id.split('-')[0]);
	let accentColor = $derived(getEntityColorSolid(entityPrefix));
</script>

{#if href}
	<a {href} class="entity-card touch-highlight" data-accent={entityPrefix}>
		<div class="card-accent" style="background-color: {accentColor}"></div>
		<div class="card-body">
			<div class="card-top">
				<span class="card-id">{truncateEntityId(id)}</span>
				{#if status}
					<StatusBadge {status} />
				{/if}
			</div>
			<h3 class="card-title">{title}</h3>
			{#if subtitle}
				<p class="card-subtitle">{subtitle}</p>
			{/if}
			{#if meta}
				<p class="card-meta">{meta}</p>
			{/if}
		</div>
	</a>
{:else}
	<button class="entity-card touch-highlight" onclick={onclick} type="button">
		<div class="card-accent" style="background-color: {accentColor}"></div>
		<div class="card-body">
			<div class="card-top">
				<span class="card-id">{truncateEntityId(id)}</span>
				{#if status}
					<StatusBadge {status} />
				{/if}
			</div>
			<h3 class="card-title">{title}</h3>
			{#if subtitle}
				<p class="card-subtitle">{subtitle}</p>
			{/if}
			{#if meta}
				<p class="card-meta">{meta}</p>
			{/if}
		</div>
	</button>
{/if}

<style>
	.entity-card {
		display: flex;
		width: 100%;
		text-decoration: none;
		color: inherit;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 16px;
		overflow: hidden;
		transition: all 0.15s ease;
		text-align: left;
		cursor: pointer;
		-webkit-tap-highlight-color: transparent;
	}

	.entity-card:active {
		transform: scale(0.98);
		background-color: var(--theme-accent);
	}

	.card-accent {
		width: 4px;
		flex-shrink: 0;
		border-radius: 4px 0 0 4px;
	}

	.card-body {
		flex: 1;
		padding: 14px 16px;
		min-width: 0;
	}

	.card-top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 6px;
	}

	.card-id {
		font-size: 11px;
		font-weight: 600;
		font-family: var(--font-mono);
		color: var(--theme-muted-foreground);
		letter-spacing: 0.03em;
		text-transform: uppercase;
	}

	.card-title {
		font-size: 15px;
		font-weight: 600;
		line-height: 1.35;
		color: var(--theme-foreground);
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.card-subtitle {
		font-size: 13px;
		color: var(--theme-muted-foreground);
		margin-top: 4px;
		line-height: 1.3;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.card-meta {
		font-size: 11px;
		color: var(--theme-muted-foreground);
		margin-top: 6px;
		opacity: 0.7;
	}
</style>
