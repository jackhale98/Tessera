/**
 * Tauri API wrapper for type-safe IPC calls
 */

import { invoke } from '@tauri-apps/api/core';
import type {
	ProjectInfo,
	EntityData,
	EntityListResult,
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

// Risk-specific types from backend
export interface RiskSummary {
	id: string;
	title: string;
	risk_type: string;
	failure_mode: string;
	severity?: number;
	occurrence?: number;
	detection?: number;
	rpn?: number;
	risk_level?: string;
	status: string;
	author: string;
	created: string;
	tags: string[];
	mitigation_count: number;
}

export interface ListRisksResult {
	items: RiskSummary[];
	total_count: number;
	has_more: boolean;
}

export interface RiskStats {
	total: number;
	by_level: Record<string, number>;
	by_type: Record<string, number>;
	by_status: Record<string, number>;
	average_rpn?: number;
	high_priority_count: number;
	unmitigated_count: number;
}

// Component-specific types
export interface ComponentSummary {
	id: string;
	title: string;
	part_number: string;
	revision?: string;
	category: string;
	make_buy: string;
	unit_cost?: number;
	mass_kg?: number;
	status: string;
	author: string;
	created: string;
	tags: string[];
}

export interface ListComponentsResult {
	items: ComponentSummary[];
	total_count: number;
	has_more: boolean;
}

export interface ComponentStats {
	total: number;
	by_category: Record<string, number>;
	by_status: Record<string, number>;
	make_count: number;
	buy_count: number;
	total_cost?: number;
	total_mass?: number;
}

export interface BomCostSummary {
	total_cost: number;
	by_category: Record<string, number>;
	by_make_buy: Record<string, number>;
	item_count: number;
}

// Requirement stats
export interface RequirementStats {
	total: number;
	inputs: number;
	outputs: number;
	unverified: number;
	orphaned: number;
	by_status: StatusCounts;
}

export interface StatusCounts {
	draft: number;
	review: number;
	approved: number;
	released: number;
	obsolete: number;
}

// Link info
export interface LinkInfo {
	source_id: string;
	target_id: string;
	link_type: string;
	target_title?: string;
	target_status?: string;
}

/**
 * Project management API
 */
export const project = {
	open: (path: string) => call<ProjectInfo>('open_project', { path }),
	init: (path: string) => call<ProjectInfo>('init_project', { path }),
	close: () => call<void>('close_project'),
	getInfo: () => call<ProjectInfo | null>('get_project_info'),
	refresh: () => call<ProjectInfo>('refresh_project')
};

/**
 * Generic entity API (works with all entity types)
 */
export const entities = {
	list: (entityType: string, params?: ListParams) =>
		call<EntityListResult>('list_entities', {
			params: { entity_type: entityType, ...params }
		}),
	get: async (id: string) => {
		console.log('[API] entities.get called with id:', id);
		const startTime = Date.now();
		try {
			// Add timeout to detect hangs
			const timeoutPromise = new Promise<never>((_, reject) => {
				setTimeout(() => reject(new Error(`Timeout after 10s for entity ${id}`)), 10000);
			});
			const result = await Promise.race([
				call<EntityData | null>('get_entity', { id }),
				timeoutPromise
			]);
			console.log('[API] entities.get returned in', Date.now() - startTime, 'ms:', result?.title ?? 'null');
			return result;
		} catch (e) {
			console.error('[API] entities.get failed for id:', id, 'after', Date.now() - startTime, 'ms:', e);
			throw e;
		}
	},
	save: (entityType: string, data: Record<string, unknown>) =>
		call<string>('save_entity', { entityType, data }),
	delete: (id: string) => call<void>('delete_entity', { id }),
	getCount: (entityType: string) => call<number>('get_entity_count', { entityType }),
	getAllCounts: () => call<Record<string, number>>('get_all_entity_counts')
};

/**
 * Requirements API
 */
export interface ListRequirementsParams {
	status?: string[];
	req_type?: string;
	level?: string;
	priority?: string;
	category?: string;
	orphans_only?: boolean;
	unverified_only?: boolean;
	needs_review?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface RequirementSummary {
	id: string;
	title: string;
	req_type: string;
	level: string;
	priority: string;
	category?: string;
	status: string;
	author: string;
	created: string;
	tags: string[];
}

export interface ListRequirementsResult {
	items: RequirementSummary[];
	total_count: number;
}

export const requirements = {
	list: (params?: ListRequirementsParams) =>
		call<ListRequirementsResult>('list_requirements', { params }),
	get: (id: string) => call<unknown>('get_requirement', { id }),
	getStats: () => call<RequirementStats>('get_requirement_stats')
};

/**
 * Risks API (specialized commands + CRUD)
 */
export interface ListRisksParams {
	status?: string[];
	priority?: string[];
	risk_type?: string;
	risk_level?: string;
	search?: string;
	tags?: string[];
	min_rpn?: number;
	unmitigated_only?: boolean;
	limit?: number;
	offset?: number;
	sort_by?: string;
	sort_desc?: boolean;
}

export interface CreateRiskInput {
	title: string;
	description: string;
	author: string;
	risk_type?: string;
	category?: string;
	failure_mode?: string;
	cause?: string;
	effect?: string;
	severity?: number;
	occurrence?: number;
	detection?: number;
	tags?: string[];
}

export interface UpdateRiskInput {
	title?: string;
	description?: string;
	risk_type?: string;
	status?: string;
	category?: string;
	failure_mode?: string;
	cause?: string;
	effect?: string;
	severity?: number;
	occurrence?: number;
	detection?: number;
	tags?: string[];
}

export interface AddMitigationInput {
	action: string;
	mitigation_type?: string;
	owner?: string;
	due_date?: string;
}

export const risks = {
	list: (params?: ListRisksParams) => call<ListRisksResult>('list_risks', { params }),
	get: (id: string) => call<unknown>('get_risk', { id }),
	create: (input: CreateRiskInput) => call<unknown>('create_risk', { input }),
	update: (id: string, input: UpdateRiskInput) => call<unknown>('update_risk', { id, input }),
	delete: (id: string) => call<void>('delete_risk', { id }),
	addMitigation: (id: string, input: AddMitigationInput) =>
		call<unknown>('add_risk_mitigation', { id, input }),
	getStats: () => call<RiskStats>('get_risk_stats'),
	getMatrix: () => call<RiskMatrix>('get_risk_matrix')
};

/**
 * Components API (specialized commands + CRUD)
 */
export interface ListComponentsParams {
	status?: string[];
	category?: string;
	make_buy?: string;
	search?: string;
	tags?: string[];
	limit?: number;
	offset?: number;
	sort_by?: string;
	sort_desc?: boolean;
}

export interface CreateComponentInput {
	title: string;
	part_number: string;
	author: string;
	revision?: string;
	description?: string;
	category?: string;
	make_buy?: string;
	unit_cost?: number;
	mass_kg?: number;
	material?: string;
	tags?: string[];
}

export interface UpdateComponentInput {
	title?: string;
	part_number?: string;
	revision?: string;
	description?: string;
	category?: string;
	make_buy?: string;
	unit_cost?: number;
	mass_kg?: number;
	material?: string;
	status?: string;
}

export const components = {
	list: (params?: ListComponentsParams) => call<ListComponentsResult>('list_components', { params }),
	get: (id: string) => call<unknown>('get_component', { id }),
	getByPartNumber: (partNumber: string) =>
		call<unknown>('get_component_by_part_number', { partNumber }),
	create: (input: CreateComponentInput) => call<unknown>('create_component', { input }),
	update: (id: string, input: UpdateComponentInput) =>
		call<unknown>('update_component', { id, input }),
	delete: (id: string) => call<void>('delete_component', { id }),
	getStats: () => call<ComponentStats>('get_component_stats'),
	getBomCostSummary: () => call<BomCostSummary>('get_bom_cost_summary')
};

/**
 * Assemblies API (specialized commands + CRUD)
 */
export interface ListAssembliesParams {
	status?: string[];
	part_number?: string;
	empty_bom?: boolean;
	has_subassemblies?: boolean;
	top_level_only?: boolean;
	sub_only?: boolean;
	search?: string;
	tags?: string[];
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
	offset?: number;
}

export interface AssemblySummary {
	id: string;
	part_number: string;
	title: string;
	bom_count: number;
	subassembly_count: number;
	status: string;
	author: string;
	created: string;
}

export interface ListAssembliesResult {
	items: AssemblySummary[];
	total_count: number;
	has_more: boolean;
}

export interface AssemblyStats {
	total: number;
	top_level: number;
	sub_assemblies: number;
	empty_bom: number;
	total_bom_items: number;
	by_status: {
		draft: number;
		review: number;
		approved: number;
		released: number;
		obsolete: number;
	};
}

export interface BomNode {
	id: string;
	title: string;
	part_number: string;
	is_assembly: boolean;
	quantity: number;
	unit_cost?: number;
	mass_kg?: number;
	extended_cost?: number;
	extended_mass?: number;
	children: BomNode[];
}

export interface BomCostResult {
	total_cost: number;
	total_nre: number;
	components_with_cost: number;
	components_without_cost: number;
	missing_cost: string[];
}

export interface BomMassResult {
	total_mass_kg: number;
	components_with_mass: number;
	components_without_mass: number;
	missing_mass: string[];
}

export const assemblies = {
	list: (params?: ListAssembliesParams) => call<ListAssembliesResult>('list_assemblies', { params }),
	get: (id: string) => call<unknown>('get_assembly', { id }),
	getByPartNumber: (partNumber: string) =>
		call<unknown>('get_assembly_by_part_number', { partNumber }),
	getStats: () => call<AssemblyStats>('get_assembly_stats'),
	getBomTree: (id: string, quantity?: number) => call<BomNode>('get_bom_tree', { id, quantity }),
	calculateCost: (id: string, quantity?: number) =>
		call<BomCostResult>('calculate_assembly_cost', { id, quantity }),
	calculateMass: (id: string, quantity?: number) =>
		call<BomMassResult>('calculate_assembly_mass', { id, quantity }),
	addComponent: (assemblyId: string, componentId: string, quantity: number) =>
		call<unknown>('add_assembly_component', { assemblyId, componentId, quantity }),
	removeComponent: (assemblyId: string, componentId: string) =>
		call<unknown>('remove_assembly_component', { assemblyId, componentId }),
	updateComponentQuantity: (assemblyId: string, componentId: string, quantity: number) =>
		call<unknown>('update_assembly_component_quantity', { assemblyId, componentId, quantity })
};

/**
 * Traceability API
 */
export interface TraceParams {
	id: string;
	direction?: string;
	depth?: number;
	link_types?: string[];
}

export const traceability = {
	getLinksFrom: (id: string) => call<LinkInfo[]>('get_links_from', { id }),
	getLinksTo: (id: string) => call<LinkInfo[]>('get_links_to', { id }),
	traceFrom: (params: TraceParams) => call<TraceResult>('trace_from', { params }),
	traceTo: (params: TraceParams) => call<TraceResult>('trace_to', { params }),
	getCoverage: () => call<CoverageReport>('get_coverage_report'),
	getDsm: (entityType?: string) => call<unknown>('get_dsm', { entity_type: entityType }),
	findOrphans: (entityType?: string) => call<string[]>('find_orphans', { entity_type: entityType }),
	findCycles: (entityType?: string) =>
		call<string[][]>('find_cycles', { entity_type: entityType }),
	addLink: (sourceId: string, targetId: string, linkType?: string) =>
		call<void>('add_link', { sourceId, targetId, linkType }),
	removeLink: (sourceId: string, targetId: string, linkType?: string) =>
		call<void>('remove_link', { sourceId, targetId, linkType }),
	getLinkTypes: () => call<string[]>('get_link_types')
};

/**
 * Deviations API (specialized commands + CRUD)
 */
export interface ListDeviationsParams {
	status?: string[];
	dev_status?: string;
	deviation_type?: string;
	category?: string;
	risk_level?: string;
	active_only?: boolean;
	recent_days?: number;
	search?: string;
	tags?: string[];
	limit?: number;
	offset?: number;
	sort_by?: string;
	sort_desc?: boolean;
}

export interface DeviationSummary {
	id: string;
	title: string;
	deviation_number?: string;
	deviation_type: string;
	category: string;
	risk_level: string;
	dev_status: string;
	status: string;
	effective_date?: string;
	expiration_date?: string;
	approved_by?: string;
	approval_date?: string;
	author: string;
	created: string;
}

export interface ListDeviationsResult {
	items: DeviationSummary[];
	total_count: number;
	has_more: boolean;
}

export interface CreateDeviationInput {
	title: string;
	deviation_number?: string;
	deviation_type?: string;
	category?: string;
	description?: string;
	risk_level?: string;
	risk_assessment?: string;
	effective_date?: string;
	expiration_date?: string;
	notes?: string;
	author: string;
}

export interface UpdateDeviationInput {
	title?: string;
	deviation_number?: string;
	deviation_type?: string;
	category?: string;
	description?: string;
	effective_date?: string;
	expiration_date?: string;
	notes?: string;
	status?: string;
	dev_status?: string;
}

export interface ApproveDeviationInput {
	approved_by: string;
	authorization_level: string;
	activate?: boolean;
}

export interface RejectDeviationInput {
	reason?: string;
}

export interface DeviationStats {
	total: number;
	by_dev_status: {
		pending: number;
		approved: number;
		active: number;
		expired: number;
		closed: number;
		rejected: number;
	};
	by_type: {
		temporary: number;
		permanent: number;
		emergency: number;
	};
	by_category: {
		material: number;
		process: number;
		equipment: number;
		tooling: number;
		specification: number;
		documentation: number;
	};
	by_risk: {
		low: number;
		medium: number;
		high: number;
	};
	active: number;
}

export const deviations = {
	list: (params?: ListDeviationsParams) =>
		call<ListDeviationsResult>('list_deviations', { params }),
	get: (id: string) => call<unknown>('get_deviation', { id }),
	create: (input: CreateDeviationInput) => call<unknown>('create_deviation', { input }),
	update: (id: string, input: UpdateDeviationInput) =>
		call<unknown>('update_deviation', { id, input }),
	delete: (id: string) => call<void>('delete_deviation', { id }),
	approve: (id: string, input: ApproveDeviationInput) =>
		call<unknown>('approve_deviation', { id, input }),
	reject: (id: string, input?: RejectDeviationInput) =>
		call<unknown>('reject_deviation', { id, input }),
	activate: (id: string) => call<unknown>('activate_deviation', { id }),
	close: (id: string, reason?: string) => call<unknown>('close_deviation', { id, reason }),
	expire: (id: string) => call<unknown>('expire_deviation', { id }),
	addMitigation: (id: string, mitigation: string) =>
		call<unknown>('add_deviation_mitigation', { id, mitigation }),
	setRisk: (id: string, level: string, assessment?: string) =>
		call<unknown>('set_deviation_risk', { id, level, assessment }),
	addProcessLink: (id: string, processId: string) =>
		call<unknown>('add_deviation_process_link', { id, processId }),
	addLotLink: (id: string, lotId: string) =>
		call<unknown>('add_deviation_lot_link', { id, lotId }),
	addComponentLink: (id: string, componentId: string) =>
		call<unknown>('add_deviation_component_link', { id, componentId }),
	getStats: () => call<DeviationStats>('get_deviation_stats')
};

/**
 * Combined API namespace
 */
export const api = {
	project,
	entities,
	requirements,
	risks,
	components,
	assemblies,
	deviations,
	traceability
};

export default api;
