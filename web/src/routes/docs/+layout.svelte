<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import { page } from '$app/state';
	import Badge from '#app/components/Badge.svelte';
	import Button from '#app/components/Button.svelte';
	import Icon from '#app/components/Icon.svelte';
	import CodeToolbar from '#app/docs/CodeToolbar.svelte';
	import DocsPager from '#app/docs/DocsPager.svelte';
	import DocsSidebar from '#app/docs/DocsSidebar.svelte';
	import { mount, unmount } from 'svelte';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();
	let sidebarOpen = $state(false);

	const currentSlug = $derived.by(() => {
		const path = page.url.pathname.replace(/\/+$/, '');

		if (path === '/docs') return 'home';
		return path.startsWith('/docs/') ? path.slice('/docs/'.length) : 'home';
	});

	type ToolbarInstance = { component: ReturnType<typeof mount>; target: HTMLElement };
	let toolbars: ToolbarInstance[] = [];

	// Markdown code blocks arrive as raw HTML, so the Svelte toolbar mounts into their slot.
	afterNavigate(() => {
		toolbars = toolbars.filter(({ component, target }) => {
			if (target.isConnected) return true;
			unmount(component);
			return false;
		});

		for (const target of document.querySelectorAll<HTMLElement>('.code-toolbar')) {
			if (target.dataset.mounted) continue;
			target.dataset.mounted = 'true';
			toolbars.push({
				component: mount(CodeToolbar, {
					target,
					props: { language: target.dataset.language ?? '' }
				}),
				target
			});
		}
	});
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- Internal docs navigation links -->
<a
	href="#doc-content"
	class="sr-only font-sans focus:not-sr-only focus:absolute focus:top-3 focus:left-3 focus:z-60 focus:rounded-md focus:bg-neutral-900 focus:px-3 focus:py-2 focus:text-sm focus:text-neutral-100 focus:ring-1 focus:ring-white/20"
>
	Skip to content
</a>

<div class="relative flex min-h-screen flex-col font-sans">
	<header class="sticky top-0 z-50 border-b border-white/[0.07] bg-neutral-950/85 backdrop-blur-md">
		<div class="mx-auto flex h-14 w-full max-w-400 items-center justify-between gap-4 px-4 sm:px-6">
			<div class="flex min-w-0 items-center gap-2">
				<button
					type="button"
					class="-ml-1 inline-flex items-center justify-center rounded-md p-2 text-neutral-400 transition-colors hover:bg-white/5 hover:text-neutral-100 lg:hidden"
					onclick={() => (sidebarOpen = !sidebarOpen)}
					aria-label="Toggle documentation navigation"
					aria-expanded={sidebarOpen}
					aria-controls="docs-sidebar"
				>
					<svg
						class="size-5"
						fill="none"
						viewBox="0 0 24 24"
						stroke-width="1.5"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d={sidebarOpen
								? 'M6 18L18 6M6 6l12 12'
								: 'M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5'}
						/>
					</svg>
				</button>

				<a href="/" class="flex min-w-0 items-center gap-2">
					<Icon name="modcrawl" size={20} alt="" aria-hidden="true" class="shrink-0 sm:size-6" />
					<!-- Silkscreen caps sit low in the line box; match the logo and badge optical centers. -->
					<span
						class="font-pixel -translate-y-px text-base tracking-wide whitespace-nowrap uppercase sm:text-lg"
					>
						<span class="text-purple-300">mod</span>crawl
					</span>
					<Badge variant="muted" class="text-xs">docs</Badge>
				</a>
			</div>

			<Button
				href="/"
				variant="ghost"
				size="sm"
				pixel
				class="shrink-0 text-neutral-100"
				aria-label="Back to app"
			>
				<span class="sm:hidden">Back</span>
				<span class="hidden sm:inline">Back to app</span>
			</Button>
		</div>
	</header>

	<div class="mx-auto flex w-full max-w-400 flex-1">
		<DocsSidebar
			sections={data.sections}
			open={sidebarOpen}
			onclose={() => (sidebarOpen = false)}
		/>

		<main id="doc-content" class="min-w-0 flex-1 px-5 py-8 sm:px-8 lg:py-12 xl:px-12">
			<div class="mx-auto w-full max-w-172">
				{@render children()}

				<DocsPager sections={data.sections} {currentSlug} />
			</div>
		</main>
	</div>
</div>
<!-- eslint-enable svelte/no-navigation-without-resolve -->
