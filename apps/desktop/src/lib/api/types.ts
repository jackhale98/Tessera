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
}

export interface ProjectInfo {
	path: string;
	name: string;
	entity_counts: EntityCounts;
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
	tags: string[];
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
export interface TraceLink {
	entity_id: string;
	entity_type: EntityPrefix;
	title: string;
	status: Status;
	link_type: string;
}

export interface TraceResult {
	entity_id: string;
	entity_type: EntityPrefix;
	title: string;
	upstream: TraceLink[];
	downstream: TraceLink[];
}

export interface CoverageStats {
	total: number;
	covered: number;
	percentage: number;
}

export interface CoverageReport {
	requirements_verified: CoverageStats;
	requirements_tested: CoverageStats;
	risks_mitigated: CoverageStats;
	tests_executed: CoverageStats;
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
