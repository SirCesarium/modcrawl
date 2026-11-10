<script lang="ts">
	import { onMount } from 'svelte';
	import AppBar from '#app/widgets/AppBar.svelte';
	import Logo from '#app/widgets/Logo.svelte';
	import ScrollHint from '#app/widgets/ScrollHint.svelte';
	import Typography from '#app/components/Typography.svelte';
	import Button from '#app/components/Button.svelte';
	import FileInput from '#app/components/FileInput.svelte';

	let heroBrand: HTMLDivElement;
	let heroBrandVisible = $state(true);
	let modFiles = $state<File[]>([]);
	let modFolderFiles = $state<File[]>([]);

	onMount(() => {
		const observer = new IntersectionObserver(
			(entries) => {
				const entry = entries[0];

				if (entry) heroBrandVisible = entry.isIntersecting;
			},
			{ rootMargin: '-56px 0px 0px 0px' }
		);

		observer.observe(heroBrand);

		return () => observer.disconnect();
	});
</script>

<AppBar showLogo={!heroBrandVisible} />

<main class="pt-14 sm:pt-16">
	<section
		class="relative isolate flex min-h-[calc(100svh-3.5rem)] flex-col items-center justify-center gap-8 px-5 pt-8 pb-24 sm:min-h-[calc(100svh-4rem)] sm:gap-10 sm:px-8 sm:pt-12 lg:flex-row lg:justify-between lg:gap-8 lg:px-12 lg:py-0 xl:px-24 2xl:px-40"
	>
		<div class="pointer-events-none absolute inset-0 -z-10" aria-hidden="true">
			<div
				class="absolute top-1/2 left-1/2 h-96 w-[85vw] -translate-1/2 animate-fog bg-[radial-gradient(ellipse_at_center,rgba(179,92,255,0.16)_0%,rgba(179,92,255,0.05)_38%,transparent_70%)] sm:h-144 sm:w-xl lg:left-[10%] lg:translate-x-0"
			></div>
			<div
				class="absolute top-[20%] left-1/2 h-96 w-[90vw] -translate-x-1/2 animate-fog bg-[radial-gradient(ellipse_at_center,rgba(179,92,255,0.1)_0%,rgba(179,92,255,0.03)_40%,transparent_70%)] [animation-delay:-8s] sm:h-120 sm:w-2xl lg:left-[34%] lg:translate-x-0"
			></div>
			<div
				class="absolute right-0 -bottom-24 size-96 animate-fog bg-[radial-gradient(ellipse_at_center,rgba(254,128,25,0.1)_0%,rgba(254,128,25,0.03)_40%,transparent_70%)] [animation-delay:-14s] sm:right-[4%] sm:-bottom-40 sm:size-120"
			></div>
		</div>

		<div class="flex w-full max-w-xl flex-col items-center gap-4 lg:w-auto">
			<div class="flex flex-col items-center gap-2 sm:flex-row sm:gap-4" bind:this={heroBrand}>
				<Logo />
				<Typography
					variant="h1"
					component="span"
					pixel
					uppercase
					class="relative z-10 drop-shadow-[-5px_5px_5px_rgba(0,0,0,1)]"
				>
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
		<div class="flex w-full max-w-lg flex-col gap-6">
			<Typography variant="h5" component="h2" pixel uppercase class="text-base sm:text-lg">
				Analyze your mods
			</Typography>
			<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
				<FileInput
					bind:files={modFiles}
					accept=".jar"
					label="Mods"
					placeholder="Drop one or more .jar mods to analyze"
					height="h-48"
				/>
				<FileInput
					bind:files={modFolderFiles}
					folder
					label="&quot;mods/&quot; Folder"
					placeholder="Drop the mod folder to analyze"
					height="h-48"
				/>
				<Button pixel variant="outline" class="mt-3 w-full sm:col-span-2">Analyze</Button>
			</div>
		</div>

		<ScrollHint />
	</section>
</main>
