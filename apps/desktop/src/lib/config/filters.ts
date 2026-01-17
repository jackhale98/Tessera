/**
 * Filter configurations for each entity type
 *
 * These configurations define what filters are available for each entity list view.
 * They are used by the FilterPanel component to render appropriate filter controls.
 */

import type { EntityFilterConfig, FilterFieldDefinition, QuickFilter } from '$lib/api/types';

// ============================================================================
// Common filter field definitions (reused across entity types)
// ============================================================================

const statusOptions = [
	{ value: 'draft', label: 'Draft' },
	{ value: 'review', label: 'Review' },
	{ value: 'approved', label: 'Approved' },
	{ value: 'released', label: 'Released' },
	{ value: 'obsolete', label: 'Obsolete' }
];

const priorityOptions = [
	{ value: 'low', label: 'Low' },
	{ value: 'medium', label: 'Medium' },
	{ value: 'high', label: 'High' },
	{ value: 'critical', label: 'Critical' }
];

const statusField: FilterFieldDefinition = {
	key: 'status',
	label: 'Status',
	type: 'multi-select',
	options: statusOptions
};

const priorityField: FilterFieldDefinition = {
	key: 'priority',
	label: 'Priority',
	type: 'select',
	options: priorityOptions,
	placeholder: 'Any priority'
};

// ============================================================================
// Requirements Filter Configuration
// ============================================================================

export const requirementsFilterConfig: EntityFilterConfig = {
	entityType: 'REQ',
	fields: [
		statusField,
		{
			key: 'req_type',
			label: 'Type',
			type: 'select',
			options: [
				{ value: 'input', label: 'Input' },
				{ value: 'output', label: 'Output' }
			],
			placeholder: 'Any type'
		},
		{
			key: 'level',
			label: 'Level',
			type: 'select',
			options: [
				{ value: 'stakeholder', label: 'Stakeholder' },
				{ value: 'system', label: 'System' },
				{ value: 'subsystem', label: 'Subsystem' },
				{ value: 'component', label: 'Component' },
				{ value: 'detail', label: 'Detail' }
			],
			placeholder: 'Any level'
		},
		priorityField,
		{
			key: 'orphans_only',
			label: 'Show orphans only',
			type: 'boolean',
			trueLabel: 'Orphans only'
		},
		{
			key: 'unverified_only',
			label: 'Show unverified only',
			type: 'boolean',
			trueLabel: 'Unverified only'
		}
	],
	quickFilters: [
		{
			id: 'active',
			label: 'Active',
			filters: { status: ['draft', 'review', 'approved'] }
		},
		{
			id: 'needs-review',
			label: 'Needs Review',
			filters: { status: ['review'] }
		},
		{
			id: 'high-priority',
			label: 'High Priority',
			filters: { priority: 'high' }
		}
	]
};

// ============================================================================
// Risks Filter Configuration
// ============================================================================

export const risksFilterConfig: EntityFilterConfig = {
	entityType: 'RISK',
	fields: [
		statusField,
		{
			key: 'risk_type',
			label: 'Risk Type',
			type: 'select',
			options: [
				{ value: 'design', label: 'Design' },
				{ value: 'process', label: 'Process' },
				{ value: 'use', label: 'Use' },
				{ value: 'software', label: 'Software' }
			],
			placeholder: 'Any type'
		},
		{
			key: 'risk_level',
			label: 'Risk Level',
			type: 'select',
			options: [
				{ value: 'low', label: 'Low' },
				{ value: 'medium', label: 'Medium' },
				{ value: 'high', label: 'High' },
				{ value: 'critical', label: 'Critical' }
			],
			placeholder: 'Any level'
		},
		{
			key: 'rpn_range',
			label: 'RPN Range',
			type: 'number-range',
			min: 1,
			max: 1000
		},
		{
			key: 'unmitigated_only',
			label: 'Show unmitigated only',
			type: 'boolean',
			trueLabel: 'Unmitigated only'
		}
	],
	quickFilters: [
		{
			id: 'high-risk',
			label: 'High/Critical',
			filters: { risk_level: 'high' }
		},
		{
			id: 'unmitigated',
			label: 'Unmitigated',
			filters: { unmitigated_only: true }
		}
	]
};

// ============================================================================
// Components Filter Configuration
// ============================================================================

