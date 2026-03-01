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
	CycleEntity,
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
	/** Count of mitigations with actual content (non-empty action) */
	mitigation_count: number;
	/** Count of linked controls (via mitigated_by links) */
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
	by_type: {
		design: number;
		process: number;
		use: number;
		software: number;
	};
	by_level: {
		low: number;
		medium: number;
		high: number;
		critical: number;
	};
	by_status: {
		draft: number;
		review: number;
		approved: number;
		released: number;
		obsolete: number;
	};
	rpn_stats: {
		count: number;
		min: number;
		max: number;
		sum: number;
		avg: number;
	};
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

// Verification Matrix Types
export interface LinkedEntitySummary {
	id: string;
	title: string;
	status: string;
}

export interface TestResultSummary {
	id: string;
	verdict: string;
	executed_date?: string;
	executor?: string;
}

export interface TestWithResults {
	id: string;
	title: string;
	status: string;
	test_type: string;
	level: string;
	results: TestResultSummary[];
	latest_verdict?: string;
}

export interface VerificationMatrixRow {
	requirement: LinkedEntitySummary;
	req_type: string;
	level: string;
	priority: string;
	derived_requirements: LinkedEntitySummary[];
	tests: TestWithResults[];
	verification_status: string;
	pass_count: number;
	fail_count: number;
	not_run_count: number;
}

export interface VerificationMatrixSummary {
	total_requirements: number;
	fully_verified: number;
	partially_verified: number;
	not_tested: number;
	failed: number;
	verification_coverage: number;
}

export interface VerificationMatrixResponse {
	rows: VerificationMatrixRow[];
	summary: VerificationMatrixSummary;
}

