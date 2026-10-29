<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { SvelteHTMLElements } from 'svelte/elements';
	import { twMerge } from 'tailwind-merge';

	type Variant =
		| 'h1'
		| 'h2'
		| 'h3'
		| 'h4'
		| 'h5'
		| 'h6'
		| 'subtitle1'
		| 'subtitle2'
		| 'body'
		| 'caption'
		| 'overline';
	type Tag = keyof SvelteHTMLElements;

	interface Props {
		variant?: Variant;
		component?: Tag;
		pixel?: boolean;
		bold?: boolean;
		semibold?: boolean;
		medium?: boolean;
		light?: boolean;
		italic?: boolean;
		underline?: boolean;
		lineThrough?: boolean;
		uppercase?: boolean;
		lowercase?: boolean;
		capitalize?: boolean;
		truncate?: boolean;
		muted?: boolean;
		center?: boolean;
		right?: boolean;
		justify?: boolean;
		class?: string;
		children?: Snippet;
		[key: string]: unknown;
	}

	let {
		variant = 'body',
		component,
		pixel = false,
		bold = false,
		semibold = false,
		medium = false,
		light = false,
		italic = false,
		underline = false,
		lineThrough = false,
		uppercase = false,
		lowercase = false,
		capitalize = false,
		truncate = false,
		muted = false,
		center = false,
		right = false,
		justify = false,
		class: className = '',
		children,
		...restProps
	}: Props = $props();

	const defaultTags: Record<Variant, Tag> = {
		h1: 'h1',
		h2: 'h2',
		h3: 'h3',
		h4: 'h4',
		h5: 'h5',
		h6: 'h6',
		subtitle1: 'p',
		subtitle2: 'p',
		body: 'p',
		caption: 'span',
		overline: 'span'
	};

	const resolvedComponent = $derived(component ?? defaultTags[variant]);

	const variantClasses: Record<Variant, string> = {
		h1: 'text-4xl md:text-5xl font-extrabold tracking-tight',
		h2: 'text-3xl md:text-4xl font-bold tracking-tight',
		h3: 'text-2xl md:text-3xl font-semibold tracking-tight',
		h4: 'text-xl md:text-2xl font-semibold',
		h5: 'text-lg md:text-xl font-medium',
		h6: 'text-base md:text-lg font-medium',
		subtitle1: 'text-lg text-neutral-300',
		subtitle2: 'text-sm text-neutral-400',
		body: 'text-base text-neutral-100',
		caption: 'text-xs text-neutral-400',
		overline: 'text-xs uppercase tracking-widest text-neutral-400'
	};

	const computedClass = $derived(
		twMerge(
			variantClasses[variant],
			pixel && 'font-pixel',
			bold && 'font-bold',
			semibold && 'font-semibold',
			medium && 'font-medium',
			light && 'font-light',
			italic && 'italic',
			underline && 'underline underline-offset-4',
			lineThrough && 'line-through',
			uppercase && 'uppercase',
			lowercase && 'lowercase',
			capitalize && 'capitalize',
			truncate && 'truncate',
			muted && 'text-neutral-400',
			center && 'text-center',
			right && 'text-right',
			justify && 'text-justify',
			className
		)
	);
</script>

<svelte:element this={resolvedComponent} class={computedClass} {...restProps}>
	{#if children}
		{@render children()}
	{/if}
</svelte:element>
