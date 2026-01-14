import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'light' | 'dark' | 'system';

function createThemeStore() {
	// Get initial theme from localStorage or default to system
	const getInitialTheme = (): Theme => {
		if (!browser) return 'system';
		const stored = localStorage.getItem('theme') as Theme;
		if (stored === 'light' || stored === 'dark' || stored === 'system') {
			return stored;
		}
		return 'system';
	};

	const { subscribe, set, update } = writable<Theme>(getInitialTheme());

	// Apply theme to document
	function applyTheme(theme: Theme) {
		if (!browser) return;

		const root = document.documentElement;
		let effectiveTheme: 'light' | 'dark';

		if (theme === 'system') {
			effectiveTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
		} else {
			effectiveTheme = theme;
		}

		root.classList.remove('light', 'dark');
		root.classList.add(effectiveTheme);
		localStorage.setItem('theme', theme);
	}

	// Subscribe to system preference changes
	if (browser) {
		const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		mediaQuery.addEventListener('change', () => {
			subscribe((current) => {
				if (current === 'system') {
					applyTheme('system');
				}
			})();
		});

		// Apply initial theme
		subscribe((theme) => applyTheme(theme))();
	}

	return {
		subscribe,
		set: (theme: Theme) => {
			set(theme);
			applyTheme(theme);
		},
		toggle: () => {
			update((current) => {
				const next = current === 'light' ? 'dark' : current === 'dark' ? 'system' : 'light';
				applyTheme(next);
				return next;
			});
		},
		setLight: () => {
			set('light');
			applyTheme('light');
		},
		setDark: () => {
			set('dark');
			applyTheme('dark');
		},
		setSystem: () => {
			set('system');
			applyTheme('system');
		}
	};
}

export const theme = createThemeStore();
