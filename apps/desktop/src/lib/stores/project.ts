/**
 * Project state store
 */

import { writable, derived } from 'svelte/store';
import { project as projectApi } from '$lib/api/tauri.js';
import type { ProjectInfo } from '$lib/api/types.js';

// Core state
export const projectInfo = writable<ProjectInfo | null>(null);
export const isLoading = writable(false);
export const error = writable<string | null>(null);

// Derived state
export const isProjectOpen = derived(projectInfo, ($info) => $info !== null);
export const projectName = derived(projectInfo, ($info) => $info?.name ?? '');
export const projectPath = derived(projectInfo, ($info) => $info?.path ?? '');
export const entityCounts = derived(projectInfo, ($info) => $info?.entity_counts ?? null);

export const totalEntities = derived(entityCounts, ($counts) => {
	if (!$counts) return 0;
	return (
		$counts.requirements +
		$counts.risks +
		$counts.tests +
		$counts.results +
		$counts.components +
		$counts.assemblies +
		$counts.features +
		$counts.mates +
		$counts.stackups +
		$counts.processes +
		$counts.controls +
		$counts.work_instructions +
		$counts.lots +
		$counts.deviations +
		$counts.ncrs +
		$counts.capas +
		$counts.quotes +
		$counts.suppliers
	);
});

/**
 * Open an existing project
 */
export async function openProject(path: string): Promise<ProjectInfo> {
	isLoading.set(true);
	error.set(null);

	try {
		const info = await projectApi.open(path);
		projectInfo.set(info);
		return info;
	} catch (e) {
		const message = e instanceof Error ? e.message : String(e);
		error.set(message);
		throw e;
	} finally {
		isLoading.set(false);
	}
}

/**
 * Initialize a new project
 */
export async function initProject(path: string): Promise<ProjectInfo> {
	isLoading.set(true);
	error.set(null);

	try {
		const info = await projectApi.init(path);
		projectInfo.set(info);
		return info;
	} catch (e) {
		const message = e instanceof Error ? e.message : String(e);
		error.set(message);
		throw e;
	} finally {
		isLoading.set(false);
	}
}

/**
 * Close the current project
 */
export async function closeProject(): Promise<void> {
	isLoading.set(true);
	error.set(null);

	try {
		await projectApi.close();
		projectInfo.set(null);
	} catch (e) {
		const message = e instanceof Error ? e.message : String(e);
		error.set(message);
		throw e;
	} finally {
		isLoading.set(false);
	}
}

/**
 * Refresh the project cache
 */
export async function refreshProject(): Promise<ProjectInfo> {
	isLoading.set(true);
	error.set(null);

	try {
		const info = await projectApi.refresh();
		projectInfo.set(info);
		return info;
	} catch (e) {
		const message = e instanceof Error ? e.message : String(e);
		error.set(message);
		throw e;
	} finally {
		isLoading.set(false);
	}
}

/**
 * Check and restore project state on app load
 */
export async function checkProjectState(): Promise<void> {
	try {
		const info = await projectApi.getInfo();
		projectInfo.set(info);
	} catch {
		// No project open, that's fine
		projectInfo.set(null);
	}
}
