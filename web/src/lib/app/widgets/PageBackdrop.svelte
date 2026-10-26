<script lang="ts">
	import { onMount } from 'svelte';
	import { createStarfield } from '../backdrop/starfield';

	let canvas: HTMLCanvasElement;

	onMount(() => {
		const starfield = createStarfield(canvas);

		return starfield.destroy;
	});
</script>

<div class="pointer-events-none fixed inset-0 z-0 overflow-hidden" aria-hidden="true">
	<canvas bind:this={canvas} class="block h-full w-full [image-rendering:pixelated]"></canvas>

	<div
		class="absolute -inset-24 bg-[url('/noise.png')] bg-size-[128px_128px] bg-repeat opacity-[0.045] [image-rendering:pixelated]"
	></div>
</div>
