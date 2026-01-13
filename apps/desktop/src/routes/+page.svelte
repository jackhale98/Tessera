<script lang="ts">
	import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { isProjectOpen, entityCounts, projectName, totalEntities } from '$lib/stores/project.js';
	import { openProject, initProject } from '$lib/stores/project.js';
	import { open } from '@tauri-apps/plugin-dialog';
	import {
		FileText,
		AlertTriangle,
		FlaskConical,
		Box,
		Factory,
		Shield,
		FolderOpen,
		FolderPlus,
		ArrowRight
	} from 'lucide-svelte';

	interface StatCard {
		label: string;
		value: number;
		icon: typeof FileText;
		href: string;
		color: string;
	}

	let stats: StatCard[] = $derived.by(() => {
		const counts = $entityCounts;
		if (!counts) return [];

		return [
			{
				label: 'Requirements',
				value: counts.requirements,
				icon: FileText,
				href: '/requirements',
				color: 'text-blue-500'
			},
			{
				label: 'Risks',
				value: counts.risks,
				icon: AlertTriangle,
				href: '/risks',
				color: 'text-amber-500'
			},
			{
				label: 'Tests',
				value: counts.tests + counts.results,
				icon: FlaskConical,
				href: '/verification',
				color: 'text-green-500'
			},
			{
				label: 'Components',
				value: counts.components + counts.assemblies,
				icon: Box,
				href: '/bom',
				color: 'text-purple-500'
			},
			{
				label: 'Manufacturing',
				value: counts.processes + counts.lots + counts.work_instructions,
				icon: Factory,
				href: '/manufacturing',
				color: 'text-orange-500'
			},
			{
				label: 'Quality',
				value: counts.ncrs + counts.capas,
				icon: Shield,
				href: '/quality',
				color: 'text-red-500'
			}
		];
	});

	async function handleOpenProject() {
		const selected = await open({
			directory: true,
			multiple: false,
			title: 'Open TDT Project'
		});

		if (selected && typeof selected === 'string') {
			await openProject(selected);
		}
	}

	async function handleInitProject() {
		const selected = await open({
			directory: true,
			multiple: false,
			title: 'Select Directory for New Project'
		});

		if (selected && typeof selected === 'string') {
			await initProject(selected);
		}
	}
</script>

{#if !$isProjectOpen}
	<!-- Welcome screen when no project is open -->
	<div class="flex h-full items-center justify-center">
		<div class="max-w-lg text-center">
			<div class="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-2xl bg-primary/10">
				<span class="text-4xl font-bold text-primary">T</span>
			</div>
			<h1 class="mb-2 text-3xl font-bold tracking-tight">Welcome to TDT Desktop</h1>
			<p class="mb-8 text-muted-foreground">
				Tessera Design Toolkit helps you manage engineering artifacts including requirements, risks,
				tests, BOMs, and more with full traceability.
			</p>
			<div class="flex flex-col gap-3 sm:flex-row sm:justify-center">
				<Button variant="outline" size="lg" onclick={handleOpenProject}>
					<FolderOpen class="mr-2 h-5 w-5" />
					Open Existing Project
				</Button>
				<Button size="lg" onclick={handleInitProject}>
					<FolderPlus class="mr-2 h-5 w-5" />
					Create New Project
				</Button>
			</div>
		</div>
	</div>
{:else}
	<!-- Dashboard when project is open -->
	<div class="space-y-6">
		<!-- Header -->
		<div>
			<h1 class="text-2xl font-bold tracking-tight">{$projectName}</h1>
			<p class="text-muted-foreground">
				{$totalEntities} total entities across all categories
			</p>
		</div>

		<!-- Stats grid -->
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each stats as stat}
				<a href={stat.href} class="block">
					<Card class="transition-colors hover:bg-accent/50">
						<CardHeader class="flex flex-row items-center justify-between pb-2">
							<CardTitle class="text-sm font-medium text-muted-foreground">
								{stat.label}
							</CardTitle>
							<stat.icon class="h-4 w-4 {stat.color}" />
						</CardHeader>
						<CardContent>
							<div class="flex items-center justify-between">
								<div class="text-2xl font-bold">{stat.value}</div>
								<ArrowRight class="h-4 w-4 text-muted-foreground" />
							</div>
						</CardContent>
					</Card>
				</a>
			{/each}
		</div>

		<!-- Quick actions -->
		<Card>
			<CardHeader>
				<CardTitle>Quick Actions</CardTitle>
				<CardDescription>Common tasks to get you started</CardDescription>
			</CardHeader>
			<CardContent>
				<div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-4">
					<Button variant="outline" class="justify-start">
						<FileText class="mr-2 h-4 w-4" />
						New Requirement
					</Button>
					<Button variant="outline" class="justify-start">
						<AlertTriangle class="mr-2 h-4 w-4" />
						New Risk
					</Button>
					<Button variant="outline" class="justify-start">
						<FlaskConical class="mr-2 h-4 w-4" />
						New Test
					</Button>
					<Button variant="outline" class="justify-start">
						<Box class="mr-2 h-4 w-4" />
						New Component
					</Button>
				</div>
			</CardContent>
		</Card>

		<!-- Recent activity placeholder -->
		<Card>
			<CardHeader>
				<CardTitle>Recent Activity</CardTitle>
				<CardDescription>Recently modified entities</CardDescription>
			</CardHeader>
			<CardContent>
				<div class="flex h-32 items-center justify-center text-muted-foreground">
					<p class="text-sm">Activity tracking coming soon...</p>
				</div>
			</CardContent>
		</Card>
	</div>
{/if}
