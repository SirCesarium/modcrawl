<script lang="ts">
	import Badge from '#app/components/Badge.svelte';
	import Button from '#app/components/Button.svelte';
	import Icon from '#app/components/Icon.svelte';
	import Typography from '#app/components/Typography.svelte';
	import CodeToolbar from '#app/docs/CodeToolbar.svelte';
	import DocsSidebar from '#app/docs/DocsSidebar.svelte';
	import { afterNavigate } from '$app/navigation';
	import { mount, unmount } from 'svelte';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();
	let sidebarOpen = $state(false);

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

<!-- eslint-disable svelte/no-navigation-without-resolve -- Internal docs sidebar navigation links -->
<div class="relative flex min-h-screen flex-col">
	<!-- Top Navigation Header -->
	<header
		class="sticky top-0 z-40 flex h-14 items-center justify-between border-b border-white/10 bg-neutral-950/90 px-4 backdrop-blur-md sm:h-16 sm:px-8"
	>
		<div class="flex items-center gap-3 sm:gap-4">
			<!-- Mobile sidebar toggle -->
			<button
				type="button"
				class="inline-flex items-center justify-center p-2 text-neutral-400 hover:bg-neutral-800 hover:text-white lg:hidden"
				onclick={() => (sidebarOpen = !sidebarOpen)}
				aria-label="Toggle navigation menu"
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

			<a href="/" class="flex items-center gap-2">
				<Icon name="modcrawl" size={20} alt="" aria-hidden="true" class="sm:size-6" />
				<Typography variant="h4" component="span" pixel uppercase class="text-base sm:text-xl">
					<span class="text-purple-300">mod</span>crawl
				</Typography>

				<Badge variant="muted" class="text-xs">docs</Badge>
			</a>
		</div>

		<div class="flex items-center gap-2">
			<Button href="/" variant="ghost" size="sm" pixel uppercase class="text-xs sm:text-sm">
				Home Page
			</Button>
		</div>
	</header>

	<div class="relative flex w-full flex-1">
		<DocsSidebar
			sections={data.sections}
			open={sidebarOpen}
			onclose={() => (sidebarOpen = false)}
		/>

		<main class="min-w-0 flex-1 px-4 py-6 sm:px-8 lg:py-10 xl:pr-12">
			{@render children()}
		</main>
	</div>
</div>