export const componentsFilterConfig: EntityFilterConfig = {
	entityType: 'CMP',
	fields: [
		statusField,
		{
			key: 'category',
			label: 'Category',
			type: 'select',
			options: [
				{ value: 'mechanical', label: 'Mechanical' },
				{ value: 'electrical', label: 'Electrical' },
				{ value: 'software', label: 'Software' },
				{ value: 'firmware', label: 'Firmware' },
				{ value: 'other', label: 'Other' }
			],
			placeholder: 'Any category'
		},
		{
			key: 'make_buy',
			label: 'Make/Buy',
			type: 'select',
			options: [
				{ value: 'make', label: 'Make' },
				{ value: 'buy', label: 'Buy' },
				{ value: 'modify', label: 'Modify' }
			],
			placeholder: 'Any'
		},
		{
			key: 'long_lead_only',
			label: 'Long lead items',
			type: 'boolean',
			trueLabel: 'Long lead only'
		},
		{
			key: 'single_source_only',
			label: 'Single source items',
			type: 'boolean',
			trueLabel: 'Single source only'
		}
	],
	quickFilters: [
		{
			id: 'buy-items',
			label: 'Buy Items',
			filters: { make_buy: 'buy' }
		},
		{
			id: 'long-lead',
			label: 'Long Lead',
			filters: { long_lead_only: true }
		}
	]
};

// ============================================================================
// Assemblies Filter Configuration
// ============================================================================

export const assembliesFilterConfig: EntityFilterConfig = {
	entityType: 'ASM',
	fields: [
		statusField,
		{
			key: 'top_level_only',
			label: 'Top-level only',
			type: 'boolean',
			trueLabel: 'Show top-level assemblies only'
		},
		{
			key: 'sub_only',
			label: 'Sub-assemblies only',
			type: 'boolean',
			trueLabel: 'Show sub-assemblies only'
		},
		{
			key: 'has_subassemblies',
			label: 'Has sub-assemblies',
			type: 'boolean',
			trueLabel: 'Show assemblies with sub-assemblies'
		},
		{
			key: 'empty_bom',
			label: 'Empty BOM',
			type: 'boolean',
			trueLabel: 'Show assemblies with no components'
		}
	],
	quickFilters: [
		{
			id: 'active',
			label: 'Active',
			filters: { status: ['draft', 'review', 'approved', 'released'] }
		},
		{
			id: 'top-level',
			label: 'Top Level',
			filters: { top_level_only: true }
		},
		{
			id: 'empty',
			label: 'Empty BOM',
			filters: { empty_bom: true }
		}
	]
};

// ============================================================================
// Tests Filter Configuration
// ============================================================================

export const testsFilterConfig: EntityFilterConfig = {
	entityType: 'TEST',
	fields: [
		statusField,
		{
			key: 'test_type',
			label: 'Test Type',
			type: 'select',
			options: [
				{ value: 'unit', label: 'Unit' },
				{ value: 'integration', label: 'Integration' },
				{ value: 'system', label: 'System' },
				{ value: 'acceptance', label: 'Acceptance' }
			],
			placeholder: 'Any type'
		},
		{
			key: 'level',
			label: 'V-Model Level',
			type: 'select',
			options: [
				{ value: 'component', label: 'Component' },
				{ value: 'subsystem', label: 'Subsystem' },
				{ value: 'system', label: 'System' },
				{ value: 'validation', label: 'Validation' }
			],
			placeholder: 'Any level'
		},
		{
			key: 'method',
			label: 'Method',
			type: 'select',
			options: [
				{ value: 'inspection', label: 'Inspection' },
				{ value: 'analysis', label: 'Analysis' },
				{ value: 'demonstration', label: 'Demonstration' },
				{ value: 'test', label: 'Test' }
			],
			placeholder: 'Any method'
		},
		{
			key: 'has_results',
			label: 'Has results',
			type: 'boolean',
			trueLabel: 'With results only'
		}
	],
	quickFilters: [
		{
			id: 'system-tests',
			label: 'System Tests',
			filters: { test_type: 'system' }
		},
		{
			id: 'no-results',
			label: 'No Results',
			filters: { has_results: false }
		}
	]
};

// ============================================================================
// Results Filter Configuration
// ============================================================================

