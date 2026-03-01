<script lang="ts">
	import { openProject, projectInfo } from '$lib/stores/project.js';
	import { goto } from '$app/navigation';
	import { FolderOpen, ChevronRight } from 'lucide-svelte';

	const isTauri = typeof window !== 'undefined' && !!(window as Record<string, unknown>).__TAURI_INTERNALS__;

	let loading = $state(false);
	let error = $state('');

	async function selectFolder() {
		loading = true;
		error = '';
		try {
			if (isTauri) {
				const { open } = await import('@tauri-apps/plugin-dialog');
				const selected = await open({ directory: true, multiple: false, title: 'Select TDT Project Folder' });
				if (selected && typeof selected === 'string') {
					await openProject(selected);
					goto('/');
				}
			} else {
				// Browser preview mode — open demo project
				await openProject('/demo/project');
				goto('/');
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}
</script>

<div class="project-screen">
	<div class="hero">
		<div class="logo-container">
			<div class="logo">
				<span class="logo-letter">T</span>
			</div>
		</div>
		<h1 class="hero-title">Tessera</h1>
		<p class="hero-subtitle">Engineering Artifact Management</p>
	</div>

	<div class="actions">
		<button class="open-btn" onclick={selectFolder} disabled={loading}>
			<div class="btn-icon">
				<FolderOpen size={24} />
			</div>
			<div class="btn-text">
				<span class="btn-label">Open Project</span>
				<span class="btn-desc">Select a TDT project folder</span>
			</div>
			<ChevronRight size={20} class="btn-chevron" />
		</button>

		{#if error}
			<div class="error-card">
				<p>{error}</p>
			</div>
		{/if}
	</div>

	<p class="footer-note">Projects are stored locally on your device. Sync via desktop app.</p>
</div>

<style>
	.project-screen {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 100%;
		padding: 32px 24px;
		gap: 48px;
	}

	.hero {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 16px;
	}

	.logo-container {
		margin-bottom: 8px;
	}

	.logo {
		width: 80px;
		height: 80px;
		border-radius: 24px;
		background: linear-gradient(135deg, var(--theme-primary), color-mix(in oklch, var(--theme-primary) 70%, var(--theme-destructive)));
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 8px 32px color-mix(in oklch, var(--theme-primary) 30%, transparent);
	}

	.logo-letter {
		font-size: 36px;
		font-weight: 800;
		color: white;
		letter-spacing: -0.02em;
	}

	.hero-title {
		font-size: 28px;
		font-weight: 800;
		letter-spacing: -0.02em;
	}

	.hero-subtitle {
		font-size: 15px;
		color: var(--theme-muted-foreground);
	}

	.actions {
		width: 100%;
		max-width: 360px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.open-btn {
		display: flex;
		align-items: center;
		gap: 14px;
		width: 100%;
		padding: 16px 20px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 16px;
		color: var(--theme-foreground);
		cursor: pointer;
		transition: all 0.15s ease;
		text-align: left;
	}

	.open-btn:active {
		transform: scale(0.98);
		background-color: var(--theme-accent);
	}

	.open-btn:disabled {
		opacity: 0.5;
	}

	.btn-icon {
		width: 44px;
		height: 44px;
		border-radius: 12px;
		background-color: var(--theme-primary);
		color: var(--theme-primary-foreground);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.btn-text {
		flex: 1;
		min-width: 0;
	}

	.btn-label {
		display: block;
		font-size: 16px;
		font-weight: 600;
	}

	.btn-desc {
		display: block;
		font-size: 13px;
		color: var(--theme-muted-foreground);
		margin-top: 2px;
	}

	:global(.btn-chevron) {
		color: var(--theme-muted-foreground);
		flex-shrink: 0;
	}

	.error-card {
		padding: 12px 16px;
		background-color: color-mix(in oklch, var(--theme-error) 15%, transparent);
		border: 1px solid var(--theme-error);
		border-radius: 12px;
		color: var(--theme-error);
		font-size: 13px;
	}

	.footer-note {
		font-size: 12px;
		color: var(--theme-muted-foreground);
		text-align: center;
		opacity: 0.6;
	}
</style>
