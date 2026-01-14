<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle, Badge } from '$lib/components/ui';
	import { EntityDetailHeader, LinksSection } from '$lib/components/entities';
	import { StatusBadge } from '$lib/components/common';
	import { entities, traceability } from '$lib/api';
	import type { EntityData } from '$lib/api/types';
	import type { LinkInfo } from '$lib/api/tauri';
	import {
		Building2,
		User,
		Calendar,
		Tag,
		FileText,
		Award,
		Wrench,
		Mail,
		Phone,
		MapPin,
		Globe
	} from 'lucide-svelte';

	const id = $derived($page.params.id);

	let entity = $state<EntityData | null>(null);
	let linksFrom = $state<LinkInfo[]>([]);
	let linksTo = $state<LinkInfo[]>([]);
	let loading = $state(true);
	let linksLoading = $state(true);
	let error = $state<string | null>(null);

	// Track if we've loaded for this ID to prevent double-loads
	let loadedId = $state<string | null>(null);

	// Type-safe data access
	const data = $derived(entity?.data ?? {});
	const description = $derived((data.description as string) ?? '');
	const website = $derived((data.website as string) ?? null);
	const address = $derived((data.address as string) ?? null);
	const revision = $derived((data.revision as number) ?? 1);

	interface Contact {
		name: string;
		role?: string;
		email?: string;
		phone?: string;
	}
	const contacts = $derived((data.contacts as Contact[]) ?? []);

	interface Certification {
		name: string;
		number?: string;
		expiration?: string;
	}
	const certifications = $derived((data.certifications as Certification[]) ?? []);

	const capabilities = $derived((data.capabilities as string[]) ?? []);

	async function loadData() {
		if (!id) return;

		loading = true;
		linksLoading = true;
		error = null;

		try {
			const [entityResult, fromLinks, toLinks] = await Promise.all([
				entities.get(id),
				traceability.getLinksFrom(id),
				traceability.getLinksTo(id)
			]);

			entity = entityResult;
			linksFrom = fromLinks;
			linksTo = toLinks;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to load supplier:', e);
		} finally {
			loading = false;
			linksLoading = false;
		}
	}

	function formatDate(dateStr: string): string {
		try {
			return new Date(dateStr).toLocaleDateString('en-US', {
				year: 'numeric',
				month: 'short',
				day: 'numeric'
			});
		} catch {
			return dateStr;
		}
	}

	$effect(() => {
		// Only load if we have an ID and haven't already loaded this ID
		if (id && id !== loadedId) {
			loadedId = id;
			loadData();
		}
	});
</script>

<div class="space-y-6">
	{#if loading}
		<div class="flex h-64 items-center justify-center">
			<div class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
		</div>
	{:else if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
			</CardContent>
		</Card>
	{:else if entity}
		<!-- Header -->
		<EntityDetailHeader
			id={entity.id}
			title={entity.title}
			status={entity.status}
			subtitle="Supplier"
			backHref="/procurement/suppliers"
			backLabel="Suppliers"
			onEdit={() => goto(`/procurement/suppliers/${id}/edit`)}
		/>

		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Main content -->
			<div class="space-y-6 lg:col-span-2">
				<!-- Description -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<FileText class="h-5 w-5" />
							Description
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p class="whitespace-pre-wrap">{description || 'No description specified.'}</p>
					</CardContent>
				</Card>

				<!-- Contacts -->
				{#if contacts.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<User class="h-5 w-5" />
								Contacts ({contacts.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="grid gap-4 sm:grid-cols-2">
								{#each contacts as contact}
									<div class="rounded-lg border p-4">
										<div class="font-medium">{contact.name}</div>
										{#if contact.role}
											<div class="text-sm text-muted-foreground">{contact.role}</div>
										{/if}
										<div class="mt-3 space-y-2 text-sm">
											{#if contact.email}
												<div class="flex items-center gap-2">
													<Mail class="h-3 w-3 text-muted-foreground" />
													<a href="mailto:{contact.email}" class="text-primary hover:underline">
														{contact.email}
													</a>
												</div>
											{/if}
											{#if contact.phone}
												<div class="flex items-center gap-2">
													<Phone class="h-3 w-3 text-muted-foreground" />
													<span>{contact.phone}</span>
												</div>
											{/if}
										</div>
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Certifications -->
				{#if certifications.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Award class="h-5 w-5" />
								Certifications ({certifications.length})
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="space-y-3">
								{#each certifications as cert}
									<div class="flex items-center justify-between rounded-lg border p-3">
										<div>
											<div class="font-medium">{cert.name}</div>
											{#if cert.number}
												<div class="text-sm text-muted-foreground">#{cert.number}</div>
											{/if}
										</div>
										{#if cert.expiration}
											<Badge variant="outline">
												Expires: {formatDate(cert.expiration)}
											</Badge>
										{/if}
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Capabilities -->
				{#if capabilities.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Wrench class="h-5 w-5" />
								Capabilities
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="flex flex-wrap gap-2">
								{#each capabilities as capability}
									<Badge variant="secondary">{capability}</Badge>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- Links -->
				<LinksSection {linksFrom} {linksTo} loading={linksLoading} />
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Properties -->
				<Card>
					<CardHeader>
						<CardTitle>Properties</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Status</span>
							<StatusBadge status={entity.status} />
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Certifications</span>
							<Badge variant="outline">{certifications.length}</Badge>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-muted-foreground">Revision</span>
							<span class="text-sm font-medium">{revision}</span>
						</div>
					</CardContent>
				</Card>

				<!-- Contact Info -->
				{#if website || address}
					<Card>
						<CardHeader>
							<CardTitle>Contact Info</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if website}
								<div class="flex items-center gap-2">
									<Globe class="h-4 w-4 text-muted-foreground" />
									<a href={website} target="_blank" rel="noopener noreferrer" class="text-sm text-primary hover:underline truncate">
										{website}
									</a>
								</div>
							{/if}
							{#if address}
								<div class="flex items-start gap-2">
									<MapPin class="h-4 w-4 text-muted-foreground mt-0.5" />
									<span class="text-sm text-muted-foreground whitespace-pre-line">{address}</span>
								</div>
							{/if}
						</CardContent>
					</Card>
				{/if}

				<!-- Metadata -->
				<Card>
					<CardHeader>
						<CardTitle>Metadata</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="flex items-center gap-2">
							<User class="h-4 w-4 text-muted-foreground" />
							<span class="text-sm text-muted-foreground">Author</span>
							<span class="ml-auto text-sm font-medium">{entity.author}</span>
						</div>
						<div class="flex items-center gap-2">
							<Calendar class="h-4 w-4 text-muted-foreground" />
							<span class="text-sm text-muted-foreground">Created</span>
							<span class="ml-auto text-sm font-medium">{formatDate(entity.created)}</span>
						</div>
					</CardContent>
				</Card>

				<!-- Tags -->
				{#if entity.tags && entity.tags.length > 0}
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Tag class="h-4 w-4" />
								Tags
							</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="flex flex-wrap gap-2">
								{#each entity.tags as tag}
									<Badge variant="outline">{tag}</Badge>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}
			</div>
		</div>
	{:else}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<p class="text-muted-foreground">Supplier not found</p>
			</CardContent>
		</Card>
	{/if}
</div>
