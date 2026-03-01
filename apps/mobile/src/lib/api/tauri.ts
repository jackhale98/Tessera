/**
 * Tauri API wrapper for mobile — subset of desktop API
 * Excludes: version control, settings, mates, stackups, features, hazards, quotes, suppliers
 */

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
 * Check if we're running inside Tauri or in a plain browser
 */
function checkIsTauri(): boolean {
	try {
		return typeof window !== 'undefined' && !!(window as Record<string, unknown>).__TAURI_INTERNALS__;
	} catch {
		return false;
	}
}

/**
 * Type-safe invoke wrapper with error handling.
 * Falls back to mock data when running in a plain browser (no Tauri runtime).
 */
async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	if (!checkIsTauri()) {
		return getMockResponse<T>(cmd, args);
	}
	try {
		const { invoke } = await import('@tauri-apps/api/core');
		return await invoke<T>(cmd, args);
	} catch (error) {
		console.error(`Tauri command failed: ${cmd}`, error);
		throw error;
	}
}

// ============================================================================
// Mock data for browser preview mode
// ============================================================================

function getMockResponse<T>(cmd: string, args?: Record<string, unknown>): T {
	const mocks: Record<string, unknown> = {
		get_project_info: {
			name: 'Demo Project',
			path: '/demo/project',
			author: 'Demo User',
			created: new Date().toISOString(),
			entity_counts: { requirements: 12, risks: 8, hazards: 2, tests: 15, results: 10, components: 20, assemblies: 5, features: 3, mates: 2, stackups: 1, processes: 6, controls: 4, work_instructions: 3, lots: 7, deviations: 2, ncrs: 4, capas: 3, quotes: 1, suppliers: 2, actions: 0 }
		},
		open_project: {
			name: 'Demo Project', path: '/demo/project', author: 'Demo User',
			created: new Date().toISOString(),
			entity_counts: { requirements: 12, risks: 8, hazards: 2, tests: 15, results: 10, components: 20, assemblies: 5, features: 3, mates: 2, stackups: 1, processes: 6, controls: 4, work_instructions: 3, lots: 7, deviations: 2, ncrs: 4, capas: 3, quotes: 1, suppliers: 2, actions: 0 }
		},
		close_project: undefined,
		refresh_project: {
			name: 'Demo Project', path: '/demo/project', author: 'Demo User',
			created: new Date().toISOString(),
			entity_counts: { requirements: 12, risks: 8, hazards: 2, tests: 15, results: 10, components: 20, assemblies: 5, features: 3, mates: 2, stackups: 1, processes: 6, controls: 4, work_instructions: 3, lots: 7, deviations: 2, ncrs: 4, capas: 3, quotes: 1, suppliers: 2, actions: 0 }
		},
		get_all_entity_counts: { REQ: 12, RISK: 8, HAZ: 2, TEST: 15, RSLT: 10, CMP: 20, ASM: 5, FEAT: 3, MATE: 2, TOL: 1, PROC: 6, CTRL: 4, WORK: 3, LOT: 7, DEV: 2, NCR: 4, CAPA: 3, QUOT: 1, SUP: 2 },
		list_lots: {
			items: [
				{ id: 'LOT-01DEMO001', title: 'Production Lot A-2024', lot_number: 'LOT-001', quantity: 100, lot_status: 'in_progress', status: 'draft', start_date: '2024-12-01', author: 'J. Smith', created: '2024-11-28' },
				{ id: 'LOT-01DEMO002', title: 'Prototype Lot B', lot_number: 'LOT-002', quantity: 10, lot_status: 'in_progress', status: 'draft', author: 'A. Chen', created: '2024-12-05' },
				{ id: 'LOT-01DEMO003', title: 'QC Validation Batch', lot_number: 'LOT-003', quantity: 50, lot_status: 'on_hold', status: 'draft', author: 'M. Park', created: '2024-12-10' },
				{ id: 'LOT-01DEMO004', title: 'Final Assembly Run', lot_number: 'LOT-004', quantity: 200, lot_status: 'completed', status: 'approved', completion_date: '2024-12-20', author: 'J. Smith', created: '2024-11-15' }
			],
			total_count: 4, has_more: false
		},
		list_ncrs: {
			items: [
				{ id: 'NCR-01DEMO001', title: 'Dimensional out-of-spec on housing', ncr_number: 'NCR-2024-001', ncr_type: 'internal', severity: 'major', ncr_status: 'investigation', category: 'dimensional', status: 'draft', author: 'Q. Inspector', created: '2024-12-15' },
				{ id: 'NCR-01DEMO002', title: 'Surface finish defect on cover plate', ncr_number: 'NCR-2024-002', ncr_type: 'supplier', severity: 'minor', ncr_status: 'open', category: 'cosmetic', status: 'draft', author: 'R. Lead', created: '2024-12-18' },
				{ id: 'NCR-01DEMO003', title: 'Solder joint crack on PCB assembly', ncr_number: 'NCR-2024-003', ncr_type: 'internal', severity: 'critical', ncr_status: 'containment', category: 'workmanship', status: 'draft', author: 'E. Tech', created: '2024-12-20' }
			],
			total_count: 3, has_more: false
		},
		list_capas: {
			items: [
				{ id: 'CAPA-01DEMO001', title: 'Root cause: solder process parameters', capa_number: 'CAPA-2024-001', capa_type: 'corrective', capa_status: 'investigation', status: 'draft', due_date: '2025-01-15', author: 'M. Engineer', created: '2024-12-21' },
				{ id: 'CAPA-01DEMO002', title: 'Prevent dimensional drift on CNC ops', capa_number: 'CAPA-2024-002', capa_type: 'preventive', capa_status: 'implementation', status: 'draft', due_date: '2025-02-01', author: 'J. Smith', created: '2024-12-16' }
			],
			total_count: 2, has_more: false
		},
		list_deviations: {
			items: [
				{ id: 'DEV-01DEMO001', title: 'Temporary material substitution - AL6061 to AL7075', deviation_number: 'DEV-2024-001', deviation_type: 'temporary', category: 'material', risk_level: 'low', dev_status: 'active', status: 'approved', author: 'J. Smith', created: '2024-12-01' },
				{ id: 'DEV-01DEMO002', title: 'Emergency: alternate supplier for connector P/N 12345', deviation_number: 'DEV-2024-002', deviation_type: 'emergency', category: 'material', risk_level: 'medium', dev_status: 'pending', status: 'draft', author: 'A. Buyer', created: '2024-12-19' }
			],
			total_count: 2, has_more: false
		},
		get_ncr_stats: { total: 3, by_ncr_status: { open: 1, containment: 1, investigation: 1, disposition: 0, closed: 0 }, by_type: { internal: 2, supplier: 1, customer: 0 }, by_severity: { minor: 1, major: 1, critical: 1 }, open: 3, total_cost: 2500 },
		get_capa_stats: { total: 2, by_capa_status: { initiation: 0, investigation: 1, implementation: 1, verification: 0, closed: 0 }, by_type: { corrective: 1, preventive: 1 }, open: 2, overdue: 0, verified_effective: 0 },
		get_lot_stats: { total: 4, by_status: { in_progress: 2, on_hold: 1, completed: 1, scrapped: 0 }, total_quantity: 360, avg_quantity: 90, with_git_branch: 0, merged_branches: 0 },
		get_deviation_stats: { total: 2, by_dev_status: { pending: 1, approved: 0, active: 1, expired: 0, closed: 0, rejected: 0 }, by_type: { temporary: 1, permanent: 0, emergency: 1 }, by_category: { material: 2, process: 0, equipment: 0, tooling: 0, specification: 0, documentation: 0 }, by_risk: { low: 1, medium: 1, high: 0 }, active: 1 },
		get_lot_next_step: 0,
		list_entities: { items: [], total_count: 0, has_more: false },
		search_entities: [],
		get_entity: null,
		get_entity_count: 0,
		get_links_from: [],
		get_links_to: [],
		get_link_types: ['satisfied_by', 'verified_by', 'allocated_to', 'mitigated_by', 'derived_from'],
		sync_cache: undefined,
	};

	// Handle specific lot/ncr/capa get by returning first mock item
	if (cmd === 'get_lot') {
		const lotItems = (mocks['list_lots'] as { items: unknown[] }).items;
		const match = lotItems.find((l: unknown) => (l as Record<string, string>).id === (args?.id as string));
		return (match ?? { id: args?.id, title: 'Demo Lot', lot_status: 'in_progress', status: 'draft', quantity: 50, author: 'Demo User', created: new Date().toISOString(), steps: [
			{ process_title: 'Incoming Inspection', status: 'completed', operator: 'Q. Inspector' },
			{ process_title: 'CNC Machining', status: 'in_progress', operator: 'M. Operator' },
			{ process_title: 'Deburring & Finishing', status: 'pending' },
			{ process_title: 'Final QC', status: 'pending' },
			{ process_title: 'Packaging', status: 'pending' }
		] }) as T;
	}
	if (cmd === 'get_ncr') {
		return { id: args?.id, title: 'Demo NCR', ncr_status: 'investigation', ncr_type: 'internal', severity: 'major', category: 'dimensional', description: 'Housing bore diameter measured at 25.12mm, spec is 25.00 +/- 0.05mm. 3 of 10 parts affected.', author: 'Q. Inspector', created: new Date().toISOString() } as T;
	}
	if (cmd === 'get_capa') {
		return { id: args?.id, title: 'Demo CAPA', capa_status: 'investigation', capa_type: 'corrective', description: 'Investigating root cause of dimensional non-conformance on housing bore.', author: 'M. Engineer', created: new Date().toISOString(), due_date: '2025-02-01' } as T;
	}
	if (cmd === 'get_deviation') {
		return { id: args?.id, title: 'Demo Deviation', dev_status: 'active', deviation_type: 'temporary', category: 'material', risk_level: 'low', description: 'Temporary substitution of AL6061-T6 with AL7075-T6 for bracket P/N 10042.', author: 'J. Smith', created: new Date().toISOString() } as T;
	}
	if (cmd === 'trace_from' || cmd === 'trace_to') {
		return { nodes: [], edges: [] } as T;
	}
	if (cmd === 'get_coverage_report') {
		return { health_score: 72, total_entities: 95, linked_entities: 68, orphaned_entities: 27 } as T;
	}
	if (cmd === 'create_ncr' || cmd === 'create_deviation' || cmd === 'create_capa' || cmd === 'create_lot') {
		return 'DEMO-01NEWID001' as T;
	}

	if (cmd in mocks) {
		return mocks[cmd] as T;
	}

	console.warn(`[Mock] No mock for command: ${cmd}`);
	return undefined as T;
}

