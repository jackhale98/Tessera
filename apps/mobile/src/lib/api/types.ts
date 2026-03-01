/**
 * TypeScript types mirroring Rust types from tdt-core
 */

// Entity ID types
export type EntityPrefix =
	| 'REQ'
	| 'HAZ'
	| 'RISK'
	| 'TEST'
	| 'RSLT'
	| 'CMP'
	| 'ASM'
	| 'FEAT'
	| 'MATE'
	| 'TOL'
	| 'PROC'
	| 'CTRL'
	| 'WORK'
	| 'LOT'
	| 'DEV'
	| 'NCR'
	| 'CAPA'
	| 'QUOT'
	| 'SUP';

// Common entity fields
export type Status = 'draft' | 'review' | 'approved' | 'released' | 'obsolete';
export type Priority = 'low' | 'medium' | 'high' | 'critical';

// Project types
export interface EntityCounts {
	requirements: number;
	risks: number;
	hazards: number;
	tests: number;
	results: number;
	components: number;
	assemblies: number;
	features: number;
	mates: number;
	stackups: number;
	processes: number;
	controls: number;
	work_instructions: number;
	lots: number;
	deviations: number;
	ncrs: number;
	capas: number;
	quotes: number;
	suppliers: number;
	actions: number;
}

export interface ProjectInfo {
	path: string;
	name: string;
	entity_counts: EntityCounts;
	author: string;
}

// Links structure
export interface Links {
	[key: string]: string[];
}

// Requirement types
export type RequirementType = 'input' | 'output';
export type RequirementLevel = 'stakeholder' | 'system' | 'subsystem' | 'component' | 'detail';

export interface Requirement {
	id: string;
	req_type: RequirementType;
	level: RequirementLevel;
	title: string;
	text: string;
	rationale?: string;
	acceptance_criteria: string[];
	priority: Priority;
	status: Status;
	created: string;
	author: string;
	tags: string[];
	links: Links;
}

// Risk types
export type RiskType = 'design' | 'process' | 'use' | 'software';
export type RiskLevel = 'low' | 'medium' | 'high' | 'critical';

export interface Mitigation {
	action: string;
	mitigation_type: 'prevention' | 'detection' | 'protection';
	responsible?: string;
	due_date?: string;
	effectiveness?: number;
	status: Status;
}

export interface Risk {
	id: string;
	title: string;
	risk_type: RiskType;
	description: string;
	failure_mode: string;
	cause?: string;
	effect?: string;
	severity?: number;
	occurrence?: number;
	detection?: number;
	rpn?: number;
	risk_level?: RiskLevel;
	mitigations: Mitigation[];
	status: Status;
	priority: Priority;
	created: string;
	author: string;
	tags: string[];
	links: Links;
}

// Component types
export type ComponentCategory = 'mechanical' | 'electrical' | 'software' | 'firmware' | 'other';
export type MakeBuy = 'make' | 'buy' | 'modify';

export interface Component {
	id: string;
	title: string;
	part_number?: string;
	revision?: string;
	description?: string;
	category: ComponentCategory;
	make_buy: MakeBuy;
	unit_cost?: number;
	mass?: number;
	quantity?: number;
	status: Status;
	created: string;
	author: string;
	tags: string[];
	links: Links;
}

// Test types
export type TestType = 'unit' | 'integration' | 'system' | 'acceptance' | 'regression';
export type ValidationMethod = 'analysis' | 'demonstration' | 'inspection' | 'test';

export interface TestStep {
	number: number;
	description: string;
	expected: string;
}

export interface Test {
	id: string;
	title: string;
	test_type: TestType;
	validation_method: ValidationMethod;
	description?: string;
	preconditions?: string;
	procedure: TestStep[];
	pass_criteria?: string;
	status: Status;
	priority: Priority;
	created: string;
	author: string;
	tags: string[];
	links: Links;
}

// Result types
export type Verdict = 'pass' | 'fail' | 'blocked' | 'skip';

export interface StepResult {
	step_number: number;
	actual?: string;
	verdict: Verdict;
	notes?: string;
}

export interface Result {
	id: string;
	title: string;
	test_id: string;
	verdict: Verdict;
	execution_date: string;
	executor: string;
	step_results: StepResult[];
	notes?: string;
	attachments: string[];
	status: Status;
	created: string;
	author: string;
	tags: string[];
	links: Links;
}

