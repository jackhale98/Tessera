<script lang="ts">
	import { onMount } from 'svelte';
	import {
		Card,
		CardHeader,
		CardTitle,
		CardDescription,
		CardContent,
		CardFooter
	} from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import { isProjectOpen } from '$lib/stores/project.js';
	import { settings } from '$lib/api/index.js';
	import type {
		AllSettings,
		GeneralSettings,
		WorkflowSettings,
		ManufacturingSettings,
		TeamRosterDto,
		TeamMemberDto,
		EntityPrefixInfo
	} from '$lib/api/index.js';
	import {
		Settings,
		User,
		GitBranch,
		Factory,
		Users,
		Save,
		Plus,
		Trash2,
		Check,
		X,
		Shield,
		RefreshCw,
		FolderCog,
		Globe
	} from 'lucide-svelte';

	// State
	let loading = $state(true);
	let saving = $state(false);
	let error = $state<string | null>(null);
	let successMessage = $state<string | null>(null);
	let activeTab = $state<'general' | 'workflow' | 'manufacturing' | 'team'>('general');

	// Settings data
	let allSettings = $state<AllSettings | null>(null);
	let teamRoster = $state<TeamRosterDto | null>(null);
	let availableRoles = $state<string[]>([]);
	let availableSigningFormats = $state<string[]>([]);
	let entityPrefixes = $state<EntityPrefixInfo[]>([]);

	// Form state for general settings
	let generalForm = $state<GeneralSettings>({
		author: null,
		editor: null,
		pager: null,
		default_format: null
	});

	// Form state for workflow settings
	let workflowForm = $state<WorkflowSettings>({
		enabled: false,
		provider: 'none',
		require_branch: true,
		auto_commit: true,
		auto_merge: false,
		base_branch: 'main',
		branch_pattern: 'review/{prefix}-{short_id}',
		submit_message: 'Submit {id}: {title}',
		approve_message: 'Approve {id}: {title}'
	});

	// Form state for manufacturing settings
	let manufacturingForm = $state<ManufacturingSettings>({
		lot_branch_enabled: false,
		base_branch: null,
		branch_pattern: null,
		create_tags: true,
		sign_commits: false
	});

	// New member form
	let showNewMemberForm = $state(false);
	let newMember = $state<TeamMemberDto>({
		name: '',
		email: '',
		username: '',
		roles: [],
		active: true,
		signing_format: null
	});

	// Editing member
	let editingMember = $state<string | null>(null);
	let editMemberForm = $state<TeamMemberDto | null>(null);

	// New approval matrix entry
	let showNewApprovalEntry = $state(false);
	let newApprovalPrefix = $state('');
	let newApprovalRoles = $state<string[]>([]);

	async function loadSettings() {
		loading = true;
		error = null;

		try {
			const [settingsData, roster, roles, formats, prefixes] = await Promise.all([
				settings.getAll(),
				settings.getTeamRoster(),
				settings.getAvailableRoles(),
				settings.getAvailableSigningFormats(),
				settings.getEntityPrefixesForApproval()
			]);

			allSettings = settingsData;
			teamRoster = roster;
			availableRoles = roles;
			availableSigningFormats = formats;
			entityPrefixes = prefixes;

			// Initialize forms
			generalForm = { ...settingsData.general };
			workflowForm = { ...settingsData.workflow };
			manufacturingForm = { ...settingsData.manufacturing };
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	async function saveGeneralSettings(toGlobal: boolean) {
		saving = true;
		error = null;
		successMessage = null;

		try {
			await settings.saveGeneral(generalForm, toGlobal);
			successMessage = `General settings saved to ${toGlobal ? 'global' : 'project'} config`;
			setTimeout(() => (successMessage = null), 3000);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}

	async function saveWorkflowSettings() {
		saving = true;
		error = null;
		successMessage = null;

		try {
			await settings.saveWorkflow(workflowForm);
			successMessage = 'Workflow settings saved';
			setTimeout(() => (successMessage = null), 3000);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}

	async function saveManufacturingSettings() {
		saving = true;
		error = null;
		successMessage = null;

		try {
			await settings.saveManufacturing(manufacturingForm);
			successMessage = 'Manufacturing settings saved';
			setTimeout(() => (successMessage = null), 3000);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}

	async function initTeamRoster() {
		try {
			teamRoster = await settings.initTeamRoster();
			successMessage = 'Team roster initialized';
			setTimeout(() => (successMessage = null), 3000);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function prefillNewMemberFromGit() {
		try {
			const gitUser = await settings.getCurrentGitUser();
			if (gitUser.name) newMember.name = gitUser.name;
			if (gitUser.email) newMember.email = gitUser.email;
			if (gitUser.name) newMember.username = gitUser.name;
		} catch (e) {
			console.error('Failed to get git user:', e);
		}
	}

	async function addTeamMember() {
		if (!newMember.name || !newMember.email || !newMember.username) {
			error = 'Name, email, and username are required';
			return;
		}

		try {
			teamRoster = await settings.addTeamMember(newMember);
			showNewMemberForm = false;
			newMember = {
				name: '',
				email: '',
				username: '',
				roles: [],
				active: true,
				signing_format: null
			};
			successMessage = 'Team member added';
			setTimeout(() => (successMessage = null), 3000);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function updateTeamMember() {
		if (!editingMember || !editMemberForm) return;

		try {
			teamRoster = await settings.updateTeamMember(editingMember, editMemberForm);
			editingMember = null;
			editMemberForm = null;
			successMessage = 'Team member updated';
			setTimeout(() => (successMessage = null), 3000);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function removeTeamMember(username: string) {
		if (!confirm(`Remove team member ${username}?`)) return;

		try {
			teamRoster = await settings.removeTeamMember(username);
			successMessage = 'Team member removed';
			setTimeout(() => (successMessage = null), 3000);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function toggleMemberActive(username: string, currentActive: boolean) {
		try {
			teamRoster = await settings.setTeamMemberActive(username, !currentActive);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function updateApprovalMatrix(prefix: string, roles: string[]) {
		try {
			teamRoster = await settings.updateApprovalMatrix(prefix, roles);
			successMessage = 'Approval matrix updated';
			setTimeout(() => (successMessage = null), 3000);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	async function removeApprovalEntry(prefix: string) {
		try {
			teamRoster = await settings.removeApprovalMatrixEntry(prefix);
			successMessage = 'Approval entry removed';
			setTimeout(() => (successMessage = null), 3000);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
	}

	function startEditMember(member: TeamMemberDto) {
		editingMember = member.username;
		editMemberForm = { ...member };
	}

	function cancelEditMember() {
		editingMember = null;
		editMemberForm = null;
	}

	function toggleRole(member: TeamMemberDto, role: string) {
		if (member.roles.includes(role)) {
			member.roles = member.roles.filter((r) => r !== role);
		} else {
			member.roles = [...member.roles, role];
		}
	}

	onMount(() => {
		if ($isProjectOpen) {
			loadSettings();
		}
	});

	$effect(() => {
		if ($isProjectOpen && !allSettings) {
			loadSettings();
		}
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div>
		<h1 class="text-2xl font-bold tracking-tight flex items-center gap-2">
			<Settings class="h-6 w-6" />
			Settings
		</h1>
		<p class="text-muted-foreground">Configure Tessera project and user settings</p>
	</div>

	{#if !$isProjectOpen}
		<Card>
			<CardContent class="flex h-64 items-center justify-center">
				<div class="text-center">
					<FolderCog class="h-12 w-12 mx-auto mb-4 text-muted-foreground/50" />
					<p class="text-muted-foreground">Open a project to manage settings</p>
				</div>
			</CardContent>
		</Card>
	{:else if loading}
		<div class="flex h-64 items-center justify-center">
			<div class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"
			></div>
		</div>
	{:else if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
				<Button variant="outline" class="mt-4" onclick={loadSettings}>
					<RefreshCw class="h-4 w-4 mr-2" />
					Retry
				</Button>
			</CardContent>
		</Card>
	{:else}
		<!-- Success message -->
		{#if successMessage}
			<div
				class="fixed top-4 right-4 z-50 bg-green-500 text-white px-4 py-2 rounded-lg shadow-lg flex items-center gap-2"
			>
				<Check class="h-4 w-4" />
				{successMessage}
			</div>
		{/if}

		<!-- Tabs -->
		<div class="flex gap-2 border-b">
			<button
				class="px-4 py-2 -mb-px border-b-2 transition-colors {activeTab === 'general'
					? 'border-primary text-primary font-medium'
					: 'border-transparent text-muted-foreground hover:text-foreground'}"
				onclick={() => (activeTab = 'general')}
			>
				<User class="h-4 w-4 inline mr-2" />
				General
			</button>
			<button
				class="px-4 py-2 -mb-px border-b-2 transition-colors {activeTab === 'workflow'
					? 'border-primary text-primary font-medium'
					: 'border-transparent text-muted-foreground hover:text-foreground'}"
				onclick={() => (activeTab = 'workflow')}
			>
				<GitBranch class="h-4 w-4 inline mr-2" />
				Workflow
			</button>
			<button
				class="px-4 py-2 -mb-px border-b-2 transition-colors {activeTab === 'manufacturing'
					? 'border-primary text-primary font-medium'
					: 'border-transparent text-muted-foreground hover:text-foreground'}"
				onclick={() => (activeTab = 'manufacturing')}
			>
				<Factory class="h-4 w-4 inline mr-2" />
				Manufacturing
			</button>
			<button
				class="px-4 py-2 -mb-px border-b-2 transition-colors {activeTab === 'team'
					? 'border-primary text-primary font-medium'
					: 'border-transparent text-muted-foreground hover:text-foreground'}"
				onclick={() => (activeTab = 'team')}
			>
				<Users class="h-4 w-4 inline mr-2" />
				Team Roster
			</button>
		</div>

		<!-- General Settings Tab -->
		{#if activeTab === 'general'}
			<Card>
				<CardHeader>
					<CardTitle>General Settings</CardTitle>
					<CardDescription>Basic configuration for Tessera</CardDescription>
				</CardHeader>
				<CardContent class="space-y-4">
					<div class="grid gap-4 sm:grid-cols-2">
						<div class="space-y-2">
							<Label for="author">Default Author</Label>
							<Input
								id="author"
								placeholder="Your name"
								bind:value={generalForm.author}
							/>
							<p class="text-xs text-muted-foreground">Used as the default author for new entities</p>
						</div>
						<div class="space-y-2">
							<Label for="editor">Editor Command</Label>
							<Input
								id="editor"
								placeholder="e.g., code --wait, vim, nano"
								bind:value={generalForm.editor}
							/>
							<p class="text-xs text-muted-foreground">Command to open files for editing</p>
						</div>
						<div class="space-y-2">
							<Label for="pager">Pager Command</Label>
							<Input
								id="pager"
								placeholder="e.g., less, bat"
								bind:value={generalForm.pager}
							/>
							<p class="text-xs text-muted-foreground">Command for viewing long output</p>
						</div>
						<div class="space-y-2">
							<Label for="default_format">Default Output Format</Label>
							<select
								id="default_format"
								class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
								bind:value={generalForm.default_format}
							>
								<option value={null}>Auto</option>
								<option value="table">Table</option>
								<option value="yaml">YAML</option>
								<option value="json">JSON</option>
								<option value="csv">CSV</option>
							</select>
							<p class="text-xs text-muted-foreground">Default format for list output</p>
						</div>
					</div>
				</CardContent>
				<CardFooter class="flex justify-between">
					<div class="text-sm text-muted-foreground">
						{#if allSettings?.config_paths.global_config}
							<Globe class="h-4 w-4 inline mr-1" />
							Global: {allSettings.config_paths.global_config}
						{/if}
					</div>
					<div class="flex gap-2">
						<Button variant="outline" onclick={() => saveGeneralSettings(true)} disabled={saving}>
							<Globe class="h-4 w-4 mr-2" />
							Save to Global
						</Button>
						<Button onclick={() => saveGeneralSettings(false)} disabled={saving}>
							<Save class="h-4 w-4 mr-2" />
							Save to Project
						</Button>
					</div>
				</CardFooter>
			</Card>
		{/if}

		<!-- Workflow Settings Tab -->
		{#if activeTab === 'workflow'}
			<Card>
				<CardHeader>
					<CardTitle>Workflow Settings</CardTitle>
					<CardDescription>Configure git-based approval workflows</CardDescription>
				</CardHeader>
				<CardContent class="space-y-6">
					<div class="flex items-center justify-between">
						<div>
							<Label>Enable Workflow Features</Label>
							<p class="text-xs text-muted-foreground">
								Enable git branch-based approval workflow
							</p>
						</div>
						<button
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {workflowForm.enabled
								? 'bg-primary'
								: 'bg-muted'}"
							onclick={() => (workflowForm.enabled = !workflowForm.enabled)}
							aria-label="Toggle workflow features"
						>
							<span
								class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {workflowForm.enabled
									? 'translate-x-6'
									: 'translate-x-1'}"
							></span>
						</button>
					</div>

					{#if workflowForm.enabled}
						<Separator />
						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="provider">Git Provider</Label>
								<select
									id="provider"
									class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
									bind:value={workflowForm.provider}
								>
									<option value="none">None</option>
									<option value="github">GitHub</option>
									<option value="gitlab">GitLab</option>
								</select>
							</div>
							<div class="space-y-2">
								<Label for="base_branch">Base Branch</Label>
								<Input
									id="base_branch"
									placeholder="main"
									bind:value={workflowForm.base_branch}
								/>
							</div>
							<div class="space-y-2">
								<Label for="branch_pattern">Branch Pattern</Label>
								<Input
									id="branch_pattern"
									placeholder={'review/{prefix}-{short_id}'}
									bind:value={workflowForm.branch_pattern}
								/>
								<p class="text-xs text-muted-foreground">
									Available: {'{prefix}'}, {'{short_id}'}
								</p>
							</div>
							<div class="space-y-2">
								<Label for="submit_message">Submit Message</Label>
								<Input
									id="submit_message"
									placeholder={'Submit {id}: {title}'}
									bind:value={workflowForm.submit_message}
								/>
							</div>
							<div class="space-y-2">
								<Label for="approve_message">Approve Message</Label>
								<Input
									id="approve_message"
									placeholder={'Approve {id}: {title}'}
									bind:value={workflowForm.approve_message}
								/>
							</div>
						</div>

						<Separator />

						<div class="space-y-4">
							<h4 class="font-medium">Workflow Options</h4>
							<div class="grid gap-4 sm:grid-cols-2">
								<label class="flex items-center gap-3 cursor-pointer">
									<input
										type="checkbox"
										class="h-4 w-4 rounded border-input"
										bind:checked={workflowForm.require_branch}
									/>
									<div>
										<div class="text-sm font-medium">Require Feature Branch</div>
										<div class="text-xs text-muted-foreground">
											Require separate branch for submit
										</div>
									</div>
								</label>
								<label class="flex items-center gap-3 cursor-pointer">
									<input
										type="checkbox"
										class="h-4 w-4 rounded border-input"
										bind:checked={workflowForm.auto_commit}
									/>
									<div>
										<div class="text-sm font-medium">Auto-commit</div>
										<div class="text-xs text-muted-foreground">
											Automatically commit on status change
										</div>
									</div>
								</label>
								<label class="flex items-center gap-3 cursor-pointer">
									<input
										type="checkbox"
										class="h-4 w-4 rounded border-input"
										bind:checked={workflowForm.auto_merge}
									/>
									<div>
										<div class="text-sm font-medium">Auto-merge</div>
										<div class="text-xs text-muted-foreground">
											Merge PR automatically on approve
										</div>
									</div>
								</label>
							</div>
						</div>
					{/if}
				</CardContent>
				<CardFooter class="flex justify-end">
					<Button onclick={saveWorkflowSettings} disabled={saving}>
						<Save class="h-4 w-4 mr-2" />
						Save Workflow Settings
					</Button>
				</CardFooter>
			</Card>
		{/if}

		<!-- Manufacturing Settings Tab -->
		{#if activeTab === 'manufacturing'}
			<Card>
				<CardHeader>
					<CardTitle>Manufacturing Settings</CardTitle>
					<CardDescription>Configure lot tracking and manufacturing workflows</CardDescription>
				</CardHeader>
				<CardContent class="space-y-6">
					<div class="flex items-center justify-between">
						<div>
							<Label>Enable Lot Branches</Label>
							<p class="text-xs text-muted-foreground">
								Create git branches for lot tracking
							</p>
						</div>
						<button
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {manufacturingForm.lot_branch_enabled
								? 'bg-primary'
								: 'bg-muted'}"
							onclick={() =>
								(manufacturingForm.lot_branch_enabled = !manufacturingForm.lot_branch_enabled)}
							aria-label="Toggle lot branches"
						>
							<span
								class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {manufacturingForm.lot_branch_enabled
									? 'translate-x-6'
									: 'translate-x-1'}"
							></span>
						</button>
					</div>

					{#if manufacturingForm.lot_branch_enabled}
						<Separator />
						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="mfg_base_branch">Base Branch</Label>
								<Input
									id="mfg_base_branch"
									placeholder="main"
									bind:value={manufacturingForm.base_branch}
								/>
							</div>
							<div class="space-y-2">
								<Label for="mfg_branch_pattern">Branch Pattern</Label>
								<Input
									id="mfg_branch_pattern"
									placeholder={'lot/{lot_number}'}
									bind:value={manufacturingForm.branch_pattern}
								/>
								<p class="text-xs text-muted-foreground">Available: {'{lot_number}'}</p>
							</div>
						</div>
					{/if}

					<Separator />

					<div class="space-y-4">
						<h4 class="font-medium">Git Options</h4>
						<div class="grid gap-4 sm:grid-cols-2">
							<label class="flex items-center gap-3 cursor-pointer">
								<input
									type="checkbox"
									class="h-4 w-4 rounded border-input"
									bind:checked={manufacturingForm.create_tags}
								/>
								<div>
									<div class="text-sm font-medium">Create Tags</div>
									<div class="text-xs text-muted-foreground">
										Create git tags at lot lifecycle events
									</div>
								</div>
							</label>
							<label class="flex items-center gap-3 cursor-pointer">
								<input
									type="checkbox"
									class="h-4 w-4 rounded border-input"
									bind:checked={manufacturingForm.sign_commits}
								/>
								<div>
									<div class="text-sm font-medium">Sign Commits</div>
									<div class="text-xs text-muted-foreground">
										GPG sign commits for compliance
									</div>
								</div>
							</label>
						</div>
					</div>
				</CardContent>
				<CardFooter class="flex justify-end">
					<Button onclick={saveManufacturingSettings} disabled={saving}>
						<Save class="h-4 w-4 mr-2" />
						Save Manufacturing Settings
					</Button>
				</CardFooter>
			</Card>
		{/if}

		<!-- Team Roster Tab -->
		{#if activeTab === 'team'}
			{#if !teamRoster}
				<Card>
					<CardContent class="flex flex-col items-center justify-center py-12">
						<Users class="h-12 w-12 text-muted-foreground/50 mb-4" />
						<p class="text-muted-foreground mb-4">Team roster not initialized</p>
						<Button onclick={initTeamRoster}>
							<Plus class="h-4 w-4 mr-2" />
							Initialize Team Roster
						</Button>
					</CardContent>
				</Card>
			{:else}
				<!-- Team Members -->
				<Card>
					<CardHeader class="flex flex-row items-center justify-between">
						<div>
							<CardTitle>Team Members</CardTitle>
							<CardDescription>Manage team members and their roles</CardDescription>
						</div>
						{#if !showNewMemberForm}
							<Button
								variant="outline"
								size="sm"
								onclick={() => {
									showNewMemberForm = true;
									prefillNewMemberFromGit();
								}}
							>
								<Plus class="h-4 w-4 mr-2" />
								Add Member
							</Button>
						{/if}
					</CardHeader>
					<CardContent>
						{#if showNewMemberForm}
							<div class="border rounded-lg p-4 mb-4 bg-muted/50">
								<h4 class="font-medium mb-4">New Team Member</h4>
								<div class="grid gap-4 sm:grid-cols-2">
									<div class="space-y-2">
										<Label for="new_name">Name</Label>
										<Input id="new_name" placeholder="Full name" bind:value={newMember.name} />
									</div>
									<div class="space-y-2">
										<Label for="new_email">Email</Label>
										<Input
											id="new_email"
											type="email"
											placeholder="email@example.com"
											bind:value={newMember.email}
										/>
									</div>
									<div class="space-y-2">
										<Label for="new_username">Username</Label>
										<Input
											id="new_username"
											placeholder="Git username"
											bind:value={newMember.username}
										/>
									</div>
									<div class="space-y-2">
										<Label for="new_signing">Signing Format</Label>
										<select
											id="new_signing"
											class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
											bind:value={newMember.signing_format}
										>
											<option value={null}>None</option>
											{#each availableSigningFormats as format}
												<option value={format}>{format.toUpperCase()}</option>
											{/each}
										</select>
									</div>
								</div>
								<div class="mt-4">
									<Label>Roles</Label>
									<div class="flex flex-wrap gap-2 mt-2">
										{#each availableRoles as role}
											<button
												class="px-3 py-1 rounded-full text-sm border transition-colors {newMember.roles.includes(
													role
												)
													? 'bg-primary text-primary-foreground border-primary'
													: 'bg-background hover:bg-accent'}"
												onclick={() => toggleRole(newMember, role)}
											>
												{role}
											</button>
										{/each}
									</div>
								</div>
								<div class="flex justify-end gap-2 mt-4">
									<Button
										variant="outline"
										onclick={() => {
											showNewMemberForm = false;
											newMember = {
												name: '',
												email: '',
												username: '',
												roles: [],
												active: true,
												signing_format: null
											};
										}}
									>
										Cancel
									</Button>
									<Button onclick={addTeamMember}>
										<Plus class="h-4 w-4 mr-2" />
										Add Member
									</Button>
								</div>
							</div>
						{/if}

						{#if teamRoster.members.length === 0}
							<p class="text-muted-foreground text-center py-8">No team members yet</p>
						{:else}
							<div class="space-y-3">
								{#each teamRoster.members as member}
									<div
										class="border rounded-lg p-4 {member.active
											? ''
											: 'opacity-50 bg-muted/30'}"
									>
										{#if editingMember === member.username && editMemberForm}
											<!-- Edit form -->
											<div class="grid gap-4 sm:grid-cols-2">
												<div class="space-y-2">
													<Label>Name</Label>
													<Input bind:value={editMemberForm.name} />
												</div>
												<div class="space-y-2">
													<Label>Email</Label>
													<Input type="email" bind:value={editMemberForm.email} />
												</div>
												<div class="space-y-2">
													<Label>Username</Label>
													<Input bind:value={editMemberForm.username} />
												</div>
												<div class="space-y-2">
													<Label>Signing Format</Label>
													<select
														class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
														bind:value={editMemberForm.signing_format}
													>
														<option value={null}>None</option>
														{#each availableSigningFormats as format}
															<option value={format}>{format.toUpperCase()}</option>
														{/each}
													</select>
												</div>
											</div>
											<div class="mt-4">
												<Label>Roles</Label>
												<div class="flex flex-wrap gap-2 mt-2">
													{#each availableRoles as role}
														<button
															class="px-3 py-1 rounded-full text-sm border transition-colors {editMemberForm?.roles.includes(
																role
															)
																? 'bg-primary text-primary-foreground border-primary'
																: 'bg-background hover:bg-accent'}"
															onclick={() => editMemberForm && toggleRole(editMemberForm, role)}
														>
															{role}
														</button>
													{/each}
												</div>
											</div>
											<div class="flex justify-end gap-2 mt-4">
												<Button variant="outline" onclick={cancelEditMember}>Cancel</Button>
												<Button onclick={updateTeamMember}>Save Changes</Button>
											</div>
										{:else}
											<!-- Display member -->
											<div class="flex items-start justify-between">
												<div>
													<div class="flex items-center gap-2">
														<span class="font-medium">{member.name}</span>
														{#if !member.active}
															<Badge variant="outline">Inactive</Badge>
														{/if}
													</div>
													<p class="text-sm text-muted-foreground">{member.email}</p>
													<p class="text-xs text-muted-foreground">@{member.username}</p>
													<div class="flex flex-wrap gap-1 mt-2">
														{#each member.roles as role}
															<Badge variant="secondary">{role}</Badge>
														{/each}
													</div>
													{#if member.signing_format}
														<p class="text-xs text-muted-foreground mt-1">
															Signing: {member.signing_format.toUpperCase()}
														</p>
													{/if}
												</div>
												<div class="flex gap-1">
													<Button
														variant="ghost"
														size="sm"
														onclick={() => toggleMemberActive(member.username, member.active)}
														title={member.active ? 'Deactivate' : 'Activate'}
													>
														{#if member.active}
															<X class="h-4 w-4" />
														{:else}
															<Check class="h-4 w-4" />
														{/if}
													</Button>
													<Button
														variant="ghost"
														size="sm"
														onclick={() => startEditMember(member)}
													>
														Edit
													</Button>
													<Button
														variant="ghost"
														size="sm"
														class="text-destructive hover:text-destructive"
														onclick={() => removeTeamMember(member.username)}
													>
														<Trash2 class="h-4 w-4" />
													</Button>
												</div>
											</div>
										{/if}
									</div>
								{/each}
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Approval Matrix -->
				<Card>
					<CardHeader class="flex flex-row items-center justify-between">
						<div>
							<CardTitle class="flex items-center gap-2">
								<Shield class="h-5 w-5" />
								Approval Matrix
							</CardTitle>
							<CardDescription>
								Define which roles can approve each entity type
							</CardDescription>
						</div>
						{#if !showNewApprovalEntry}
							<Button variant="outline" size="sm" onclick={() => (showNewApprovalEntry = true)}>
								<Plus class="h-4 w-4 mr-2" />
								Add Entry
							</Button>
						{/if}
					</CardHeader>
					<CardContent>
						{#if showNewApprovalEntry}
							<div class="border rounded-lg p-4 mb-4 bg-muted/50">
								<h4 class="font-medium mb-4">New Approval Entry</h4>
								<div class="space-y-4">
									<div class="space-y-2">
										<Label>Entity Type</Label>
										<select
											class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
											bind:value={newApprovalPrefix}
										>
											<option value="">Select entity type...</option>
											{#each entityPrefixes.filter((p) => !teamRoster?.approval_matrix[p.prefix]) as prefix}
												<option value={prefix.prefix}>{prefix.name} ({prefix.prefix})</option>
											{/each}
										</select>
									</div>
									<div class="space-y-2">
										<Label>Required Roles</Label>
										<div class="flex flex-wrap gap-2">
											{#each availableRoles as role}
												<button
													class="px-3 py-1 rounded-full text-sm border transition-colors {newApprovalRoles.includes(
														role
													)
														? 'bg-primary text-primary-foreground border-primary'
														: 'bg-background hover:bg-accent'}"
													onclick={() => {
														if (newApprovalRoles.includes(role)) {
															newApprovalRoles = newApprovalRoles.filter((r) => r !== role);
														} else {
															newApprovalRoles = [...newApprovalRoles, role];
														}
													}}
												>
													{role}
												</button>
											{/each}
										</div>
									</div>
								</div>
								<div class="flex justify-end gap-2 mt-4">
									<Button
										variant="outline"
										onclick={() => {
											showNewApprovalEntry = false;
											newApprovalPrefix = '';
											newApprovalRoles = [];
										}}
									>
										Cancel
									</Button>
									<Button
										onclick={async () => {
											if (newApprovalPrefix && newApprovalRoles.length > 0) {
												await updateApprovalMatrix(newApprovalPrefix, newApprovalRoles);
												showNewApprovalEntry = false;
												newApprovalPrefix = '';
												newApprovalRoles = [];
											}
										}}
										disabled={!newApprovalPrefix || newApprovalRoles.length === 0}
									>
										Add Entry
									</Button>
								</div>
							</div>
						{/if}

						{#if Object.keys(teamRoster.approval_matrix).length === 0}
							<p class="text-muted-foreground text-center py-8">
								No approval rules defined. Any team member can approve entities.
							</p>
						{:else}
							<div class="space-y-3">
								{#each Object.entries(teamRoster.approval_matrix) as [prefix, roles]}
									{@const prefixInfo = entityPrefixes.find((p) => p.prefix === prefix)}
									<div class="flex items-center justify-between border rounded-lg p-4">
										<div>
											<span class="font-medium">
												{prefixInfo?.name ?? prefix}
											</span>
											<span class="text-muted-foreground ml-2">({prefix})</span>
											<div class="flex flex-wrap gap-1 mt-2">
												{#each roles as role}
													<Badge variant="secondary">{role}</Badge>
												{/each}
											</div>
										</div>
										<Button
											variant="ghost"
											size="sm"
											class="text-destructive hover:text-destructive"
											onclick={() => removeApprovalEntry(prefix)}
										>
											<Trash2 class="h-4 w-4" />
										</Button>
									</div>
								{/each}
							</div>
						{/if}
					</CardContent>
				</Card>
			{/if}
		{/if}
	{/if}
</div>
