<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { entities, traceability } from '$lib/api/tauri.js';
	import type { EntityData } from '$lib/api/types.js';
	import type { LinkInfo } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { StatusBadge } from '$lib/components/common/index.js';
	import { truncateEntityId } from '$lib/config/entities.js';
	import { ArrowRight, ArrowLeft, Link as LinkIcon, Inbox } from 'lucide-svelte';

	let entity = $state<EntityData | null>(null);
	let linksFrom = $state<LinkInfo[]>([]);
	let linksTo = $state<LinkInfo[]>([]);
	let loading = $state(true);

	let entityId = $derived($page.params.id);
	let entityPrefix = $derived(entity?.prefix ?? entityId.split('-')[0]);
	let backHref = $derived(`/browse/${entityPrefix.toLowerCase()}`);

	// Build display fields from entity data, excluding internal fields
	const HIDDEN_FIELDS = new Set(['id', 'title', 'status', 'prefix', 'author', 'created', 'tags', 'data', 'links', 'entity_type']);

	let displayFields = $derived.by(() => {
		if (!entity?.data) return [];
		const fields: Array<{ key: string; value: string }> = [];
		for (const [key, value] of Object.entries(entity.data)) {
			if (HIDDEN_FIELDS.has(key)) continue;
			if (value === null || value === undefined || value === '') continue;
			if (Array.isArray(value) && value.length === 0) continue;
			const display = typeof value === 'object' ? JSON.stringify(value) : String(value);
			fields.push({ key, value: display });
		}
		return fields;
	});

	function formatFieldLabel(key: string): string {
		return key.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
	}

	onMount(async () => {
		try {
			const [entityData, from, to] = await Promise.all([
				entities.get(entityId),
				traceability.getLinksFrom(entityId),
				traceability.getLinksTo(entityId)
			]);
			entity = entityData;
			linksFrom = from;
			linksTo = to;
		} finally {
			loading = false;
		}
	});
</script>

<MobileHeader
	title={entity?.title ?? 'Loading...'}
	{backHref}
/>