// Generic entity data from list/get commands
export interface EntityData {
	id: string;
	prefix: string;
	title: string;
	status: string;
	author: string;
	created: string;
	tags: string[]; // May be empty array, never null from backend
	data: Record<string, unknown> | null;
}

// Entity list result
export interface EntityListResult {
	items: EntityData[];
	total_count: number;
}

// Filter and list params
export interface ListParams {
	status?: Status[];
	priority?: Priority[];
	search?: string;
	tags?: string[];
	limit?: number;
	offset?: number;
}

// Traceability types
export interface TraceNode {
	id: string;
	entity_type: EntityPrefix;
	title: string;
	status: Status;
	depth: number;
}

export interface TraceEdge {
	from_id: string;
	to_id: string;
	link_type: string;
}

export interface TraceResult {
	root_id: string;
	nodes: TraceNode[];
	edges: TraceEdge[];
}

export interface CycleEntity {
	id: string;
	entity_type: string;
	title: string;
}

export interface CoverageStats {
	total: number;
	covered: number;
	percentage: number;
}

export interface CoverageReport {
	requirements_verified: CoverageStats;
	requirements_satisfied: CoverageStats;
	risks_mitigated: CoverageStats;
	risks_verified: CoverageStats;
	tests_linked: CoverageStats;
	components_with_suppliers: CoverageStats;
	health_score: number;
}

// Risk matrix
export interface RiskMatrix {
	cells: RiskMatrixCell[];
	max_severity: number;
	max_occurrence: number;
}

export interface RiskMatrixCell {
	severity: number;
	occurrence: number;
	count: number;
	risk_ids: string[];
	risk_level: RiskLevel;
}

// Deviation types
export type DeviationType = 'temporary' | 'permanent' | 'emergency';
export type DeviationCategory =
	| 'material'
	| 'process'
	| 'equipment'
	| 'tooling'
	| 'specification'
	| 'documentation';
export type DevStatus = 'pending' | 'approved' | 'active' | 'expired' | 'closed' | 'rejected';
export type DeviationRiskLevel = 'low' | 'medium' | 'high';
export type AuthorizationLevel = 'engineering' | 'quality' | 'management';

export interface DevRisk {
	level: DeviationRiskLevel;
	assessment?: string;
	mitigations: string[];
}

export interface DevApproval {
	approved_by?: string;
	approval_date?: string;
	authorization_level?: AuthorizationLevel;
}

export interface DevLinks {
	processes: string[];
	lots: string[];
	components: string[];
	requirements: string[];
	ncrs: string[];
	change_order?: string;
}

