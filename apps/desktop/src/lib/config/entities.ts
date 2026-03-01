/**
 * Shared entity type configuration: route paths, colors, and type lists.
 * Used across Header search, traceability components, and navigation.
 */

/** All entity type prefixes supported by the system */
export const ALL_ENTITY_TYPES = [
	'REQ', 'RISK', 'HAZ', 'TEST', 'RSLT', 'CMP', 'ASM', 'FEAT', 'MATE', 'TOL',
	'PROC', 'CTRL', 'WORK', 'LOT', 'DEV', 'NCR', 'CAPA', 'QUOT', 'SUP'
] as const;

/** Maps entity prefix to its route path segment (without leading slash) */
export const ENTITY_ROUTES: Record<string, string> = {
	REQ: 'requirements',
	RISK: 'risks',
	HAZ: 'hazards',
	TEST: 'verification/tests',
	RSLT: 'verification/results',
	CMP: 'components',
	ASM: 'assemblies',
	FEAT: 'features',
	MATE: 'mates',
	TOL: 'tolerances',
	PROC: 'manufacturing/processes',
	CTRL: 'controls',
	WORK: 'manufacturing/work-instructions',
	LOT: 'manufacturing/lots',
	DEV: 'manufacturing/deviations',
	NCR: 'quality/ncrs',
	CAPA: 'quality/capas',
	QUOT: 'procurement/quotes',
	SUP: 'procurement/suppliers'
};

/** Get the full route path for an entity detail page */
export function getEntityRoute(prefix: string, id: string): string {
	const route = ENTITY_ROUTES[prefix];
	return route ? `/${route}/${id}` : '/';
}

/** Get the route segment for an entity prefix (without leading slash) */
export function getEntityRouteSegment(prefix: string): string {
	return ENTITY_ROUTES[prefix] ?? 'entities';
}

/**
 * Bright, saturated entity colors for use on colored circle backgrounds (e.g., graph nodes).
 * Text is always white on these backgrounds.
 */
export const ENTITY_COLORS_SOLID: Record<string, string> = {
	REQ: 'bg-blue-500',
	RISK: 'bg-red-500',
	HAZ: 'bg-orange-500',
	TEST: 'bg-green-500',
	RSLT: 'bg-emerald-500',
	CMP: 'bg-purple-500',
	ASM: 'bg-violet-500',
	FEAT: 'bg-cyan-500',
	MATE: 'bg-teal-500',
	TOL: 'bg-indigo-500',
	PROC: 'bg-amber-500',
	CTRL: 'bg-yellow-500',
	WORK: 'bg-lime-500',
	LOT: 'bg-pink-500',
	DEV: 'bg-rose-500',
	NCR: 'bg-red-600',
	CAPA: 'bg-orange-600',
	QUOT: 'bg-sky-500',
	SUP: 'bg-slate-500'
};

/**
 * Muted entity colors with dark mode support for use on badges/chips in tables.
 */
export const ENTITY_COLORS_MUTED: Record<string, string> = {
	REQ: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
	RISK: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
	HAZ: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
	TEST: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
	RSLT: 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200',
	CMP: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
	ASM: 'bg-violet-100 text-violet-800 dark:bg-violet-900 dark:text-violet-200',
	FEAT: 'bg-cyan-100 text-cyan-800 dark:bg-cyan-900 dark:text-cyan-200',
	MATE: 'bg-teal-100 text-teal-800 dark:bg-teal-900 dark:text-teal-200',
	TOL: 'bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-200',
	PROC: 'bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200',
	CTRL: 'bg-cyan-100 text-cyan-800 dark:bg-cyan-900 dark:text-cyan-200',
	WORK: 'bg-lime-100 text-lime-800 dark:bg-lime-900 dark:text-lime-200',
	LOT: 'bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-200',
	DEV: 'bg-rose-100 text-rose-800 dark:bg-rose-900 dark:text-rose-200',
	NCR: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
	CAPA: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
	QUOT: 'bg-sky-100 text-sky-800 dark:bg-sky-900 dark:text-sky-200',
	SUP: 'bg-slate-100 text-slate-800 dark:bg-slate-900 dark:text-slate-200'
};

/** Get solid color class for an entity type (for circle badges, graph nodes) */
export function getEntityColorSolid(prefix: string): string {
	return ENTITY_COLORS_SOLID[prefix] ?? 'bg-gray-500';
}

/** Get muted color class for an entity type (for table badges, matrix headers) */
export function getEntityColorMuted(prefix: string): string {
	return ENTITY_COLORS_MUTED[prefix] ?? 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200';
}

/** Get status color classes for entity status badges */
export function getStatusColor(status: string): string {
	switch (status) {
		case 'approved':
		case 'released':
			return 'bg-green-500/20 text-green-400';
		case 'review':
			return 'bg-yellow-500/20 text-yellow-400';
		case 'draft':
			return 'bg-blue-500/20 text-blue-400';
		case 'obsolete':
			return 'bg-red-500/20 text-red-400';
		default:
			return 'bg-muted text-muted-foreground';
	}
}

/** Truncate a ULID-based entity ID for display */
export function truncateEntityId(id: string): string {
	const parts = id.split('-');
	if (parts.length === 2 && parts[1].length > 8) {
		return `${parts[0]}-${parts[1].slice(0, 6)}\u2026`;
	}
	return id;
}