export const resultsFilterConfig: EntityFilterConfig = {
	entityType: 'RSLT',
	fields: [
		statusField,
		{
			key: 'verdict',
			label: 'Verdict',
			type: 'select',
			options: [
				{ value: 'pass', label: 'Pass' },
				{ value: 'fail', label: 'Fail' },
				{ value: 'blocked', label: 'Blocked' },
				{ value: 'skip', label: 'Skip' }
			],
			placeholder: 'Any verdict'
		},
		{
			key: 'with_failures',
			label: 'Show failures only',
			type: 'boolean',
			trueLabel: 'Failures only'
		}
	],
	quickFilters: [
		{
			id: 'failures',
			label: 'Failures',
			filters: { verdict: 'fail' }
		},
		{
			id: 'passing',
			label: 'Passing',
			filters: { verdict: 'pass' }
		}
	]
};

// ============================================================================
// Hazards Filter Configuration
// ============================================================================

export const hazardsFilterConfig: EntityFilterConfig = {
	entityType: 'HAZ',
	fields: [
		statusField,
		{
			key: 'category',
			label: 'Category',
			type: 'select',
			options: [
				{ value: 'electrical', label: 'Electrical' },
				{ value: 'mechanical', label: 'Mechanical' },
				{ value: 'thermal', label: 'Thermal' },
				{ value: 'chemical', label: 'Chemical' },
				{ value: 'biological', label: 'Biological' },
				{ value: 'radiation', label: 'Radiation' },
				{ value: 'ergonomic', label: 'Ergonomic' }
			],
			placeholder: 'Any category'
		},
		{
			key: 'severity',
			label: 'Severity',
			type: 'select',
			options: [
				{ value: 'negligible', label: 'Negligible' },
				{ value: 'minor', label: 'Minor' },
				{ value: 'serious', label: 'Serious' },
				{ value: 'critical', label: 'Critical' },
				{ value: 'catastrophic', label: 'Catastrophic' }
			],
			placeholder: 'Any severity'
		},
		{
			key: 'has_risks',
			label: 'Has linked risks',
			type: 'boolean',
			trueLabel: 'With risks only'
		},
		{
			key: 'has_controls',
			label: 'Has controls',
			type: 'boolean',
			trueLabel: 'With controls only'
		}
	],
	quickFilters: [
		{
			id: 'uncontrolled',
			label: 'Uncontrolled',
			filters: { has_controls: false }
		},
		{
			id: 'critical',
			label: 'Critical/Catastrophic',
			filters: { severity: 'critical' }
		}
	]
};

// ============================================================================
// Deviations Filter Configuration
// ============================================================================

export const deviationsFilterConfig: EntityFilterConfig = {
	entityType: 'DEV',
	fields: [
		statusField,
		{
			key: 'dev_status',
			label: 'Deviation Status',
			type: 'select',
			options: [
				{ value: 'pending', label: 'Pending' },
				{ value: 'approved', label: 'Approved' },
				{ value: 'active', label: 'Active' },
				{ value: 'expired', label: 'Expired' },
				{ value: 'closed', label: 'Closed' },
				{ value: 'rejected', label: 'Rejected' }
			],
			placeholder: 'Any status'
		},
		{
			key: 'deviation_type',
			label: 'Type',
			type: 'select',
			options: [
				{ value: 'temporary', label: 'Temporary' },
				{ value: 'permanent', label: 'Permanent' },
				{ value: 'emergency', label: 'Emergency' }
			],
			placeholder: 'Any type'
		},
		{
			key: 'category',
			label: 'Category',
			type: 'select',
			options: [
				{ value: 'material', label: 'Material' },
				{ value: 'process', label: 'Process' },
				{ value: 'equipment', label: 'Equipment' },
				{ value: 'tooling', label: 'Tooling' },
				{ value: 'specification', label: 'Specification' },
				{ value: 'documentation', label: 'Documentation' }
			],
			placeholder: 'Any category'
		},
		{
			key: 'risk_level',
			label: 'Risk Level',
			type: 'select',
			options: [
				{ value: 'low', label: 'Low' },
				{ value: 'medium', label: 'Medium' },
				{ value: 'high', label: 'High' }
			],
			placeholder: 'Any level'
		},
		{
			key: 'active_only',
			label: 'Active only',
			type: 'boolean',
			trueLabel: 'Active only'
		}
	],
	quickFilters: [
		{
			id: 'active',
			label: 'Active',
			filters: { dev_status: 'active' }
		},
		{
			id: 'pending-approval',
			label: 'Pending Approval',
			filters: { dev_status: 'pending' }
		},
		{
			id: 'high-risk',
			label: 'High Risk',
			filters: { risk_level: 'high' }
		}
	]
};