export interface Deviation {
	id: string;
	title: string;
	deviation_number?: string;
	deviation_type: DeviationType;
	category: DeviationCategory;
	description?: string;
	risk: DevRisk;
	approval: DevApproval;
	effective_date?: string;
	expiration_date?: string;
	dev_status: DevStatus;
	notes?: string;
	links: DevLinks;
	status: Status;
	created: string;
	author: string;
	entity_revision: number;
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

export interface DevStatusCounts {
	pending: number;
	approved: number;
	active: number;
	expired: number;
	closed: number;
	rejected: number;
}

export interface DeviationTypeCounts {
	temporary: number;
	permanent: number;
	emergency: number;
}

export interface DeviationCategoryCounts {
	material: number;
	process: number;
	equipment: number;
	tooling: number;
	specification: number;
	documentation: number;
}

export interface DeviationRiskLevelCounts {
	low: number;
	medium: number;
	high: number;
}

export interface DeviationStats {
	total: number;
	by_dev_status: DevStatusCounts;
	by_type: DeviationTypeCounts;
	by_category: DeviationCategoryCounts;
	by_risk: DeviationRiskLevelCounts;
	active: number;
}

// ============================================================================
// Filter Framework Types
// ============================================================================

/**
 * Filter field types determine how the filter control is rendered
 */
export type FilterFieldType =
	| 'select' // Single-select dropdown
	| 'multi-select' // Multi-select with checkboxes
	| 'text' // Free-text input
	| 'number' // Numeric input
	| 'number-range' // Min/max range
	| 'boolean' // Checkbox/toggle
	| 'date' // Date picker
	| 'date-range'; // Date range picker

/**
 * Option for select/multi-select filters
 */
export interface FilterOption {
	value: string;
	label: string;
	count?: number; // Optional count to show next to option
}

/**
 * Filter field definition - describes a single filter control
 */
export interface FilterFieldDefinition {
	key: string; // The parameter key sent to backend (e.g., 'status', 'risk_level')
	label: string; // Display label for the filter
	type: FilterFieldType;
	options?: FilterOption[]; // For select/multi-select
	placeholder?: string; // Placeholder text
	defaultValue?: unknown; // Default value when filter is reset
	// Number-specific
	min?: number;
	max?: number;
	step?: number;
	// Boolean-specific
	trueLabel?: string;
	falseLabel?: string;
}

/**
 * Current filter value (can be various types depending on field type)
 */
export type FilterValue =
	| string
	| string[]
	| number
	| boolean
	| null
	| { min?: number; max?: number }
	| { start?: string; end?: string };

/**
 * Active filter state - maps field keys to their current values
 */
export interface FilterState {
	[key: string]: FilterValue;
}

/**
 * Filter configuration for an entity type
 */
export interface EntityFilterConfig {
	entityType: EntityPrefix;
	fields: FilterFieldDefinition[];
	// Optional quick filters (commonly used filter combinations)
	quickFilters?: QuickFilter[];
}

/**
 * Quick filter - a preset combination of filter values
 */
export interface QuickFilter {
	id: string;
	label: string;
	icon?: string;
	filters: FilterState;
}

/**
 * Callback when filters change
 */
export type OnFiltersChange = (filters: FilterState) => void;

// ============================================================================
// Entity-Specific List Parameter Types (for Tauri commands)
// ============================================================================

export interface ListRequirementsParams {
	status?: string[];
	req_type?: string;
	level?: string;
	priority?: string;
	orphans_only?: boolean;
	unverified_only?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListRisksParams {
	status?: string[];
	risk_type?: string;
	risk_level?: string;
	min_rpn?: number;
	max_rpn?: number;
	unmitigated_only?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListComponentsParams {
	status?: string[];
	category?: string;
	make_buy?: string;
	has_routing?: boolean;
	long_lead_only?: boolean;
	single_source_only?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListAssembliesParams {
	status?: string[];
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListTestsParams {
	status?: string[];
	test_type?: string;
	level?: string;
	method?: string;
	has_results?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListResultsParams {
	status?: string[];
	verdict?: string;
	test_id?: string;
	with_failures?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListHazardsParams {
	status?: string[];
	category?: string;
	severity?: string;
	has_risks?: boolean;
	has_controls?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListFeaturesParams {
	status?: string[];
	component?: string;
	feature_type?: string;
	has_gdt?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListMatesParams {
	status?: string[];
	fit_type?: string;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListStackupsParams {
	status?: string[];
	result?: string;
	critical_only?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListProcessesParams {
	status?: string[];
	process_type?: string;
	has_equipment?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListControlsParams {
	status?: string[];
	control_type?: string;
	control_category?: string;
	critical_only?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListWorkInstructionsParams {
	status?: string[];
	process?: string;
	has_safety?: boolean;
	has_quality_checks?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListLotsParams {
	status?: string[];
	lot_status?: string;
	product?: string;
	active_only?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListNcrsParams {
	status?: string[];
	ncr_type?: string;
	ncr_status?: string;
	severity?: string;
	category?: string;
	mrb_required?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListCapasParams {
	status?: string[];
	capa_type?: string;
	capa_status?: string;
	overdue_only?: boolean;
	open_only?: boolean;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListSuppliersParams {
	status?: string[];
	capability?: string;
	expired_certs?: boolean;
	certs_expiring_days?: number;
	currency?: string;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}

export interface ListQuotesParams {
	status?: string[];
	quote_status?: string;
	supplier?: string;
	component?: string;
	assembly?: string;
	expired_only?: boolean;
	has_price_breaks?: boolean;
	currency?: string;
	search?: string;
	sort_by?: string;
	sort_desc?: boolean;
	limit?: number;
}