// ============================================================================
// Shared types (from desktop)
// ============================================================================

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
	control_count: number;
}

export interface ListRisksResult {
	items: RiskSummary[];
	total_count: number;
	has_more: boolean;
}

export interface RiskStats {
	total: number;
	unmitigated: number;
	unverified: number;
	by_type: { design: number; process: number; use: number; software: number };
	by_level: { low: number; medium: number; high: number; critical: number };
	by_status: { draft: number; review: number; approved: number; released: number; obsolete: number };
	rpn_stats: { count: number; min: number; max: number; sum: number; avg: number };
}

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

export interface LinkInfo {
	source_id: string;
	target_id: string;
	link_type: string;
	target_title?: string;
	target_status?: string;
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
	by_status: { draft: number; review: number; approved: number; released: number; obsolete: number };
}

// ============================================================================
// Deviation types
// ============================================================================

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

export interface DeviationStats {
	total: number;
	by_dev_status: { pending: number; approved: number; active: number; expired: number; closed: number; rejected: number };
	by_type: { temporary: number; permanent: number; emergency: number };
	by_category: { material: number; process: number; equipment: number; tooling: number; specification: number; documentation: number };
	by_risk: { low: number; medium: number; high: number };
	active: number;
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

// ============================================================================
// NCR types
// ============================================================================

export interface NcrSummary {
	id: string;
	title: string;
	ncr_number?: string;
	ncr_type: string;
	severity: string;
	ncr_status: string;
	category: string;
	status: string;
	author: string;
	created: string;
}

export interface ListNcrsResult {
	items: NcrSummary[];
	total_count: number;
	has_more: boolean;
}

export interface NcrStats {
	total: number;
	by_ncr_status: { open: number; containment: number; investigation: number; disposition: number; closed: number };
	by_type: { internal: number; supplier: number; customer: number };
	by_severity: { minor: number; major: number; critical: number };
	open: number;
	total_cost: number;
}

export interface CreateNcrInput {
	title: string;
	ncr_number?: string;
	description?: string;
	ncr_type?: string;
	severity?: string;
	category?: string;
	author: string;
	lot_ids?: string[];
}

export interface CloseNcrInput {
	decision: string;
	decision_maker: string;
	justification?: string;
	mrb_required?: boolean;
}

// ============================================================================
// CAPA types
// ============================================================================

export interface CapaSummary {
	id: string;
	title: string;
	capa_number?: string;
	capa_type: string;
	capa_status: string;
	status: string;
	due_date?: string;
	effectiveness_verified?: boolean;
	author: string;
	created: string;
}

export interface ListCapasResult {
	items: CapaSummary[];
	total_count: number;
	has_more: boolean;
}

export interface CapaStats {
	total: number;
	by_capa_status: { initiation: number; investigation: number; implementation: number; verification: number; closed: number };
	by_type: { corrective: number; preventive: number };
	open: number;
	overdue: number;
	verified_effective: number;
}

export interface CreateCapaInput {
	title: string;
	capa_number?: string;
	description?: string;
	capa_type?: string;
	source_ncr?: string;
	author: string;
}

export interface VerifyEffectivenessInput {
	effective: boolean;
	verified_by: string;
	notes?: string;
}

// ============================================================================
// Lot types
// ============================================================================

export interface LotSummary {
	id: string;
	title: string;
	lot_number?: string;
	quantity?: number;
	lot_status: string;
	status: string;
	start_date?: string;
	completion_date?: string;
	author: string;
	created: string;
}

export interface ListLotsResult {
	items: LotSummary[];
	total_count: number;
	has_more: boolean;
}

export interface LotStats {
	total: number;
	by_status: { in_progress: number; on_hold: number; completed: number; scrapped: number };
	total_quantity: number;
	avg_quantity: number;
	with_git_branch: number;
	merged_branches: number;
}

export interface CreateLotInput {
	title: string;
	lot_number?: string;
	quantity?: number;
	product?: string;
	notes?: string;
	author: string;
	from_routing?: boolean;
}

// ============================================================================
// Traceability types
// ============================================================================

export interface TraceParams {
	id: string;
	direction?: string;
	depth?: number;
	link_types?: string[];
}

// ============================================================================
// API namespaces
// ============================================================================

export const project = {
	open: (path: string) => call<ProjectInfo>('open_project', { path }),
	close: () => call<void>('close_project'),
	getInfo: () => call<ProjectInfo | null>('get_project_info'),
	refresh: () => call<ProjectInfo>('refresh_project')
};

export const entities = {
	list: (entityType: string, params?: ListParams) =>
		call<EntityListResult>('list_entities', { params: { entity_type: entityType, ...params } }),
	get: (id: string) => call<EntityData | null>('get_entity', { id }),
	save: (entityType: string, data: Record<string, unknown>) =>
		call<string>('save_entity', { entityType, data }),
	delete: (id: string) => call<void>('delete_entity', { id }),
	getCount: (entityType: string) => call<number>('get_entity_count', { entityType }),
	getAllCounts: () => call<Record<string, number>>('get_all_entity_counts'),
	search: (params: { entity_types?: string[]; search?: string | null; limit?: number; query?: string }) =>
		call<EntityData[]>('search_entities', { params })
};

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

export const requirements = {
	list: (params?: ListRequirementsParams) =>
		call<ListRequirementsResult>('list_requirements', { params }),
	get: (id: string) => call<unknown>('get_requirement', { id }),
	getStats: () => call<RequirementStats>('get_requirement_stats')
};

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

export const risks = {
	list: (params?: ListRisksParams) => call<ListRisksResult>('list_risks', { params }),
	get: (id: string) => call<unknown>('get_risk', { id }),
	getStats: () => call<RiskStats>('get_risk_stats'),
	getMatrix: () => call<RiskMatrix>('get_risk_matrix')
};

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

export const components = {
	list: (params?: ListComponentsParams) => call<ListComponentsResult>('list_components', { params }),
	get: (id: string) => call<unknown>('get_component', { id }),
	getStats: () => call<ComponentStats>('get_component_stats')
};

export interface ListAssembliesParams {
	status?: string[];
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
	offset?: number;
}

export const assemblies = {
	list: (params?: ListAssembliesParams) => call<ListAssembliesResult>('list_assemblies', { params }),
	get: (id: string) => call<unknown>('get_assembly', { id }),
	getStats: () => call<AssemblyStats>('get_assembly_stats')
};

export interface ListDeviationsParams {
	status?: string[];
	dev_status?: string;
	deviation_type?: string;
	category?: string;
	risk_level?: string;
	active_only?: boolean;
	search?: string;
	limit?: number;
	offset?: number;
	sort_by?: string;
	sort_desc?: boolean;
}

export const deviations = {
	list: (params?: ListDeviationsParams) => call<ListDeviationsResult>('list_deviations', { params }),
	get: (id: string) => call<unknown>('get_deviation', { id }),
	create: (input: CreateDeviationInput) => call<unknown>('create_deviation', { input }),
	approve: (id: string, input: { approved_by: string; authorization_level: string; activate?: boolean }) =>
		call<unknown>('approve_deviation', { id, input }),
	activate: (id: string) => call<unknown>('activate_deviation', { id }),
	close: (id: string, reason?: string) => call<unknown>('close_deviation', { id, reason }),
	getStats: () => call<DeviationStats>('get_deviation_stats')
};

export interface ListNcrsParams {
	status?: string[];
	ncr_type?: string;
	severity?: string;
	ncr_status?: string;
	open_only?: boolean;
	search?: string;
	limit?: number;
	offset?: number;
	sort_by?: string;
	sort_desc?: boolean;
}

export const ncrs = {
	list: (params?: ListNcrsParams) => call<ListNcrsResult>('list_ncrs', { params }),
	get: (id: string) => call<unknown>('get_ncr', { id }),
	create: (input: CreateNcrInput) => call<unknown>('create_ncr', { input }),
	advanceStatus: (id: string) => call<unknown>('advance_ncr_status', { id }),
	close: (id: string, input: CloseNcrInput) => call<unknown>('close_ncr', { id, input }),
	setContainment: (id: string, actions: string[]) =>
		call<unknown>('set_ncr_containment', { id, actions }),
	setCost: (id: string, reworkCost: number, scrapCost: number) =>
		call<unknown>('set_ncr_cost', { id, reworkCost, scrapCost }),
	setCapaLink: (id: string, capaId: string) =>
		call<unknown>('set_ncr_capa_link', { id, capaId }),
	addLotLink: (id: string, lotId: string) =>
		call<unknown>('add_ncr_lot_link', { id, lotId }),
	removeLotLink: (id: string, lotId: string) =>
		call<unknown>('remove_ncr_lot_link', { id, lotId }),
	getStats: () => call<NcrStats>('get_ncr_stats')
};

export interface ListCapasParams {
	status?: string[];
	capa_type?: string;
	capa_status?: string;
	overdue_only?: boolean;
	open_only?: boolean;
	search?: string;
	limit?: number;
	offset?: number;
	sort_by?: string;
	sort_desc?: boolean;
}

export const capas = {
	list: (params?: ListCapasParams) => call<ListCapasResult>('list_capas', { params }),
	get: (id: string) => call<unknown>('get_capa', { id }),
	create: (input: CreateCapaInput) => call<unknown>('create_capa', { input }),
	advanceStatus: (id: string) => call<unknown>('advance_capa_status', { id }),
	verifyEffectiveness: (id: string, input: VerifyEffectivenessInput) =>
		call<unknown>('verify_capa_effectiveness', { id, input }),
	setNcrLink: (id: string, ncrId: string) => call<unknown>('set_capa_ncr_link', { id, ncrId }),
	getStats: () => call<CapaStats>('get_capa_stats')
};

export interface ListLotsParams {
	status?: string[];
	lot_status?: string;
	product?: string;
	active_only?: boolean;
	search?: string;
	limit?: number;
	offset?: number;
	sort_by?: string;
	sort_desc?: boolean;
}

export const lots = {
	list: (params?: ListLotsParams) => call<ListLotsResult>('list_lots', { params }),
	get: (id: string) => call<unknown>('get_lot', { id }),
	create: (input: CreateLotInput) => call<unknown>('create_lot', { input }),
	putOnHold: (id: string) => call<unknown>('put_lot_on_hold', { id }),
	resume: (id: string) => call<unknown>('resume_lot', { id }),
	complete: (id: string) => call<unknown>('complete_lot', { id }),
	forceComplete: (id: string) => call<unknown>('force_complete_lot', { id }),
	scrap: (id: string) => call<unknown>('scrap_lot', { id }),
	updateStep: (id: string, stepIndex: number, input: { status?: string; operator?: string; notes?: string }) =>
		call<unknown>('update_lot_step', { id, stepIndex, input }),
	addStep: (id: string, input: { process_id?: string }) =>
		call<unknown>('add_lot_step', { id, input }),
	getNextStep: (id: string) => call<number | null>('get_lot_next_step', { id }),
	setProduct: (id: string, productId: string) =>
		call<unknown>('set_lot_product', { id, productId }),
	addNcr: (id: string, ncrId: string) =>
		call<unknown>('add_lot_ncr', { id, ncrId }),
	addMaterial: (id: string, input: { component_id?: string; supplier_lot?: string; quantity?: number }) =>
		call<unknown>('add_lot_material', { id, input }),
	removeMaterial: (id: string, componentId: string) =>
		call<unknown>('remove_lot_material', { id, componentId }),
	addResult: (id: string, resultId: string) =>
		call<unknown>('add_lot_result', { id, resultId }),
	getStats: () => call<LotStats>('get_lot_stats'),
	executeWiStep: (id: string, input: {
		work_instruction_id: string;
		step_number: number;
		process_index?: number;
		operator: string;
		data?: Record<string, unknown>;
		equipment?: Record<string, string>;
		notes?: string;
		complete?: boolean;
		deviation_id?: string;
	}) => call<unknown>('execute_wi_step', { id, input }),
	getWiStepStatus: (id: string, processIndex: number, wiId: string, stepNumber: number) =>
		call<unknown>('get_wi_step_status', { id, processIndex, wiId, stepNumber }),
	approveWiStep: (id: string, processIndex: number, wiId: string, stepNumber: number, input: {
		approver: string;
		comment?: string;
		reject?: boolean;
	}) => call<unknown>('approve_wi_step', { id, processIndex, wiId, stepNumber, input })
};

export const workInstructions = {
	get: (id: string) => call<unknown>('get_work_instruction', { id }),
	addStep: (id: string, input: { action: string; verification?: string; caution?: string; image?: string; estimated_time_minutes?: number }) =>
		call<unknown>('add_work_instruction_step', { id, input }),
	removeStep: (id: string, stepNumber: number) =>
		call<unknown>('remove_work_instruction_step', { id, stepNumber }),
	addTool: (id: string, input: { name: string; part_number?: string }) =>
		call<unknown>('add_work_instruction_tool', { id, input }),
	removeTool: (id: string, toolName: string) =>
		call<unknown>('remove_work_instruction_tool', { id, toolName }),
	addMaterial: (id: string, input: { name: string; specification?: string }) =>
		call<unknown>('add_work_instruction_material', { id, input }),
	removeMaterial: (id: string, materialName: string) =>
		call<unknown>('remove_work_instruction_material', { id, materialName }),
	addQualityCheck: (id: string, input: { at_step: number; characteristic: string; specification?: string }) =>
		call<unknown>('add_work_instruction_quality_check', { id, input }),
	removeQualityCheck: (id: string, atStep: number) =>
		call<unknown>('remove_work_instruction_quality_check', { id, atStep }),
	setSafety: (id: string, input: { ppe_required: { item: string; standard?: string }[]; hazards: { hazard: string; control?: string }[] }) =>
		call<unknown>('set_work_instruction_safety', { id, input }),
	clearSafety: (id: string) =>
		call<unknown>('clear_work_instruction_safety', { id })
};

export const traceability = {
	getLinksFrom: (id: string) => call<LinkInfo[]>('get_links_from', { id }),
	getLinksTo: (id: string) => call<LinkInfo[]>('get_links_to', { id }),
	traceFrom: (params: TraceParams) => call<TraceResult>('trace_from', { params }),
	traceTo: (params: TraceParams) => call<TraceResult>('trace_to', { params }),
	getCoverage: () => call<CoverageReport>('get_coverage_report'),
	addLink: (sourceId: string, targetId: string, linkType?: string) =>
		call<void>('add_link', { sourceId, targetId, linkType }),
	removeLink: (sourceId: string, targetId: string, linkType: string) =>
		call<void>('remove_link', { sourceId, targetId, linkType }),
	getLinkTypes: () => call<string[]>('get_link_types')
};

export const cache = {
	sync: () => call<void>('sync_cache')
};

export const api = {
	project,
	entities,
	requirements,
	risks,
	components,
	assemblies,
	deviations,
	ncrs,
	capas,
	lots,
	workInstructions,
	traceability,
	cache
};

export default api;