// ============================================================================
// NCRs Filter Configuration
// ============================================================================

export const ncrsFilterConfig: EntityFilterConfig = {
	entityType: 'NCR',
	fields: [
		statusField,
		{
			key: 'ncr_type',
			label: 'NCR Type',
			type: 'select',
			options: [
				{ value: 'internal', label: 'Internal' },
				{ value: 'supplier', label: 'Supplier' },
				{ value: 'customer', label: 'Customer' }
			],
			placeholder: 'Any type'
		},
		{
			key: 'ncr_status',
			label: 'NCR Status',
			type: 'select',
			options: [
				{ value: 'open', label: 'Open' },
				{ value: 'containment', label: 'Containment' },
				{ value: 'investigation', label: 'Investigation' },
				{ value: 'disposition', label: 'Disposition' },
				{ value: 'closed', label: 'Closed' }
			],
			placeholder: 'Any status'
		},
		{
			key: 'severity',
			label: 'Severity',
			type: 'select',
			options: [
				{ value: 'minor', label: 'Minor' },
				{ value: 'major', label: 'Major' },
				{ value: 'critical', label: 'Critical' }
			],
			placeholder: 'Any severity'
		},
		{
			key: 'mrb_required',
			label: 'MRB Required',
			type: 'boolean',
			trueLabel: 'MRB Required'
		}
	],
	quickFilters: [
		{
			id: 'open',
			label: 'Open NCRs',
			filters: { ncr_status: 'open' }
		},
		{
			id: 'critical',
			label: 'Critical',
			filters: { severity: 'critical' }
		},
		{
			id: 'mrb',
			label: 'MRB Required',
			filters: { mrb_required: true }
		}
	]
};

// ============================================================================
// CAPAs Filter Configuration
// ============================================================================

export const capasFilterConfig: EntityFilterConfig = {
	entityType: 'CAPA',
	fields: [
		statusField,
		{
			key: 'capa_type',
			label: 'CAPA Type',
			type: 'select',
			options: [
				{ value: 'corrective', label: 'Corrective' },
				{ value: 'preventive', label: 'Preventive' }
			],
			placeholder: 'Any type'
		},
		{
			key: 'capa_status',
			label: 'CAPA Status',
			type: 'select',
			options: [
				{ value: 'initiation', label: 'Initiation' },
				{ value: 'investigation', label: 'Investigation' },
				{ value: 'implementation', label: 'Implementation' },
				{ value: 'verification', label: 'Verification' },
				{ value: 'closed', label: 'Closed' }
			],
			placeholder: 'Any status'
		},
		{
			key: 'overdue_only',
			label: 'Overdue only',
			type: 'boolean',
			trueLabel: 'Overdue only'
		},
		{
			key: 'open_only',
			label: 'Open only',
			type: 'boolean',
			trueLabel: 'Open only'
		}
	],
	quickFilters: [
		{
			id: 'open',
			label: 'Open CAPAs',
			filters: { open_only: true }
		},
		{
			id: 'overdue',
			label: 'Overdue',
			filters: { overdue_only: true }
		},
		{
			id: 'verification',
			label: 'Pending Verification',
			filters: { capa_status: 'verification' }
		}
	]
};

// ============================================================================
// Lots Filter Configuration
// ============================================================================

export const lotsFilterConfig: EntityFilterConfig = {
	entityType: 'LOT',
	fields: [
		statusField,
		{
			key: 'lot_status',
			label: 'Lot Status',
			type: 'select',
			options: [
				{ value: 'planned', label: 'Planned' },
				{ value: 'in_progress', label: 'In Progress' },
				{ value: 'on_hold', label: 'On Hold' },
				{ value: 'completed', label: 'Completed' },
				{ value: 'scrapped', label: 'Scrapped' }
			],
			placeholder: 'Any status'
		},
		{
			key: 'active_only',
			label: 'Active only',
			type: 'boolean',
			trueLabel: 'Active only'
		}
	],
	quickFilters: [
		{
			id: 'in-progress',
			label: 'In Progress',
			filters: { lot_status: 'in_progress' }
		},
		{
			id: 'on-hold',
			label: 'On Hold',
			filters: { lot_status: 'on_hold' }
		}
	]
};

// ============================================================================
// Features Filter Configuration
// ============================================================================

