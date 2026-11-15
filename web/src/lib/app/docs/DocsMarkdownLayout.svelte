<script module>
	export { default as h1 } from './Heading1.svelte';
	export { default as h2 } from './Heading2.svelte';
	export { default as h3 } from './Heading3.svelte';
	export { default as h4 } from './Heading4.svelte';
	export { default as a } from './Link.svelte';
	export { default as hr } from './Rule.svelte';
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children?: Snippet;
	}

	let { children }: Props = $props();
</script>

<div class="max-w-none text-neutral-200">
	{#if children}
		{@render children()}
	{/if}
</div>

<style>
	/* Inline code in markdown — no language class, Prism doesn't touch it */
	:global(.max-w-none :not(pre) > code) {
		background: color-mix(in srgb, var(--color-purple-950) 40%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-purple-500) 25%, transparent);
		padding: 0.15em 0.4em;
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.875em;
		color: #a599e9;
	}

	/* Spacing lives here so the rule itself stays margin-free for header use */
	:global(.max-w-none hr) {
		margin: 2rem 0 1.5rem;
	}

	/* Tailwind preflight drops markers, so prose lists restore their own */
	:global(.max-w-none ul),
	:global(.max-w-none ol) {
		padding-left: 1.35rem;
		margin: 0.75rem 0;
	}

	/* `list-style: disc` is minified away as the initial value, so state the type */
	:global(.max-w-none ul) {
		list-style-type: disc;
	}

	:global(.max-w-none ul ul) {
		list-style: circle;
		margin: 0.25rem 0;
	}

	:global(.max-w-none ol) {
		list-style: decimal;
	}

	:global(.max-w-none ol ol) {
		list-style: lower-alpha;
		margin: 0.25rem 0;
	}

	:global(.max-w-none li) {
		margin: 0.25rem 0;
		padding-left: 0.15rem;
	}

	:global(.max-w-none li) > :global(p) {
		margin: 0;
	}

	:global(.max-w-none li)::marker {
		color: color-mix(in srgb, var(--color-purple-400) 85%, transparent);
	}
</style>
