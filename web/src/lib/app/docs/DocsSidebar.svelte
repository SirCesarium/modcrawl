<script lang="ts">
	import { page } from '$app/state';
	import { getDocHref } from './nav';
	import type { DocSection } from './types';

	interface Props {
		sections: DocSection[];
		open: boolean;
		onclose: () => void;
	}

	let { sections, open, onclose }: Props = $props();

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
		aria-label="Close documentation navigation"
	></button>
{/if}

<aside
	id="docs-sidebar"
	class="fixed top-14 bottom-0 left-0 z-40 w-72 overflow-y-auto border-r border-white/[0.07] bg-neutral-950 px-4 py-6 transition-transform duration-200 ease-out lg:sticky lg:inset-y-auto lg:top-14 lg:z-auto lg:h-[calc(100dvh-3.5rem)] lg:w-64 lg:shrink-0 lg:self-start lg:border-r lg:border-white/[0.07] lg:bg-transparent lg:px-4 lg:py-8 {open
		? 'translate-x-0'
		: '-translate-x-full lg:translate-x-0'}"
>
	<nav class="space-y-7" aria-label="Documentation navigation">
		{#each sections as section, sectionIndex (section.name ?? `section-${sectionIndex}`)}
			<div>
				{#if section.name}
					<p
						class="mb-2 px-3 text-[11px] font-semibold tracking-[0.14em] text-neutral-500 uppercase"
					>
						{section.name}
					</p>
				{/if}

				<ul class="space-y-0.5">
					{#each section.docs as doc (doc.slug)}
						{@const active = isDocActive(doc.slug)}
						<li>
							<a
								href={getDocHref(doc.slug)}
								onclick={onclose}
								aria-current={active ? 'page' : undefined}
								class="relative flex items-center rounded-md px-3 py-1.5 text-sm transition-colors {active
									? 'bg-white/6 font-medium text-neutral-100'
									: 'text-neutral-400 hover:bg-white/4 hover:text-neutral-200'}"
							>
								{#if active}
									<span
										aria-hidden="true"
										class="absolute top-1/2 left-0 h-4 w-0.5 -translate-y-1/2 rounded-full bg-purple-400"
									></span>
								{/if}
								<span class="truncate">{doc.metadata.title}</span>
							</a>
						</li>
					{/each}
				</ul>
			</div>
		{/each}
	</nav>
</aside>
<!-- eslint-enable svelte/no-navigation-without-resolve -->
