<script lang="ts">
	import { onMount } from 'svelte';
	import { createPortalLogo } from '../logo/scene';

	const CODE_LINES = [
		{ src: '/code/code-1.png', width: 'w-14' },
		{ src: '/code/code-2.png', width: 'w-12' },
		{ src: '/code/code-3.png', width: 'w-10' }
	];

	let container: HTMLDivElement;
	let canvas: HTMLCanvasElement;
	let converted = $state(false);

	onMount(() => {
		const portal = createPortalLogo({
			canvas,
			container,
			onAbsorb: () => (converted = true),
			onReset: () => (converted = false)
		});

		return portal.destroy;
	});
</script>

<div class="relative isolate h-32 w-32" aria-hidden="true">
	<div bind:this={container} class="h-full w-full">
		<canvas bind:this={canvas} class="block h-full w-full"></canvas>
	</div>

	<div class="pointer-events-none absolute inset-0 -z-10">
		<div class="absolute top-1/2 -right-3 flex -translate-y-1/2 flex-col items-end gap-1.5">
			{#each CODE_LINES as line (line.src)}
				<img
					src={line.src}
					alt=""
					draggable="false"
					class="block {line.width} drop-shadow-[0_0_10px_#b35cff59] transition-opacity duration-500 ease-out [image-rendering:pixelated] motion-reduce:animate-none motion-reduce:transition-none {converted
						? 'animate-code-line opacity-100'
						: 'opacity-0'}"
				/>
			{/each}
		</div>
	</div>
</div>
