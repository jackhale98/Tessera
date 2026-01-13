/**
 * Tauri API wrapper for type-safe IPC calls
 */

import { invoke } from '@tauri-apps/api/core';
import type {
	ProjectInfo,
	Requirement,
	Risk,
	Component,
	Test,
	Result,
	ListParams,
	TraceResult,
	CoverageReport,
	RiskMatrix
} from './types.js';

/**
 * Type-safe invoke wrapper with error handling
 */
async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	try {
		return await invoke<T>(cmd, args);
	} catch (error) {
		console.error(`Tauri command failed: ${cmd}`, error);
		throw error;
	}
}

/**
 * Project management API
 */
export const project = {
	/**
	 * Open an existing TDT project
	 */
	open: (path: string) => call<ProjectInfo>('open_project', { path }),

	/**
	 * Initialize a new TDT project
	 */
	init: (path: string) => call<ProjectInfo>('init_project', { path }),

	/**
	 * Close the current project
	 */
	close: () => call<void>('close_project'),

	/**
	 * Get information about the current project
	 */
	getInfo: () => call<ProjectInfo | null>('get_project_info'),

	/**
	 * Refresh the project cache
	 */
	refresh: () => call<ProjectInfo>('refresh_project')
};

/**
 * Requirements API
 */
export const requirements = {
	list: (params?: ListParams) => call<Requirement[]>('list_requirements', { params }),
	get: (id: string) => call<Requirement | null>('get_requirement', { id }),
	create: (input: Partial<Requirement>) => call<Requirement>('create_requirement', { input }),
	update: (id: string, input: Partial<Requirement>) =>
		call<Requirement>('update_requirement', { id, input }),
	delete: (id: string) => call<void>('delete_requirement', { id })
};

/**
 * Risks API
 */
export const risks = {
	list: (params?: ListParams) => call<Risk[]>('list_risks', { params }),
	get: (id: string) => call<Risk | null>('get_risk', { id }),
	create: (input: Partial<Risk>) => call<Risk>('create_risk', { input }),
	update: (id: string, input: Partial<Risk>) => call<Risk>('update_risk', { id, input }),
	delete: (id: string) => call<void>('delete_risk', { id }),
	getMatrix: () => call<RiskMatrix>('get_risk_matrix')
};

/**
 * Components API
 */
export const components = {
	list: (params?: ListParams) => call<Component[]>('list_components', { params }),
	get: (id: string) => call<Component | null>('get_component', { id }),
	create: (input: Partial<Component>) => call<Component>('create_component', { input }),
	update: (id: string, input: Partial<Component>) =>
		call<Component>('update_component', { id, input }),
	delete: (id: string) => call<void>('delete_component', { id })
};

/**
 * Tests API
 */
export const tests = {
	list: (params?: ListParams) => call<Test[]>('list_tests', { params }),
	get: (id: string) => call<Test | null>('get_test', { id }),
	create: (input: Partial<Test>) => call<Test>('create_test', { input }),
	update: (id: string, input: Partial<Test>) => call<Test>('update_test', { id, input }),
	delete: (id: string) => call<void>('delete_test', { id })
};

/**
 * Results API
 */
export const results = {
	list: (params?: ListParams) => call<Result[]>('list_results', { params }),
	get: (id: string) => call<Result | null>('get_result', { id }),
	create: (input: Partial<Result>) => call<Result>('create_result', { input }),
	update: (id: string, input: Partial<Result>) => call<Result>('update_result', { id, input }),
	delete: (id: string) => call<void>('delete_result', { id })
};

/**
 * Traceability API
 */
export const traceability = {
	/**
	 * Trace from an entity (find what it links to)
	 */
	traceFrom: (id: string, depth?: number) =>
		call<TraceResult>('trace_from', { id, depth: depth ?? 1 }),

	/**
	 * Trace to an entity (find what links to it)
	 */
	traceTo: (id: string, depth?: number) => call<TraceResult>('trace_to', { id, depth: depth ?? 1 }),

	/**
	 * Get coverage report
	 */
	getCoverage: () => call<CoverageReport>('get_coverage')
};

/**
 * Combined API namespace
 */
export const api = {
	project,
	requirements,
	risks,
	components,
	tests,
	results,
	traceability
};

export default api;