export const requirements = {
	list: (params?: ListRequirementsParams) =>
		call<ListRequirementsResult>('list_requirements', { params }),
	get: (id: string) => call<unknown>('get_requirement', { id }),
	getStats: () => call<RequirementStats>('get_requirement_stats'),
	getVerificationMatrix: () => call<VerificationMatrixResponse>('get_verification_matrix')
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

// FMEA enriched types
export interface LinkedEntity {
	id: string;
	title: string;
	status?: string;
}

export interface FmeaMitigation {
	action: string;
	status?: string;
	owner?: string;
}

export interface FmeaControl {
	id: string;
	title: string;
	control_type?: string;
	tests: LinkedEntity[];
}

export interface FmeaInitialRisk {
	severity?: number;
	occurrence?: number;
	detection?: number;
	rpn?: number;
}

export interface FmeaRiskData {
	id: string;
	title: string;
	risk_type: string;
	failure_mode: string;
	/** Residual/current severity (after mitigations) */
	severity?: number;
	/** Residual/current occurrence (after mitigations) */
	occurrence?: number;
	/** Residual/current detection (after mitigations) */
	detection?: number;
	/** Residual/current RPN (after mitigations) */
	rpn?: number;
	/** Initial risk values (before mitigations) */
	initial_risk?: FmeaInitialRisk;
	risk_level?: string;
	status: string;
	hazards: LinkedEntity[];
	mitigations: FmeaMitigation[];
	controls: FmeaControl[];
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
	getMatrix: () => call<RiskMatrix>('get_risk_matrix'),
	getFmeaData: () => call<FmeaRiskData[]>('get_fmea_data')
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

/** Per-component cost breakdown line */
export interface ComponentCostLine {
	component_id: string;
	title: string;
	part_number: string;
	effective_qty: number;
	unit_price?: number;
	extended_price?: number;
	quote_id?: string;
	price_break_tier?: number;
	nre_contribution: number;
}

/** Detailed BOM cost result with per-component breakdown */
export interface BomCostResultDetailed {
	total_unit_cost: number;
	total_nre_cost: number;
	component_costs: ComponentCostLine[];
	warnings: string[];
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
	calculateCostDetailed: (id: string, productionQty?: number) =>
		call<BomCostResultDetailed>('calculate_assembly_cost_detailed', { id, productionQty }),
	calculateMass: (id: string, quantity?: number) =>
		call<BomMassResult>('calculate_assembly_mass', { id, quantity }),
	addComponent: (assemblyId: string, componentId: string, quantity: number) =>
		call<unknown>('add_assembly_component', { assemblyId, componentId, quantity }),
	removeComponent: (assemblyId: string, componentId: string) =>
		call<unknown>('remove_assembly_component', { assemblyId, componentId }),
	updateComponentQuantity: (assemblyId: string, componentId: string, quantity: number) =>
		call<unknown>('update_assembly_component_quantity', { assemblyId, componentId, quantity }),
	getRouting: (id: string) => call<string[]>('get_assembly_routing', { id })
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

// DMM (Domain Mapping Matrix) result types
export interface DmmEntity {
	id: string;
	title: string;
}

export interface DmmLink {
	row_id: string;
	col_id: string;
}

export interface DmmCoverage {
	row_coverage_pct: number;
	col_coverage_pct: number;
	rows_with_links: number;
	total_rows: number;
	cols_with_links: number;
	total_cols: number;
	total_links: number;
}

export interface DmmResult {
	row_type: string;
	col_type: string;
	row_entities: DmmEntity[];
	col_entities: DmmEntity[];
	links: DmmLink[];
	coverage: DmmCoverage;
}

export const traceability = {
	getLinksFrom: (id: string) => call<LinkInfo[]>('get_links_from', { id }),
	getLinksTo: (id: string) => call<LinkInfo[]>('get_links_to', { id }),
	traceFrom: (params: TraceParams) => call<TraceResult>('trace_from', { params }),
	traceTo: (params: TraceParams) => call<TraceResult>('trace_to', { params }),
	getCoverage: () => call<CoverageReport>('get_coverage_report'),
	getDsm: (entityType?: string) => call<unknown>('get_dsm', { entity_type: entityType }),
	getDmm: (rowType: string, colType: string) =>
		call<DmmResult>('get_dmm', { rowType, colType }),
	findOrphans: (entityType?: string) => call<string[]>('find_orphans', { entity_type: entityType }),
	findCycles: (entityType?: string) =>
		call<CycleEntity[][]>('find_cycles', { entity_type: entityType }),
	addLink: (sourceId: string, targetId: string, linkType?: string) =>
		call<void>('add_link', { sourceId, targetId, linkType }),
	removeLink: (sourceId: string, targetId: string, linkType: string) =>
		call<void>('remove_link', { sourceId, targetId, linkType }),
	getLinkTypes: () => call<string[]>('get_link_types'),
	getMaturityMismatches: () => call<MaturityMismatch[]>('get_maturity_mismatches')
};

export interface MaturityMismatch {
	source_id: string;
	source_title: string;
	source_status: string;
	target_id: string;
	target_title: string;
	target_status: string;
	link_type: string;
}

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
 * NCRs API (specialized commands + CRUD)
 */
export interface ListNcrsParams {
	status?: string[];
	ncr_type?: string;
	severity?: string;
	ncr_status?: string;
	category?: string;
	open_only?: boolean;
	recent_days?: number;
	search?: string;
	tags?: string[];
	limit?: number;
	offset?: number;
	sort_by?: string;
	sort_desc?: boolean;
}

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
	by_ncr_status: {
		open: number;
		containment: number;
		investigation: number;
		disposition: number;
		closed: number;
	};
	by_type: {
		internal: number;
		supplier: number;
		customer: number;
	};
	by_severity: {
		minor: number;
		major: number;
		critical: number;
	};
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

export interface UpdateNcrInput {
	title?: string;
	ncr_number?: string;
	description?: string;
	ncr_type?: string;
	severity?: string;
	category?: string;
	status?: string;
	ncr_status?: string;
}

export interface CloseNcrInput {
	decision: string;
	decision_maker: string;
	justification?: string;
	mrb_required?: boolean;
}

export const ncrs = {
	list: (params?: ListNcrsParams) => call<ListNcrsResult>('list_ncrs', { params }),
	get: (id: string) => call<unknown>('get_ncr', { id }),
	create: (input: CreateNcrInput) => call<unknown>('create_ncr', { input }),
	update: (id: string, input: UpdateNcrInput) => call<unknown>('update_ncr', { id, input }),
	delete: (id: string) => call<void>('delete_ncr', { id }),
	close: (id: string, input: CloseNcrInput) => call<unknown>('close_ncr', { id, input }),
	advanceStatus: (id: string) => call<unknown>('advance_ncr_status', { id }),
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

/**
 * CAPAs API (specialized commands + CRUD)
 */
export interface ListCapasParams {
	status?: string[];
	capa_type?: string;
	capa_status?: string;
	overdue_only?: boolean;
	open_only?: boolean;
	recent_days?: number;
	search?: string;
	tags?: string[];
	limit?: number;
	offset?: number;
	sort_by?: string;
	sort_desc?: boolean;
}

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
	by_capa_status: {
		initiation: number;
		investigation: number;
		implementation: number;
		verification: number;
		closed: number;
	};
	by_type: {
		corrective: number;
		preventive: number;
	};
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

export const capas = {
	list: (params?: ListCapasParams) => call<ListCapasResult>('list_capas', { params }),
	get: (id: string) => call<unknown>('get_capa', { id }),
	create: (input: CreateCapaInput) => call<unknown>('create_capa', { input }),
	delete: (id: string) => call<void>('delete_capa', { id }),
	advanceStatus: (id: string) => call<unknown>('advance_capa_status', { id }),
	verifyEffectiveness: (id: string, input: VerifyEffectivenessInput) =>
		call<unknown>('verify_capa_effectiveness', { id, input }),
	setNcrLink: (id: string, ncrId: string) => call<unknown>('set_capa_ncr_link', { id, ncrId }),
	getStats: () => call<CapaStats>('get_capa_stats')
};

/**
 * Lots API (specialized commands + CRUD)
 */
export interface ListLotsParams {
	status?: string[];
	lot_status?: string;
	product?: string;
	active_only?: boolean;
	recent_days?: number;
	search?: string;
	tags?: string[];
	limit?: number;
	offset?: number;
	sort_by?: string;
	sort_desc?: boolean;
}

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
	by_status: {
		in_progress: number;
		on_hold: number;
		completed: number;
		scrapped: number;
	};
	total_quantity: number;
	avg_quantity: number;
	with_git_branch: number;
	merged_branches: number;
}

export interface ExecuteWiStepInput {
	work_instruction_id: string;
	step_number: number;
	process_index?: number;
	operator: string;
	operator_email?: string;
	data?: Record<string, unknown>;
	equipment?: Record<string, string>;
	notes?: string;
	sign?: boolean;
	require_approval?: boolean;
	complete?: boolean;
	deviation_id?: string;
}

export interface WiStepExecutionResultDto {
	lot: unknown;
	process_index: number;
	step_number: number;
	was_completed: boolean;
	deviation_used?: string;
}

export interface ApproveWiStepInput {
	approver: string;
	email?: string;
	role?: string;
	comment?: string;
	sign?: boolean;
	reject?: boolean;
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

export const lots = {
	list: (params?: ListLotsParams) => call<ListLotsResult>('list_lots', { params }),
	get: (id: string) => call<unknown>('get_lot', { id }),
	create: (input: CreateLotInput) => call<unknown>('create_lot', { input }),
	delete: (id: string) => call<void>('delete_lot', { id }),
	putOnHold: (id: string) => call<unknown>('put_lot_on_hold', { id }),
	resume: (id: string) => call<unknown>('resume_lot', { id }),
	complete: (id: string) => call<unknown>('complete_lot', { id }),
	scrap: (id: string) => call<unknown>('scrap_lot', { id }),
	updateStep: (id: string, stepIndex: number, input: { status?: string; operator?: string; notes?: string }) =>
		call<unknown>('update_lot_step', { id, stepIndex, input }),
	addStep: (id: string, input: { process_id?: string }) =>
		call<unknown>('add_lot_step', { id, input }),
	getNextStep: (id: string) =>
		call<number | null>('get_lot_next_step', { id }),
	setProduct: (id: string, productId: string) =>
		call<unknown>('set_lot_product', { id, productId }),
	addNcr: (id: string, ncrId: string) =>
		call<unknown>('add_lot_ncr', { id, ncrId }),
	addMaterial: (id: string, input: { component_id?: string; supplier_lot?: string; quantity?: number }) =>
		call<unknown>('add_lot_material', { id, input }),
	removeMaterial: (id: string, componentId: string) =>
		call<unknown>('remove_lot_material', { id, componentId }),
	forceComplete: (id: string) =>
		call<unknown>('force_complete_lot', { id }),
	addResult: (id: string, resultId: string) =>
		call<unknown>('add_lot_result', { id, resultId }),
	getStats: () => call<LotStats>('get_lot_stats'),
	executeWiStep: (id: string, input: ExecuteWiStepInput) =>
		call<WiStepExecutionResultDto>('execute_wi_step', { id, input }),
	getWiStepStatus: (id: string, processIndex: number, wiId: string, stepNumber: number) =>
		call<unknown>('get_wi_step_status', { id, processIndex, wiId, stepNumber }),
	approveWiStep: (id: string, processIndex: number, wiId: string, stepNumber: number, input: ApproveWiStepInput) =>
		call<unknown>('approve_wi_step', { id, processIndex, wiId, stepNumber, input }),
	validateStepOrder: (id: string, processIndex: number, wiId: string, stepNumber: number) =>
		call<void>('validate_lot_step_order', { id, processIndex, wiId, stepNumber })
};

/**
 * Work Instructions API (specialized commands)
 */
export interface AddWiStepInput {
	action: string;
	verification?: string;
	caution?: string;
	image?: string;
	estimated_time_minutes?: number;
}

export interface AddWiToolInput {
	name: string;
	part_number?: string;
}

export interface AddWiMaterialInput {
	name: string;
	specification?: string;
}

export interface AddWiQualityCheckInput {
	at_step: number;
	characteristic: string;
	specification?: string;
}

export interface PpeItemInput {
	item: string;
	standard?: string;
}

export interface HazardInput {
	hazard: string;
	control?: string;
}

export interface SetWiSafetyInput {
	ppe_required: PpeItemInput[];
	hazards: HazardInput[];
}

export const workInstructions = {
	list: (params?: Record<string, unknown>) =>
		call<unknown>('list_work_instructions', { params }),
	get: (id: string) => call<unknown>('get_work_instruction', { id }),
	create: (input: Record<string, unknown>) =>
		call<unknown>('create_work_instruction', { input }),
	update: (id: string, input: Record<string, unknown>) =>
		call<unknown>('update_work_instruction', { id, input }),
	delete: (id: string) => call<void>('delete_work_instruction', { id }),
	getStats: () => call<unknown>('get_work_instruction_stats'),
	addStep: (id: string, input: AddWiStepInput) =>
		call<unknown>('add_work_instruction_step', { id, input }),
	removeStep: (id: string, stepNumber: number) =>
		call<unknown>('remove_work_instruction_step', { id, stepNumber }),
	addTool: (id: string, input: AddWiToolInput) =>
		call<unknown>('add_work_instruction_tool', { id, input }),
	removeTool: (id: string, toolName: string) =>
		call<unknown>('remove_work_instruction_tool', { id, toolName }),
	addMaterial: (id: string, input: AddWiMaterialInput) =>
		call<unknown>('add_work_instruction_material', { id, input }),
	removeMaterial: (id: string, materialName: string) =>
		call<unknown>('remove_work_instruction_material', { id, materialName }),
	addQualityCheck: (id: string, input: AddWiQualityCheckInput) =>
		call<unknown>('add_work_instruction_quality_check', { id, input }),
	removeQualityCheck: (id: string, atStep: number) =>
		call<unknown>('remove_work_instruction_quality_check', { id, atStep }),
	setSafety: (id: string, input: SetWiSafetyInput) =>
		call<unknown>('set_work_instruction_safety', { id, input }),
	clearSafety: (id: string) =>
		call<unknown>('clear_work_instruction_safety', { id })
};

/**
 * Settings API - Config and Team Roster Management
 */
export interface GeneralSettings {
	author: string | null;
	editor: string | null;
	pager: string | null;
	default_format: string | null;
}

export interface WorkflowSettings {
	enabled: boolean;
	provider: string;
	require_branch: boolean;
	auto_commit: boolean;
	auto_merge: boolean;
	base_branch: string;
	branch_pattern: string;
	submit_message: string;
	approve_message: string;
}

export interface ManufacturingSettings {
	lot_branch_enabled: boolean;
	base_branch: string | null;
	branch_pattern: string | null;
	create_tags: boolean;
	sign_commits: boolean;
}

export interface ConfigPaths {
	global_config: string | null;
	project_config: string | null;
}

export interface AllSettings {
	general: GeneralSettings;
	workflow: WorkflowSettings;
	manufacturing: ManufacturingSettings;
	config_paths: ConfigPaths;
}

export interface TeamMemberDto {
	name: string;
	email: string;
	username: string;
	roles: string[];
	active: boolean;
	signing_format: string | null;
}

export interface TeamRosterDto {
	version: number;
	members: TeamMemberDto[];
	approval_matrix: Record<string, string[]>;
}

export interface EntityPrefixInfo {
	prefix: string;
	name: string;
}

export interface GitUserInfo {
	name: string | null;
	email: string | null;
}

// ============================================================================
// Mate & Stackup Analysis Types
// ============================================================================

export interface RecalcMateResult {
	mate: Record<string, unknown>;
	changed: boolean;
	error: string | null;
}

export interface RecalcAllMatesResult {
	processed: number;
	changed: number;
	errors: number;
}

export interface AnalyzeStackupResult {
	stackup: Record<string, unknown>;
	changed: boolean;
}

/**
 * Mates API - fit analysis recalculation
 */
export const mates = {
	recalculate: (id: string) => call<RecalcMateResult>('recalc_mate', { id }),
	recalculateAll: () => call<RecalcAllMatesResult>('recalc_all_mates')
};

/**
 * Stackups API - tolerance analysis
 */
export const stackups = {
	analyze: (id: string, monteCarloIterations?: number) =>
		call<Record<string, unknown>>('analyze_stackup', {
			id,
			monteCarloIterations: monteCarloIterations ?? null
		})
};

// ============================================================================
// Version Control Types
// ============================================================================

export interface GitStatusInfo {
	current_branch: string;
	is_clean: boolean;
	is_main_branch: boolean;
	uncommitted_files: UncommittedFile[];
	is_repo: boolean;
}

export interface UncommittedFile {
	path: string;
	status: string; // "modified", "added", "deleted", "untracked", "renamed"
	entity_id: string | null;
	entity_title: string | null;
}

export interface VcGitUserInfo {
	name: string | null;
	email: string | null;
	signing_key: string | null;
	signing_configured: boolean;
}

export interface GitCommitInfo {
	hash: string;
	short_hash: string;
	message: string;
	author: string;
	author_email: string | null;
	date: string;
	is_signed: boolean;
}

export interface CommitFileInfo {
	path: string;
	change_type: string;
	entity_id: string | null;
	entity_title: string | null;
	entity_type: string | null;
}

export interface CommitDetails {
	hash: string;
	short_hash: string;
	full_message: string;
	author: string;
	author_email: string | null;
	date: string;
	is_signed: boolean;
	files: CommitFileInfo[];
	insertions: number;
	deletions: number;
}

export interface WorkflowHistory {
	entity_id: string;
	title: string;
	current_status: string;
	revision: number | null;
	events: WorkflowEvent[];
	tags: string[];
}

export interface WorkflowEvent {
	event_type: string; // "created", "approved", "released", "rejected"
	actor: string;
	timestamp: string;
	role: string | null;
	comment: string | null;
	signature_verified: boolean | null;
}

export interface BranchInfo {
	name: string;
	is_current: boolean;
	is_remote: boolean;
	last_commit: string | null;
	last_message: string | null;
}

export interface TagInfo {
	name: string;
	message: string | null;
	tagger: string | null;
	date: string | null;
	commit: string | null;
}

export interface CommitResult {
	hash: string;
	message: string;
	files_changed: number;
}

export interface PushResult {
	branch: string;
	upstream_set: boolean;
}

export const versionControl = {
	// Git status
	getStatus: () => call<GitStatusInfo>('get_git_status'),
	getUser: () => call<VcGitUserInfo>('get_vc_git_user'),

	// Entity history
	getEntityHistory: (id: string, limit?: number) =>
		call<GitCommitInfo[]>('get_entity_history', { id, limit }),
	getEntityWorkflowHistory: (id: string) =>
		call<WorkflowHistory>('get_entity_workflow_history', { id }),
	getEntityDiff: (id: string, commitHash: string) =>
		call<string>('get_entity_file_diff', { id, commitHash }),

	// Branches
	listBranches: () => call<BranchInfo[]>('list_git_branches'),
	checkoutBranch: (branch: string) => call<void>('checkout_git_branch', { branch }),
	createBranch: (name: string, checkout: boolean) =>
		call<void>('create_git_branch', { name, checkout }),

	// Tags
	listTags: (pattern?: string) => call<TagInfo[]>('list_git_tags', { pattern }),

	// Staging operations
	stageFiles: (paths: string[]) => call<void>('stage_files', { paths }),
	stageEntity: (id: string) => call<void>('stage_entity', { id }),
	unstageFiles: (paths: string[]) => call<void>('unstage_files', { paths }),
	discardChanges: (paths: string[]) => call<void>('discard_changes', { paths }),
	commit: (message: string, sign: boolean) =>
		call<CommitResult>('commit_changes', { message, sign }),
	push: (branch?: string, setUpstream?: boolean) =>
		call<PushResult>('push_changes', { branch, setUpstream }),
	pull: () => call<void>('pull_changes'),
	fetch: () => call<void>('fetch_changes'),

	// Recent commits
	getRecentCommits: (limit?: number) => call<GitCommitInfo[]>('get_recent_commits', { limit }),

	// Commit details
	getCommitDetails: (hash: string) => call<CommitDetails>('get_commit_details', { hash }),
	getCommitFileDiff: (commitHash: string, filePath: string) =>
		call<string>('get_commit_file_diff', { commitHash, filePath }),

	// File diff
	getUncommittedFileDiff: (path: string) => call<string>('get_uncommitted_file_diff', { path })
};

/**
 * Cache management API
 */
export const cache = {
	sync: () => call<void>('sync_cache')
};

export const settings = {
	// Config settings
	getAll: () => call<AllSettings>('get_all_settings'),
	getGeneral: () => call<GeneralSettings>('get_general_settings'),
	getWorkflow: () => call<WorkflowSettings>('get_workflow_settings'),
	getManufacturing: () => call<ManufacturingSettings>('get_manufacturing_settings'),
	saveGeneral: (settings: GeneralSettings, saveToGlobal: boolean) =>
		call<void>('save_general_settings', { settings, saveToGlobal }),
	saveWorkflow: (settings: WorkflowSettings) =>
		call<void>('save_workflow_settings', { settings }),
	saveManufacturing: (settings: ManufacturingSettings) =>
		call<void>('save_manufacturing_settings', { settings }),

	// Team roster
	getTeamRoster: () => call<TeamRosterDto | null>('get_team_roster'),
	saveTeamRoster: (roster: TeamRosterDto) => call<void>('save_team_roster', { roster }),
	initTeamRoster: () => call<TeamRosterDto>('init_team_roster'),
	addTeamMember: (member: TeamMemberDto) => call<TeamRosterDto>('add_team_member', { member }),
	updateTeamMember: (username: string, member: TeamMemberDto) =>
		call<TeamRosterDto>('update_team_member', { username, member }),
	removeTeamMember: (username: string) => call<TeamRosterDto>('remove_team_member', { username }),
	setTeamMemberActive: (username: string, active: boolean) =>
		call<TeamRosterDto>('set_team_member_active', { username, active }),
	updateApprovalMatrix: (entityPrefix: string, roles: string[]) =>
		call<TeamRosterDto>('update_approval_matrix', { entityPrefix, roles }),
	removeApprovalMatrixEntry: (entityPrefix: string) =>
		call<TeamRosterDto>('remove_approval_matrix_entry', { entityPrefix }),

	// Helpers
	getAvailableRoles: () => call<string[]>('get_available_roles'),
	getAvailableSigningFormats: () => call<string[]>('get_available_signing_formats'),
	getEntityPrefixesForApproval: () => call<EntityPrefixInfo[]>('get_entity_prefixes_for_approval'),
	getCurrentGitUser: () => call<GitUserInfo>('get_current_git_user')
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
	ncrs,
	capas,
	lots,
	workInstructions,
	mates,
	stackups,
	traceability,
	settings,
	versionControl,
	cache
};

export default api;
