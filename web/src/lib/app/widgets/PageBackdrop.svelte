<script lang="ts">
	import { onMount } from 'svelte';
	import { createStarfield } from '../backdrop/starfield';

	let canvas: HTMLCanvasElement;
	let ready = $state(false);

	onMount(() => {
		const starfield = createStarfield(canvas);
		ready = true;

		return starfield.destroy;
	});
</script>

<div class="pointer-events-none fixed inset-0 z-0 overflow-hidden" aria-hidden="true">
	<canvas
		bind:this={canvas}
		class="block h-full w-full [image-rendering:pixelated] transition-opacity duration-1000 ease-out {ready
			? 'opacity-100'
			: 'opacity-0'}"
	></canvas>

	<div
		class="absolute -inset-24 bg-[url('/noise.png')] bg-size-[128px_128px] bg-repeat opacity-[0.045] [image-rendering:pixelated]"
	></div>
</div>
