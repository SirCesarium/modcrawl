<script lang="ts">
	import Button from '#app/components/Button.svelte';
	import { getDocHref } from './nav';
	import type { DocSection } from './types';

	interface Props {
		sections: DocSection[];
		currentSlug: string;
	}

	let { sections, currentSlug }: Props = $props();

	const pages = $derived(sections.flatMap((section) => section.docs));
	const index = $derived(pages.findIndex((doc) => doc.slug === currentSlug));
	const previous = $derived(index > 0 ? pages[index - 1] : undefined);
	const next = $derived(index >= 0 && index < pages.length - 1 ? pages[index + 1] : undefined);
</script>

{#if previous || next}
	<nav
		aria-label="Documentation pages"
		class="mt-12 flex items-center justify-between gap-4 border-t border-white/10 pt-6"
	>
		{#if previous}
			<Button
				href={getDocHref(previous.slug)}
				variant="ghost"
				color="neutral"
				size="sm"
				class="h-auto max-w-[45%] flex-col items-start gap-0.5 py-2 text-left"
				aria-label={`Previous: ${previous.metadata.title}`}
			>
				<span class="text-[11px] font-medium tracking-[0.12em] text-neutral-500 uppercase">
					← Previous
				</span>
				<span
					class="block max-w-full truncate text-sm text-neutral-200 transition-colors group-hover:text-purple-300"
				>
					{previous.metadata.title}
				</span>
			</Button>
		{:else}
			<span></span>
		{/if}

		{#if next}
			<Button
				href={getDocHref(next.slug)}
				variant="ghost"
				color="neutral"
				size="sm"
				class="h-auto max-w-[45%] flex-col items-end gap-0.5 py-2 text-right"
				aria-label={`Next: ${next.metadata.title}`}
			>
				<span class="text-[11px] font-medium tracking-[0.12em] text-neutral-500 uppercase">
					Next →
				</span>
				<span
					class="block max-w-full truncate text-sm text-neutral-200 transition-colors group-hover:text-purple-300"
				>
					{next.metadata.title}
				</span>
			</Button>
		{/if}
	</nav>
{/if}
