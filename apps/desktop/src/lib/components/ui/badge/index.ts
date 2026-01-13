import Root from './badge.svelte';
import { tv, type VariantProps } from 'tailwind-variants';

export const badgeVariants = tv({
	base: 'inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
	variants: {
		variant: {
			default: 'border-transparent bg-primary text-primary-foreground shadow hover:bg-primary/80',
			secondary: 'border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80',
			destructive: 'border-transparent bg-destructive text-destructive-foreground shadow hover:bg-destructive/80',
			outline: 'text-foreground',
			success: 'border-transparent bg-success/20 text-success',
			warning: 'border-transparent bg-warning/20 text-warning',
			info: 'border-transparent bg-info/20 text-info'
		}
	},
	defaultVariants: {
		variant: 'default'
	}
});

export type BadgeVariant = VariantProps<typeof badgeVariants>['variant'];

export {
	Root,
	Root as Badge
};