export const featuresFilterConfig: EntityFilterConfig = {
	entityType: 'FEAT',
	fields: [
		statusField,
		{
			key: 'feature_type',
			label: 'Feature Type',
			type: 'select',
			options: [
				{ value: 'internal', label: 'Internal' },
				{ value: 'external', label: 'External' }
			],
			placeholder: 'Any type'
		},
		{
			key: 'has_gdt',
			label: 'Has GD&T',
			type: 'boolean',
			trueLabel: 'With GD&T only'
		}
	],
	quickFilters: []
};

// ============================================================================
// Mates Filter Configuration
// ============================================================================

export const matesFilterConfig: EntityFilterConfig = {
	entityType: 'MATE',
	fields: [
		statusField,
		{
			key: 'fit_type',
			label: 'Fit Type',
			type: 'select',
			options: [
				{ value: 'clearance', label: 'Clearance' },
				{ value: 'transition', label: 'Transition' },
				{ value: 'interference', label: 'Interference' }
			],
			placeholder: 'Any type'
		}
	],
	quickFilters: [
		{
			id: 'interference',
			label: 'Interference Fits',
			filters: { fit_type: 'interference' }
		}
	]
};

// ============================================================================
// Stackups Filter Configuration
// ============================================================================

export const stackupsFilterConfig: EntityFilterConfig = {
	entityType: 'TOL',
	fields: [
		statusField,
		{
			key: 'result',
			label: 'Analysis Result',
			type: 'select',
			options: [
				{ value: 'pass', label: 'Pass' },
				{ value: 'marginal', label: 'Marginal' },
				{ value: 'fail', label: 'Fail' }
			],
			placeholder: 'Any result'
		},
		{
			key: 'critical_only',
			label: 'Critical only',
			type: 'boolean',
			trueLabel: 'Critical only'
		}
	],
	quickFilters: [
		{
			id: 'failing',
			label: 'Failing',
			filters: { result: 'fail' }
		},
		{
			id: 'critical',
			label: 'Critical',
			filters: { critical_only: true }
		}
	]
};

// ============================================================================
// Processes Filter Configuration
// ============================================================================

export const processesFilterConfig: EntityFilterConfig = {
	entityType: 'PROC',
	fields: [
		statusField,
		{
			key: 'process_type',
			label: 'Process Type',
			type: 'select',
			options: [
				{ value: 'machining', label: 'Machining' },
				{ value: 'assembly', label: 'Assembly' },
				{ value: 'inspection', label: 'Inspection' },
				{ value: 'test', label: 'Test' },
				{ value: 'finishing', label: 'Finishing' },
				{ value: 'packaging', label: 'Packaging' },
				{ value: 'handling', label: 'Handling' },
				{ value: 'heat_treat', label: 'Heat Treat' },
				{ value: 'welding', label: 'Welding' },
				{ value: 'coating', label: 'Coating' }
			],
			placeholder: 'Any type'
		},
		{
			key: 'has_equipment',
			label: 'Has equipment',
			type: 'boolean',
			trueLabel: 'With equipment only'
		}
	],
	quickFilters: []
};

// ============================================================================
// Controls Filter Configuration
// ============================================================================

export const controlsFilterConfig: EntityFilterConfig = {
	entityType: 'CTRL',
	fields: [
		statusField,
		{
			key: 'control_type',
			label: 'Control Type',
			type: 'select',
			options: [
				{ value: 'spc', label: 'SPC' },
				{ value: 'inspection', label: 'Inspection' },
				{ value: 'poka_yoke', label: 'Poka Yoke' },
				{ value: 'visual', label: 'Visual' },
				{ value: 'functional_test', label: 'Functional Test' },
				{ value: 'attribute', label: 'Attribute' }
			],
			placeholder: 'Any type'
		},
		{
			key: 'control_category',
			label: 'Category',
			type: 'select',
			options: [
				{ value: 'variable', label: 'Variable' },
				{ value: 'attribute', label: 'Attribute' }
			],
			placeholder: 'Any category'
		},
		{
			key: 'critical_only',
			label: 'Critical only',
			type: 'boolean',
			trueLabel: 'Critical only'
		}
	],
	quickFilters: [
		{
			id: 'spc',
			label: 'SPC Controls',
			filters: { control_type: 'spc' }
		},
		{
			id: 'critical',
			label: 'Critical',
			filters: { critical_only: true }
		}
	]
};

