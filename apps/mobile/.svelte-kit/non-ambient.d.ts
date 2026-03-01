
// this file is generated — do not edit it


declare module "svelte/elements" {
	export interface HTMLAttributes<T> {
		'data-sveltekit-keepfocus'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-noscroll'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-preload-code'?:
			| true
			| ''
			| 'eager'
			| 'viewport'
			| 'hover'
			| 'tap'
			| 'off'
			| undefined
			| null;
		'data-sveltekit-preload-data'?: true | '' | 'hover' | 'tap' | 'off' | undefined | null;
		'data-sveltekit-reload'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-replacestate'?: true | '' | 'off' | undefined | null;
	}
}

export {};


declare module "$app/types" {
	export interface AppTypes {
		RouteId(): "/" | "/browse" | "/browse/entity" | "/browse/entity/[id]" | "/browse/[type]" | "/lots" | "/lots/[id]" | "/more" | "/more/deviations" | "/more/deviations/new" | "/more/deviations/[id]" | "/more/traceability" | "/project" | "/quality" | "/quality/capas" | "/quality/capas/[id]" | "/quality/ncrs" | "/quality/ncrs/new" | "/quality/ncrs/[id]";
		RouteParams(): {
			"/browse/entity/[id]": { id: string };
			"/browse/[type]": { type: string };
			"/lots/[id]": { id: string };
			"/more/deviations/[id]": { id: string };
			"/quality/capas/[id]": { id: string };
			"/quality/ncrs/[id]": { id: string }
		};
		LayoutParams(): {
			"/": { id?: string; type?: string };
			"/browse": { id?: string; type?: string };
			"/browse/entity": { id?: string };
			"/browse/entity/[id]": { id: string };
			"/browse/[type]": { type: string };
			"/lots": { id?: string };
			"/lots/[id]": { id: string };
			"/more": { id?: string };
			"/more/deviations": { id?: string };
			"/more/deviations/new": Record<string, never>;
			"/more/deviations/[id]": { id: string };
			"/more/traceability": Record<string, never>;
			"/project": Record<string, never>;
			"/quality": { id?: string };
			"/quality/capas": { id?: string };
			"/quality/capas/[id]": { id: string };
			"/quality/ncrs": { id?: string };
			"/quality/ncrs/new": Record<string, never>;
			"/quality/ncrs/[id]": { id: string }
		};
		Pathname(): "/" | "/browse" | `/browse/entity/${string}` & {} | `/browse/${string}` & {} | "/lots" | `/lots/${string}` & {} | "/more" | "/more/deviations" | "/more/deviations/new" | `/more/deviations/${string}` & {} | "/more/traceability" | "/project" | "/quality" | "/quality/capas" | `/quality/capas/${string}` & {} | "/quality/ncrs" | "/quality/ncrs/new" | `/quality/ncrs/${string}` & {};
		ResolvedPathname(): `${"" | `/${string}`}${ReturnType<AppTypes['Pathname']>}`;
		Asset(): string & {};
	}
}