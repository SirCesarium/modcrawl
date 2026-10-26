<script lang="ts">
	import { onMount } from 'svelte';
	import AppBar from '#lib/app/widgets/AppBar.svelte';
	import Logo from '#lib/app/widgets/Logo.svelte';
	import ScrollHint from '#lib/app/widgets/ScrollHint.svelte';
	import Code from '#lib/components/Code.svelte';
	import Typography from '#lib/components/Typography.svelte';

	let heroBrand: HTMLDivElement;
	let heroBrandVisible = $state(true);

	onMount(() => {
		const observer = new IntersectionObserver(
			(entries) => {
				const entry = entries[0];

				if (entry) heroBrandVisible = entry.isIntersecting;
			},
			{ rootMargin: '-64px 0px 0px 0px' }
		);

		observer.observe(heroBrand);

		return () => observer.disconnect();
	});
</script>

<AppBar showLogo={!heroBrandVisible} />

<main class="pt-16">
	<section
		class="relative isolate flex min-h-[calc(100svh-4rem)] flex-col items-center justify-center gap-8 px-5 pt-8 pb-24 sm:gap-10 sm:px-8 sm:pt-12 lg:flex-row lg:justify-between lg:gap-8 lg:px-12 lg:py-0 xl:px-24 2xl:px-40"
	>
		<div class="pointer-events-none absolute inset-0 -z-10" aria-hidden="true">
			<div
				class="absolute top-1/2 left-[10%] h-144 w-xl animate-fog bg-[radial-gradient(circle,rgba(179,92,255,0.16),rgba(179,92,255,0.05)_38%,transparent_68%)]"
			></div>
			<div
				class="absolute top-[20%] left-[34%] h-120 w-2xl animate-fog bg-[radial-gradient(circle,rgba(179,92,255,0.1),rgba(179,92,255,0.03)_40%,transparent_70%)] [animation-delay:-8s]"
			></div>
			<div
				class="absolute right-[4%] -bottom-40 h-120 w-120 animate-fog bg-[radial-gradient(circle,rgba(254,128,25,0.1),rgba(254,128,25,0.03)_40%,transparent_70%)] [animation-delay:-14s]"
			></div>
		</div>

		<div class="flex w-full max-w-xl flex-col items-center gap-4 lg:w-auto">
			<div class="flex flex-col items-center gap-2 sm:flex-row sm:gap-4" bind:this={heroBrand}>
				<Logo />
				<Typography variant="h1" component="span" pixel uppercase class="relative z-10">
					<span class="text-purple-300">mod</span>crawl
				</Typography>
			</div>
			<Typography
				variant="h5"
				component="span"
				pixel
				uppercase
				class="max-w-xs text-center text-sm leading-5 sm:max-w-none sm:text-lg sm:leading-7 md:text-xl"
			>
				Analyze Minecraft mods and plugins!
			</Typography>
		</div>
		<div class="w-full max-w-md">
			<!-- TODO: replace this with drop folder/jar(s) -->
			<Code
				language="java"
				code={`
public class HelloWorld {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}`}
			/>
		</div>

		<ScrollHint />
	</section>

	<div
		class="mx-auto flex max-w-3xl flex-col gap-8 px-5 pt-12 pb-24 text-sm leading-relaxed text-pretty text-neutral-300 sm:px-8 sm:text-base"
	>
		<section>
			Lorem ipsum dolor sit amet consectetur adipisicing elit. Dolorum unde reiciendis quam, quos
			soluta similique, molestias animi nihil hic aut autem iste omnis non repudiandae architecto
			doloribus? Nobis, veritatis iure.
		</section>
	</div>
</main>
