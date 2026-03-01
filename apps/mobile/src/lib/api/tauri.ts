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
				{ id: 'LOT-01DEMO001', title: 'Cold Plate Sub-Assembly Lot 001', lot_number: 'LOT-2026-001', quantity: 50, lot_status: 'in_progress', status: 'draft', start_date: '2026-02-15', author: 'Jack Hale', created: '2026-02-14T18:30:00Z', entity_revision: 8 },
				{ id: 'LOT-01DEMO002', title: 'Cold Plate Sub-Assembly Lot 002', lot_number: 'LOT-2026-002', quantity: 25, lot_status: 'completed', status: 'draft', start_date: '2026-02-20', completion_date: '2026-02-28', author: 'Jack Hale', created: '2026-02-19T14:00:00Z', entity_revision: 21 },
				{ id: 'LOT-01DEMO003', title: 'Bracket Assembly Lot 003', lot_number: 'LOT-2026-003', quantity: 100, lot_status: 'on_hold', status: 'draft', start_date: '2026-02-25', author: 'Mike Chen', created: '2026-02-24T09:15:00Z', entity_revision: 5 },
				{ id: 'LOT-01DEMO004', title: 'Housing Machining Lot 004', lot_number: 'LOT-2026-004', quantity: 30, lot_status: 'in_progress', status: 'draft', start_date: '2026-02-27', author: 'Sarah Kim', created: '2026-02-26T11:00:00Z', entity_revision: 3 },
				{ id: 'LOT-01DEMO005', title: 'Connector Assembly Lot 005', lot_number: 'LOT-2026-005', quantity: 200, lot_status: 'completed', status: 'approved', start_date: '2026-01-10', completion_date: '2026-02-05', author: 'Jack Hale', created: '2026-01-09T08:00:00Z', entity_revision: 15 },
				{ id: 'LOT-01DEMO006', title: 'Prototype Enclosure Lot', lot_number: 'LOT-2026-006', quantity: 5, lot_status: 'scrapped', status: 'draft', start_date: '2026-02-01', author: 'Mike Chen', created: '2026-01-31T16:00:00Z', entity_revision: 7 },
				{ id: 'LOT-01DEMO007', title: 'Heatsink Fin Assembly Lot 007', lot_number: 'LOT-2026-007', quantity: 75, lot_status: 'in_progress', status: 'draft', start_date: '2026-03-01', author: 'Sarah Kim', created: '2026-02-28T13:45:00Z', entity_revision: 2 }
			],
			total_count: 7, has_more: false
		},
		list_ncrs: {
			items: [
				{ id: 'NCR-01DEMO001', title: 'Bore diameter out-of-spec on housing', ncr_number: 'NCR-2026-001', ncr_type: 'internal', severity: 'major', ncr_status: 'investigation', category: 'dimensional', status: 'draft', author: 'Mike Chen', created: '2026-02-22T14:30:00Z' },
				{ id: 'NCR-01DEMO002', title: 'Anodize coating thickness below spec on 3 units', ncr_number: 'NCR-2026-002', ncr_type: 'internal', severity: 'minor', ncr_status: 'disposition', category: 'process', status: 'draft', author: 'Sarah Kim', created: '2026-02-24T12:00:00Z' },
				{ id: 'NCR-01DEMO003', title: 'Incoming aluminum billet hardness non-conformance', ncr_number: 'NCR-2026-003', ncr_type: 'supplier', severity: 'critical', ncr_status: 'containment', category: 'material', status: 'draft', author: 'Jack Hale', created: '2026-02-26T09:00:00Z' }
			],
			total_count: 3, has_more: false
		},
		list_capas: {
			items: [
				{ id: 'CAPA-01DEMO001', title: 'Root cause: anodize coating thickness variation', capa_number: 'CAPA-2026-001', capa_type: 'corrective', capa_status: 'investigation', status: 'draft', due_date: '2026-03-15', author: 'Sarah Kim', created: '2026-02-25T10:00:00Z' },
				{ id: 'CAPA-01DEMO002', title: 'Prevent CNC tool wear dimensional drift', capa_number: 'CAPA-2026-002', capa_type: 'preventive', capa_status: 'implementation', status: 'draft', due_date: '2026-04-01', author: 'Mike Chen', created: '2026-02-23T15:00:00Z' }
			],
			total_count: 2, has_more: false
		},
		list_deviations: {
			items: [
				{ id: 'DEV-01DEMO001', title: 'Temporary material substitution - AL6061 to AL7075 for bracket', deviation_number: 'DEV-2026-001', deviation_type: 'temporary', category: 'material', risk_level: 'low', dev_status: 'active', status: 'approved', author: 'Jack Hale', created: '2026-02-10T08:00:00Z' },
				{ id: 'DEV-01DEMO002', title: 'Emergency: alternate anodize supplier due to capacity', deviation_number: 'DEV-2026-002', deviation_type: 'emergency', category: 'process', risk_level: 'medium', dev_status: 'pending', status: 'draft', author: 'Sarah Kim', created: '2026-02-27T16:00:00Z' }
			],
			total_count: 2, has_more: false
		},
		get_ncr_stats: { total: 3, by_ncr_status: { open: 1, containment: 1, investigation: 1, disposition: 0, closed: 0 }, by_type: { internal: 2, supplier: 1, customer: 0 }, by_severity: { minor: 1, major: 1, critical: 1 }, open: 3, total_cost: 2500 },
		get_capa_stats: { total: 2, by_capa_status: { initiation: 0, investigation: 1, implementation: 1, verification: 0, closed: 0 }, by_type: { corrective: 1, preventive: 1 }, open: 2, overdue: 0, verified_effective: 0 },
		get_lot_stats: { total: 7, by_status: { in_progress: 3, on_hold: 1, completed: 2, scrapped: 1 }, total_quantity: 485, avg_quantity: 69, with_git_branch: 0, merged_branches: 0 },
		get_deviation_stats: { total: 2, by_dev_status: { pending: 1, approved: 0, active: 1, expired: 0, closed: 0, rejected: 0 }, by_type: { temporary: 1, permanent: 0, emergency: 1 }, by_category: { material: 2, process: 0, equipment: 0, tooling: 0, specification: 0, documentation: 0 }, by_risk: { low: 1, medium: 1, high: 0 }, active: 1 },
		get_lot_next_step: 1,
		list_entities: { items: [], total_count: 0, has_more: false },
		search_entities: [],
		get_entity: null,
		get_entity_count: 0,
		get_links_from: [],
		get_links_to: [],
		get_link_types: ['satisfied_by', 'verified_by', 'allocated_to', 'mitigated_by', 'derived_from'],
		sync_cache: undefined,
	};

	// Handle specific lot get — return full lot with execution steps
	if (cmd === 'get_lot') {
		const id = args?.id as string;
		const lotMocks: Record<string, unknown> = {
			'LOT-01DEMO001': {
				id: 'LOT-01DEMO001',
				title: 'Cold Plate Sub-Assembly Lot 001',
				lot_number: 'LOT-2026-001',
				quantity: 50,
				lot_status: 'in_progress',
				start_date: '2026-02-15',
				notes: 'First production run of cold plate sub-assemblies for Q1 delivery',
				materials_used: [
					{ component: 'CMP-01DEMO001', supplier_lot: 'MC-2026-0142', quantity: 50 },
					{ component: 'CMP-01DEMO002', supplier_lot: 'SC-2026-0088', quantity: 200 }
				],
				execution: [
					{
						process: 'PROC-01DEMO001',
						process_revision: 1,
						work_instructions_used: [{ id: 'WORK-01DEMO001', revision: 3 }],
						status: 'completed',
						started_date: '2026-02-15',
						completed_date: '2026-02-18',
						operator: 'Mike Chen',
						operator_email: 'mchen@example.com',
						notes: 'CNC machining complete, all 50 units pass dimensional check',
						signature_verified: true,
						commit_sha: 'a1b2c3d4e5f6789012345678901234567890abcd',
						data: { channel_depth: '3.01mm', flatness: '0.02mm' },
						wi_step_executions: [
							{
								work_instruction: 'WORK-01DEMO001',
								step_number: 1,
								operator: 'Mike Chen',
								operator_email: 'mchen@example.com',
								completed_at: '2026-02-15T08:30:00Z',
								data: { material_cert: 'MC-2026-0142' },
								approval_status: 'not_required',
								notes: 'Verified material cert for 6061-T6 aluminum billet'
							},
							{
								work_instruction: 'WORK-01DEMO001',
								step_number: 2,
								operator: 'Mike Chen',
								operator_email: 'mchen@example.com',
								completed_at: '2026-02-15T09:15:00Z',
								data: { torque_value: 35.0, fixture_id: 'FIX-CNC-007' },
								approval_status: 'not_required',
								notes: 'Cold plate blank mounted in 5-axis fixture, torqued to 35 ft-lbs'
							},
							{
								work_instruction: 'WORK-01DEMO001',
								step_number: 3,
								operator: 'Mike Chen',
								operator_email: 'mchen@example.com',
								completed_at: '2026-02-16T14:20:00Z',
								data: { cnc_program: 'CPSA-001-R3' },
								equipment_used: { 'Haas VF-4SS': 'SN-44821' },
								approval_status: 'not_required',
								notes: 'Rough machining complete, program CPSA-001-R3'
							},
							{
								work_instruction: 'WORK-01DEMO001',
								step_number: 4,
								operator: 'Mike Chen',
								operator_email: 'mchen@example.com',
								completed_at: '2026-02-18T10:00:00Z',
								data: { length: '152.1mm', width: '101.8mm', depth: '24.9mm' },
								approvals: [{
									approver: 'Jack Hale',
									email: 'jhale@example.com',
									role: 'quality',
									timestamp: '2026-02-18T10:30:00Z',
									comment: 'Rough dimensions verified, within tolerance per drawing'
								}],
								approval_status: 'approved',
								notes: 'Rough dimension check - all within tolerance'
							}
						]
					},
					{
						process: 'PROC-01DEMO002',
						process_revision: 1,
						work_instructions_used: [{ id: 'WORK-01DEMO002', revision: 2 }],
						status: 'in_progress',
						started_date: '2026-02-19',
						operator: 'Sarah Kim',
						operator_email: 'skim@example.com',
						notes: 'Anodizing in progress — 30 of 50 units processed',
						wi_step_executions: [
							{
								work_instruction: 'WORK-01DEMO002',
								step_number: 1,
								operator: 'Sarah Kim',
								operator_email: 'skim@example.com',
								completed_at: '2026-02-19T07:45:00Z',
								data: { bath_time: '15min', bath_temp: '140F' },
								approval_status: 'not_required',
								notes: 'Alkaline degreaser bath - 15 min at 140F'
							},
							{
								work_instruction: 'WORK-01DEMO002',
								step_number: 2,
								operator: 'Sarah Kim',
								operator_email: 'skim@example.com',
								completed_at: '2026-02-19T09:00:00Z',
								data: { bath_time: '45min', bath_concentration: '15pct', voltage: '18V' },
								equipment_used: { 'Anodize Tank 2': 'AT-2-SN003' },
								approval_status: 'not_required',
								notes: 'Type III sulfuric acid anodize - 18V, 45 min'
							}
						]
					},
					{
						process: 'PROC-01DEMO003',
						process_revision: 1,
						work_instructions_used: [{ id: 'WORK-01DEMO003' }],
						status: 'pending',
						notes: null
					},
					{
						process: 'PROC-01DEMO004',
						process_revision: 1,
						work_instructions_used: [],
						status: 'pending',
						notes: null
					}
				],
				links: {
					product: 'ASM-01DEMO001',
					processes: ['PROC-01DEMO001', 'PROC-01DEMO002', 'PROC-01DEMO003', 'PROC-01DEMO004'],
					work_instructions: ['WORK-01DEMO001', 'WORK-01DEMO002', 'WORK-01DEMO003'],
					ncrs: [],
					results: []
				},
				status: 'draft',
				created: '2026-02-14T18:30:00Z',
				author: 'Jack Hale',
				entity_revision: 8
			},
			'LOT-01DEMO002': {
				id: 'LOT-01DEMO002',
				title: 'Cold Plate Sub-Assembly Lot 002',
				lot_number: 'LOT-2026-002',
				quantity: 25,
				lot_status: 'completed',
				start_date: '2026-02-20',
				completion_date: '2026-02-28',
				materials_used: [
					{ component: 'CMP-01DEMO001', supplier_lot: 'MC-2026-0198', quantity: 25 }
				],
				execution: [
					{
						process: 'PROC-01DEMO001',
						process_revision: 1,
						work_instructions_used: [{ id: 'WORK-01DEMO001', revision: 3 }],
						status: 'completed',
						started_date: '2026-02-20',
						completed_date: '2026-02-22',
						operator: 'Mike Chen',
						notes: 'CNC machining complete, all 25 units pass inspection',
						signature_verified: true,
						commit_sha: 'b2c3d4e5f67890123456789012345678901abcde',
						wi_step_executions: [
							{
								work_instruction: 'WORK-01DEMO001',
								step_number: 1,
								operator: 'Mike Chen',
								completed_at: '2026-02-20T08:00:00Z',
								data: { material_cert: 'MC-2026-0198' },
								approval_status: 'not_required',
								notes: 'Verified material cert for 6061-T6 aluminum billet'
							},
							{
								work_instruction: 'WORK-01DEMO001',
								step_number: 3,
								operator: 'Mike Chen',
								completed_at: '2026-02-21T15:30:00Z',
								data: { channel_depth: '3.02mm', flatness: '0.03mm' },
								equipment_used: { 'Haas VF-4SS': 'SN-44821' },
								approvals: [{
									approver: 'Jack Hale',
									role: 'quality',
									timestamp: '2026-02-21T16:00:00Z',
									comment: 'Dimensions verified per drawing TMS-CPSA-001 Rev D'
								}],
								approval_status: 'approved',
								notes: 'All 25 pieces within tolerance'
							}
						]
					},
					{
						process: 'PROC-01DEMO002',
						process_revision: 1,
						work_instructions_used: [{ id: 'WORK-01DEMO002', revision: 2 }],
						status: 'completed',
						started_date: '2026-02-23',
						completed_date: '2026-02-25',
						operator: 'Sarah Kim',
						notes: '22 of 25 units pass anodizing. 3 units segregated under NCR for rework.',
						commit_sha: 'c3d4e5f678901234567890123456789012abcdef',
						wi_step_executions: [
							{
								work_instruction: 'WORK-01DEMO002',
								step_number: 2,
								operator: 'Sarah Kim',
								completed_at: '2026-02-24T11:00:00Z',
								data: { units_pass: 22, units_fail: 3, coating_min: '40um', coating_max: '65um' },
								equipment_used: { 'Anodize Tank Line 2': 'AT2-SN-003' },
								approvals: [{
									approver: 'Jack Hale',
									role: 'quality',
									timestamp: '2026-02-24T11:45:00Z',
									comment: '22/25 pass. 3 units below 50um spec - segregate for NCR'
								}],
								approval_status: 'approved',
								notes: 'Type III anodize complete. Coating measured 55-65um on 22/25 parts'
							}
						]
					},
					{
						process: 'PROC-01DEMO003',
						process_revision: 1,
						work_instructions_used: [{ id: 'WORK-01DEMO003' }],
						status: 'completed',
						started_date: '2026-02-26',
						completed_date: '2026-02-27',
						operator: 'Mike Chen',
						notes: 'Final assembly and inspection complete',
						commit_sha: 'd4e5f6789012345678901234567890123abcdef0'
					},
					{
						process: 'PROC-01DEMO004',
						process_revision: 1,
						work_instructions_used: [],
						status: 'completed',
						started_date: '2026-02-28',
						completed_date: '2026-02-28',
						operator: 'Sarah Kim',
						notes: 'Packaging and labeling complete, lot shipped',
						commit_sha: 'e5f67890123456789012345678901234abcdef01'
					}
				],
				links: {
					product: 'ASM-01DEMO001',
					processes: ['PROC-01DEMO001', 'PROC-01DEMO002', 'PROC-01DEMO003', 'PROC-01DEMO004'],
					ncrs: ['NCR-01DEMO002'],
					results: ['RSLT-01DEMO001']
				},
				status: 'draft',
				created: '2026-02-19T14:00:00Z',
				author: 'Jack Hale',
				entity_revision: 21
			},
			'LOT-01DEMO003': {
				id: 'LOT-01DEMO003',
				title: 'Bracket Assembly Lot 003',
				lot_number: 'LOT-2026-003',
				quantity: 100,
				lot_status: 'on_hold',
				start_date: '2026-02-25',
				notes: 'On hold pending supplier material cert verification',
				execution: [
					{
						process: 'PROC-01DEMO001',
						process_revision: 1,
						work_instructions_used: [{ id: 'WORK-01DEMO001' }],
						status: 'completed',
						started_date: '2026-02-25',
						completed_date: '2026-02-26',
						operator: 'Mike Chen',
						notes: 'Machining complete on first 40 units'
					},
					{
						process: 'PROC-01DEMO002',
						process_revision: 1,
						work_instructions_used: [{ id: 'WORK-01DEMO002' }],
						status: 'in_progress',
						started_date: '2026-02-27',
						operator: 'Sarah Kim',
						notes: 'HOLD: Anodizing paused — awaiting material cert from supplier'
					},
					{
						process: 'PROC-01DEMO003',
						process_revision: 1,
						status: 'pending'
					}
				],
				links: {
					product: 'ASM-01DEMO002',
					processes: ['PROC-01DEMO001', 'PROC-01DEMO002', 'PROC-01DEMO003'],
					ncrs: [],
					deviations: ['DEV-01DEMO001'],
					results: []
				},
				status: 'draft',
				created: '2026-02-24T09:15:00Z',
				author: 'Mike Chen',
				entity_revision: 5
			},
			'LOT-01DEMO004': {
				id: 'LOT-01DEMO004',
				title: 'Housing Machining Lot 004',
				lot_number: 'LOT-2026-004',
				quantity: 30,
				lot_status: 'in_progress',
				start_date: '2026-02-27',
				execution: [
					{
						process: 'PROC-01DEMO001',
						process_revision: 2,
						work_instructions_used: [{ id: 'WORK-01DEMO001', revision: 3 }],
						status: 'in_progress',
						started_date: '2026-02-27',
						operator: 'Mike Chen',
						notes: 'Running CNC program — 12 of 30 units completed',
						wi_step_executions: [
							{
								work_instruction: 'WORK-01DEMO001',
								step_number: 1,
								operator: 'Mike Chen',
								completed_at: '2026-02-27T07:30:00Z',
								data: { material_cert: 'MC-2026-0215' },
								approval_status: 'not_required',
								notes: 'Material loaded and cert verified'
							},
							{
								work_instruction: 'WORK-01DEMO001',
								step_number: 2,
								operator: 'Mike Chen',
								completed_at: '2026-02-27T08:00:00Z',
								data: { fixture_id: 'FIX-CNC-012', torque_value: 40.0 },
								approval_status: 'not_required',
								notes: 'Fixture mounted and torqued'
							}
						]
					},
					{
						process: 'PROC-01DEMO002',
						process_revision: 1,
						status: 'pending'
					},
					{
						process: 'PROC-01DEMO004',
						process_revision: 1,
						status: 'pending'
					}
				],
				links: {
					product: 'CMP-01DEMO003',
					processes: ['PROC-01DEMO001', 'PROC-01DEMO002', 'PROC-01DEMO004'],
					ncrs: ['NCR-01DEMO001'],
					deviations: ['DEV-01DEMO002']
				},
				status: 'draft',
				created: '2026-02-26T11:00:00Z',
				author: 'Sarah Kim',
				entity_revision: 3
			}
		};
		const match = lotMocks[id];
		if (match) return match as T;
		// Fallback for unknown lot IDs
		const listItems = (mocks['list_lots'] as { items: unknown[] }).items;
		const listMatch = listItems.find((l: unknown) => (l as Record<string, string>).id === id);
		return (listMatch ?? { id, title: 'Unknown Lot', lot_status: 'in_progress', status: 'draft', quantity: 0, author: 'Demo User', created: new Date().toISOString(), execution: [] }) as T;
	}
	if (cmd === 'get_ncr') {
		const ncrId = args?.id as string;
		const ncrMocks: Record<string, unknown> = {
			'NCR-01DEMO001': { id: 'NCR-01DEMO001', title: 'Bore diameter out-of-spec on cold plate housing units', ncr_number: 'NCR-2026-001', ncr_status: 'investigation', ncr_type: 'internal', severity: 'major', category: 'dimensional', description: 'Housing bore diameter measured at 25.12mm, spec is 25.00 +/- 0.05mm. 3 of 30 parts affected in LOT-2026-004.', lot_ids: ['LOT-01DEMO004'], author: 'Mike Chen', created: '2026-02-22T14:30:00Z', status: 'draft', entity_revision: 3 },
			'NCR-01DEMO002': { id: 'NCR-01DEMO002', title: 'Anodize coating thickness below minimum spec on 3 units', ncr_number: 'NCR-2026-002', ncr_status: 'open', ncr_type: 'internal', severity: 'minor', category: 'process', description: 'Anodize coating thickness measured at 42um on 3 units, minimum spec is 50um. Lot 002, anodize step.', lot_ids: ['LOT-01DEMO002'], author: 'Sarah Kim', created: '2026-02-24T11:00:00Z', status: 'draft', entity_revision: 2 },
			'NCR-01DEMO003': { id: 'NCR-01DEMO003', title: 'Incoming aluminum billet hardness non-conformance', ncr_number: 'NCR-2026-003', ncr_status: 'containment', ncr_type: 'supplier', severity: 'critical', category: 'material', description: 'Incoming inspection: Brinell hardness on 5 billets from supplier lot SL-2026-088 measured 58-62 HB, spec requires 65-75 HB.', lot_ids: [], author: 'Jack Hale', created: '2026-02-26T09:00:00Z', status: 'draft', entity_revision: 1 }
		};
		return (ncrMocks[ncrId] ?? { id: ncrId, title: 'Unknown NCR', ncr_status: 'open', severity: 'minor', status: 'draft', author: 'Unknown', created: new Date().toISOString() }) as T;
	}
	if (cmd === 'get_capa') {
		return { id: args?.id, title: 'Root cause: anodize coating thickness variation', capa_number: 'CAPA-2026-001', capa_status: 'investigation', capa_type: 'corrective', description: 'Investigating root cause of anodize coating thickness below 50um spec on 3 units from LOT-2026-002. Suspect bath concentration drift over shift.', source_ncr: 'NCR-01DEMO002', author: 'Sarah Kim', created: '2026-02-25T10:00:00Z', due_date: '2026-03-15', status: 'draft', entity_revision: 2 } as T;
	}
	if (cmd === 'get_deviation') {
		const devId = args?.id as string;
		const devMocks: Record<string, unknown> = {
			'DEV-01DEMO001': { id: 'DEV-01DEMO001', title: 'Temporary material substitution - AL6061 to AL7075 for bracket', deviation_number: 'DEV-2026-001', dev_status: 'active', deviation_type: 'temporary', category: 'material', risk_level: 'low', description: 'Temporary substitution of AL6061-T6 with AL7075-T6 for bracket P/N 10042 due to supplier stock-out. AL7075 meets or exceeds all mechanical requirements. Valid for LOT-2026-003 only.', approved_by: 'Jack Hale', approval_date: '2026-02-11', effective_date: '2026-02-12', expiration_date: '2026-04-01', lots: ['LOT-01DEMO003'], author: 'Jack Hale', created: '2026-02-10T08:00:00Z', status: 'approved', entity_revision: 4 },
			'DEV-01DEMO002': { id: 'DEV-01DEMO002', title: 'Emergency: alternate anodize supplier due to capacity', deviation_number: 'DEV-2026-002', dev_status: 'pending', deviation_type: 'emergency', category: 'process', risk_level: 'medium', description: 'Primary anodize supplier at capacity. Temporary switch to AnoTech Corp for LOT-2026-004 anodize step. AnoTech is AS9100 certified but not yet qualified for this part.', lots: ['LOT-01DEMO004'], author: 'Sarah Kim', created: '2026-02-27T16:00:00Z', status: 'draft', entity_revision: 1 }
		};
		return (devMocks[devId] ?? { id: devId, title: 'Unknown Deviation', dev_status: 'pending', risk_level: 'low', status: 'draft', author: 'Unknown', created: new Date().toISOString() }) as T;
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
