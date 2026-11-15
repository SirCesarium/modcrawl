<script lang="ts">
	import { page } from '$app/state';
	import Typography from '#app/components/Typography.svelte';
	import CornerMarks from './CornerMarks.svelte';
	import type { DocSection } from './types';

	interface Props {
		sections: DocSection[];
		open: boolean;
		onclose: () => void;
	}

	let { sections, open, onclose }: Props = $props();

	function getDocHref(slug: string): string {
		return slug === 'home' ? '/docs' : `/docs/${slug}`;
	}

	function isDocActive(slug: string): boolean {
		const href = getDocHref(slug);
		const currentPath = page.url.pathname.replace(/\/$/, '') || '/';
		const targetPath = href.replace(/\/$/, '') || '/';
		return currentPath === targetPath;
	}
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- Internal docs sidebar navigation links -->
{#if open}
	<button
		type="button"
		class="fixed inset-0 z-30 bg-black/60 backdrop-blur-xs lg:hidden"
		onclick={onclose}
		aria-label="Close sidebar"
	></button>
{/if}

<aside
	class="fixed inset-y-14 left-0 z-35 w-80 border-r border-white/10 bg-neutral-950/95 p-4 transition-transform duration-200 lg:static lg:inset-y-auto lg:z-auto lg:w-88 lg:translate-x-0 lg:border-r lg:border-white/10 lg:bg-transparent lg:py-8 lg:pr-6 lg:pl-6 {open
		? 'translate-x-0'
		: '-translate-x-full'}"
>
	<div class="mb-4 flex items-center justify-between px-2">
		<Typography
			variant="overline"
			pixel
			class="truncate text-base font-semibold tracking-wide text-neutral-300"
		>
			Documentation
		</Typography>
	</div>

	<nav class="space-y-6" aria-label="Documentation navigation">
		{#each sections as section, sectionIndex (section.name ?? `section-${sectionIndex}`)}
			<div class="space-y-1.5">
				{#if section.name}
					<Typography variant="overline" pixel class="px-3 text-xs tracking-wide text-neutral-500">
						{section.name}
					</Typography>
				{/if}

				{#each section.docs as doc (doc.slug)}
					{@const active = isDocActive(doc.slug)}
					<div>
						<a
							href={getDocHref(doc.slug)}
							onclick={onclose}
							aria-current={active ? 'page' : undefined}
							class="group relative flex flex-col gap-0.5 px-3 py-2 text-sm transition-colors {active
								? 'bg-purple-900/30 text-purple-200 ring-1 ring-purple-500/40'
								: 'text-neutral-300 hover:bg-neutral-800/60 hover:text-white'}"
						>
							<CornerMarks />

							<div class="flex items-center justify-between gap-1.5">
								<Typography
									component="span"
									variant="body"
									pixel
									truncate
									class="text-sm font-medium {active ? 'text-purple-200' : 'text-inherit'}"
								>
									{doc.metadata.title}
								</Typography>
							</div>
							{#if doc.metadata.description}
								<Typography
									component="span"
									variant="caption"
									class="line-clamp-1 text-neutral-500 group-hover:text-neutral-400"
								>
									{doc.metadata.description}
								</Typography>
							{/if}
						</a>
					</div>
				{/each}
			</div>
		{/each}
	</nav>
</aside>