{#if loading}
	<div class="loading-container">
		<div class="loading-spinner"></div>
	</div>
{:else if entity}
	<div class="entity-detail">
		<!-- Status & ID Card -->
		<div class="status-card">
			<div class="status-row">
				<span class="entity-id">{truncateEntityId(entity.id)}</span>
				<StatusBadge status={entity.status} />
			</div>
			<h2 class="entity-title">{entity.title}</h2>
		</div>

		<!-- Core Info -->
		<section class="section">
			<h3 class="section-title">Details</h3>
			<div class="info-grid">
				<div class="info-item">
					<span class="info-label">Author</span>
					<span class="info-value">{entity.author}</span>
				</div>
				<div class="info-item">
					<span class="info-label">Created</span>
					<span class="info-value">{new Date(entity.created).toLocaleDateString()}</span>
				</div>
				{#if entity.tags && entity.tags.length > 0}
					<div class="info-item full-width">
						<span class="info-label">Tags</span>
						<div class="tags-row">
							{#each entity.tags as tag}
								<span class="tag">{tag}</span>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		</section>

		<!-- Dynamic Fields -->
		{#if displayFields.length > 0}
			<section class="section">
				<h3 class="section-title">Properties</h3>
				<div class="info-grid">
					{#each displayFields as field}
						<div class="info-item" class:full-width={field.value.length > 40}>
							<span class="info-label">{formatFieldLabel(field.key)}</span>
							<span class="info-value">{field.value}</span>
						</div>
					{/each}
				</div>
			</section>
		{/if}

		<!-- Links From -->
		<section class="section">
			<h3 class="section-title">
				<ArrowRight size={16} />
				Links From ({linksFrom.length})
			</h3>
			{#if linksFrom.length === 0}
				<div class="empty-links">
					<LinkIcon size={20} strokeWidth={1.4} />
					<span>No outgoing links</span>
				</div>
			{:else}
				<div class="links-list">
					{#each linksFrom as link}
						<a href="/browse/entity/{link.target_id}" class="link-card">
							<div class="link-info">
								<span class="link-type">{link.link_type.replace(/_/g, ' ')}</span>
								<span class="link-target-id">{truncateEntityId(link.target_id)}</span>
								{#if link.target_title}
									<span class="link-target-title">{link.target_title}</span>
								{/if}
							</div>
							<ArrowRight size={16} class="link-chevron" />
						</a>
					{/each}
				</div>
			{/if}
		</section>

		<!-- Links To -->
		<section class="section">
			<h3 class="section-title">
				<ArrowLeft size={16} />
				Links To ({linksTo.length})
			</h3>
			{#if linksTo.length === 0}
				<div class="empty-links">
					<LinkIcon size={20} strokeWidth={1.4} />
					<span>No incoming links</span>
				</div>
			{:else}
				<div class="links-list">
					{#each linksTo as link}
						<a href="/browse/entity/{link.source_id}" class="link-card">
							<div class="link-info">
								<span class="link-type">{link.link_type.replace(/_/g, ' ')}</span>
								<span class="link-target-id">{truncateEntityId(link.source_id)}</span>
								{#if link.target_title}
									<span class="link-target-title">{link.target_title}</span>
								{/if}
							</div>
							<ArrowRight size={16} class="link-chevron" />
						</a>
					{/each}
				</div>
			{/if}
		</section>
	</div>
{:else}
	<div class="empty-state">
		<Inbox size={40} strokeWidth={1.2} />
		<p>Entity not found</p>
	</div>
{/if}

<style>
	.entity-detail {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 20px;
		padding-bottom: 32px;
	}

	.status-card {
		background: linear-gradient(135deg, var(--theme-card), color-mix(in oklch, var(--theme-primary) 5%, var(--theme-card)));
		border: 1px solid var(--theme-border);
		border-radius: 16px;
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.status-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.entity-id {
		font-size: 12px;
		font-weight: 600;
		font-family: var(--font-mono);
		color: var(--theme-muted-foreground);
		letter-spacing: 0.03em;
		text-transform: uppercase;
	}

	.entity-title {
		font-size: 18px;
		font-weight: 700;
		line-height: 1.35;
		letter-spacing: -0.01em;
	}

	.section {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.section-title {
		font-size: 15px;
		font-weight: 700;
		padding: 0 4px;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.info-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 8px;
	}

	.info-item {
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 12px;
		padding: 12px 14px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.info-item.full-width {
		grid-column: 1 / -1;
	}

	.info-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--theme-muted-foreground);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.info-value {
		font-size: 14px;
		font-weight: 600;
		word-break: break-word;
	}

	.tags-row {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.tag {
		font-size: 12px;
		font-weight: 600;
		padding: 4px 10px;
		border-radius: 8px;
		background-color: var(--theme-muted);
		color: var(--theme-muted-foreground);
	}

	.links-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.link-card {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 14px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 14px;
		text-decoration: none;
		color: inherit;
		transition: all 0.15s ease;
		-webkit-tap-highlight-color: transparent;
	}

	.link-card:active {
		transform: scale(0.98);
		background-color: var(--theme-accent);
	}

	.link-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.link-type {
		font-size: 11px;
		font-weight: 600;
		color: var(--theme-primary);
		text-transform: capitalize;
	}

	.link-target-id {
		font-size: 11px;
		font-weight: 600;
		font-family: var(--font-mono);
		color: var(--theme-muted-foreground);
		letter-spacing: 0.03em;
		text-transform: uppercase;
	}

	.link-target-title {
		font-size: 14px;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	:global(.link-chevron) {
		color: var(--theme-muted-foreground);
		flex-shrink: 0;
	}

	.empty-links {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 16px;
		background-color: var(--theme-card);
		border: 1px dashed var(--theme-border);
		border-radius: 14px;
		color: var(--theme-muted-foreground);
		font-size: 13px;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 48px 16px;
		color: var(--theme-muted-foreground);
	}

	.loading-container {
		display: flex;
		justify-content: center;
		padding: 64px 16px;
	}

	.loading-spinner {
		width: 32px;
		height: 32px;
		border: 3px solid var(--theme-border);
		border-top-color: var(--theme-primary);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