// ============================================================================
// Work Instructions Filter Configuration
// ============================================================================

export const workInstructionsFilterConfig: EntityFilterConfig = {
	entityType: 'WORK',
	fields: [
		statusField,
		{
			key: 'has_safety',
			label: 'Has safety requirements',
			type: 'boolean',
			trueLabel: 'With safety only'
		},
		{
			key: 'has_quality_checks',
			label: 'Has quality checks',
			type: 'boolean',
			trueLabel: 'With quality checks only'
		}
	],
	quickFilters: [
		{
			id: 'safety',
			label: 'Safety Critical',
			filters: { has_safety: true }
		}
	]
};

// ============================================================================
// Suppliers Filter Configuration
// ============================================================================

export const suppliersFilterConfig: EntityFilterConfig = {
	entityType: 'SUP',
	fields: [
		statusField,
		{
			key: 'capability',
			label: 'Capability',
			type: 'select',
			options: [
				{ value: 'machining', label: 'Machining' },
				{ value: 'sheet_metal', label: 'Sheet Metal' },
				{ value: 'casting', label: 'Casting' },
				{ value: 'injection', label: 'Injection' },
				{ value: 'extrusion', label: 'Extrusion' },
				{ value: 'pcb', label: 'PCB' },
				{ value: 'pcb_assembly', label: 'PCB Assembly' },
				{ value: 'cable_assembly', label: 'Cable Assembly' },
				{ value: 'assembly', label: 'Assembly' },
				{ value: 'testing', label: 'Testing' },
				{ value: 'finishing', label: 'Finishing' },
				{ value: 'packaging', label: 'Packaging' }
			],
			placeholder: 'Any capability'
		},
		{
			key: 'expired_certs',
			label: 'Expired certifications',
			type: 'boolean',
			trueLabel: 'With expired certs'
		}
	],
	quickFilters: [
		{
			id: 'machining',
			label: 'Machining',
			filters: { capability: 'machining' }
		},
		{
			id: 'pcb',
			label: 'PCB/PCBA',
			filters: { capability: 'pcb_assembly' }
		}
	]
};

// ============================================================================
// Quotes Filter Configuration
// ============================================================================

export const quotesFilterConfig: EntityFilterConfig = {
	entityType: 'QUOT',
	fields: [
		statusField,
		{
			key: 'quote_status',
			label: 'Quote Status',
			type: 'select',
			options: [
				{ value: 'pending', label: 'Pending' },
				{ value: 'received', label: 'Received' },
				{ value: 'accepted', label: 'Accepted' },
				{ value: 'rejected', label: 'Rejected' },
				{ value: 'expired', label: 'Expired' }
			],
			placeholder: 'Any status'
		},
		{
			key: 'expired_only',
			label: 'Expired only',
			type: 'boolean',
			trueLabel: 'Expired only'
		},
		{
			key: 'has_price_breaks',
			label: 'Has price breaks',
			type: 'boolean',
			trueLabel: 'With price breaks only'
		}
	],
	quickFilters: [
		{
			id: 'pending',
			label: 'Pending',
			filters: { quote_status: 'pending' }
		},
		{
			id: 'accepted',
			label: 'Accepted',
			filters: { quote_status: 'accepted' }
		}
	]
};

// ============================================================================
// Master lookup by entity type
// ============================================================================

export const filterConfigs: Record<string, EntityFilterConfig> = {
	REQ: requirementsFilterConfig,
	RISK: risksFilterConfig,
	CMP: componentsFilterConfig,
	ASM: assembliesFilterConfig,
	TEST: testsFilterConfig,
	RSLT: resultsFilterConfig,
	HAZ: hazardsFilterConfig,
	DEV: deviationsFilterConfig,
	NCR: ncrsFilterConfig,
	CAPA: capasFilterConfig,
	LOT: lotsFilterConfig,
	FEAT: featuresFilterConfig,
	MATE: matesFilterConfig,
	TOL: stackupsFilterConfig,
	PROC: processesFilterConfig,
	CTRL: controlsFilterConfig,
	WORK: workInstructionsFilterConfig,
	SUP: suppliersFilterConfig,
	QUOT: quotesFilterConfig
};

/**
 * Get filter configuration for an entity type
 */
export function getFilterConfig(entityType: string): EntityFilterConfig | undefined {
	return filterConfigs[entityType];
}
