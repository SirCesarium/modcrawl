<script module>
	export { default as h1 } from './Heading1.svelte';
	export { default as h2 } from './Heading2.svelte';
	export { default as h3 } from './Heading3.svelte';
	export { default as h4 } from './Heading4.svelte';
	export { default as a } from './Link.svelte';
	export { default as hr } from './Rule.svelte';
	export { default as table } from './Table.svelte';
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children?: Snippet;
	}

	let { children }: Props = $props();
</script>

<div data-docs-prose class="text-[15px] leading-7 text-neutral-300 sm:text-base">
	{#if children}
		{@render children()}
	{/if}
</div>

<style>
	:global([data-docs-prose] > :first-child) {
		margin-top: 0;
	}

	:global([data-docs-prose] > :last-child) {
		margin-bottom: 0;
	}

	:global([data-docs-prose] p) {
		margin: 1rem 0;
	}

	:global([data-docs-prose] strong) {
		color: var(--color-neutral-100);
		font-weight: 600;
	}

	:global([data-docs-prose] em) {
		font-style: italic;
	}

	/* Inline code stays neutral so it reads as a token, not as a highlighted surface. */
	:global([data-docs-prose] :not(pre) > code) {
		background: color-mix(in srgb, white 7%, transparent);
		border-radius: 0.25rem;
		padding: 0.1em 0.35em;
		color: var(--color-neutral-200);
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.875em;
	}

	:global([data-docs-prose] hr) {
		margin: 2.5rem 0;
		border: 0;
		border-top: 1px solid color-mix(in srgb, white 10%, transparent);
	}

	/* Tailwind preflight drops markers, so prose lists restore their own. */
	:global([data-docs-prose] ul),
	:global([data-docs-prose] ol) {
		margin: 1rem 0;
		padding-left: 1.25rem;
	}

	:global([data-docs-prose] ul) {
		list-style-type: disc;
	}

	:global([data-docs-prose] ul ul) {
		list-style: circle;
		margin: 0.25rem 0;
	}

	:global([data-docs-prose] ol) {
		list-style: decimal;
	}

	:global([data-docs-prose] ol ol) {
		list-style: lower-alpha;
		margin: 0.25rem 0;
	}

	:global([data-docs-prose] li) {
		margin: 0.35rem 0;
		padding-left: 0.15rem;
	}

	:global([data-docs-prose] li) > :global(p) {
		margin: 0;
	}

	:global([data-docs-prose] li)::marker {
		color: var(--color-neutral-500);
	}

	:global([data-docs-prose] blockquote) {
		margin: 1.5rem 0;
		border-left: 2px solid color-mix(in srgb, white 14%, transparent);
		padding-left: 1rem;
		color: var(--color-neutral-400);
	}

	:global([data-docs-prose] p img) {
		max-width: 100%;
		margin: 1.5rem 0;
		border: 1px solid color-mix(in srgb, white 10%, transparent);
		border-radius: 0.375rem;
	}

	:global([data-docs-prose] pre),
	:global([data-docs-prose] .overflow-x-auto) {
		scrollbar-width: thin;
		scrollbar-color: color-mix(in srgb, white 16%, transparent) transparent;
	}

	/* Preflight strips cell borders and padding, so markdown tables arrive unstyled. */
	:global([data-docs-prose] table) {
		width: 100%;
		font-size: 0.875rem;
	}

	:global([data-docs-prose] thead) {
		background: color-mix(in srgb, white 3%, transparent);
	}

	:global([data-docs-prose] th) {
		padding: 0.6rem 0.75rem;
		border-bottom: 1px solid color-mix(in srgb, white 10%, transparent);
		color: var(--color-neutral-400);
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: normal;
		text-align: left;
		text-transform: none;
		white-space: nowrap;
	}

	:global([data-docs-prose] td) {
		padding: 0.6rem 0.75rem;
		border-top: 1px solid color-mix(in srgb, white 6%, transparent);
		vertical-align: top;
	}

	:global([data-docs-prose] tbody tr:first-child td) {
		border-top: 0;
	}
</style>
