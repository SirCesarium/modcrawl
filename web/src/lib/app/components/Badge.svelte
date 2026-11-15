<script lang="ts">
	import type { Snippet } from 'svelte';
	import { twMerge } from 'tailwind-merge';

	type Variant = 'accent' | 'muted' | 'topic';

	interface Props {
		/** Visual treatment: `accent` (page header), `muted` (sidebar), `topic` (chips). */
		variant?: Variant;
		/** Use the pixel font instead of monospace. */
		pixel?: boolean;
		class?: string;
		children?: Snippet;
		[key: string]: unknown;
	}

	let {
		variant = 'accent',
		pixel = true,
		class: className = '',
		children,
		...restProps
	}: Props = $props();

	const variantClasses: Record<Variant, string> = {
		accent:
			'border-purple-400/60 bg-purple-600/30 px-2.5 py-1 text-xs tracking-wider text-purple-100 uppercase shadow-[2px_2px_0_0_rgba(59,7,100,0.9),inset_0_1px_0_0_rgba(255,255,255,0.22)]',
		muted:
			'border-white/15 bg-neutral-900 px-2 py-0.5 text-[10px] text-neutral-300 shadow-[2px_2px_0_0_rgba(0,0,0,0.7),inset_0_1px_0_0_rgba(255,255,255,0.08)]',
		topic:
			'border-purple-400/30 bg-[#1e1e3f] px-2 py-0.5 text-xs text-purple-200 shadow-[2px_2px_0_0_rgba(0,0,0,0.7),inset_0_1px_0_0_rgba(255,255,255,0.08)]'
	};

	const computedClass = $derived(
		twMerge(
			'relative inline-flex items-center border leading-none whitespace-nowrap select-none',
			pixel ? 'font-pixel' : 'font-mono',
			variantClasses[variant],
			className
		)
	);
</script>

<span class={computedClass} {...restProps}>
	{#if children}
		{@render children()}
	{/if}
</span>
