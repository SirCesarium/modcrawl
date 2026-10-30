<script lang="ts">
	import './layout.css';
	import PageBackdrop from '#app/widgets/PageBackdrop.svelte';
	import type { LayoutProps } from './$types';
	import { onMount } from 'svelte';

	let { children }: LayoutProps = $props();
	let ready = $state(false);

	onMount(() => {
		if (typeof document !== 'undefined' && 'fonts' in document) {
			const fallbackTimer = setTimeout(() => {
				ready = true;
			}, 350);

			document.fonts.ready.then(() => {
				clearTimeout(fallbackTimer);
				ready = true;
			});
		} else {
			ready = true;
		}
	});
</script>

<svelte:head>
	<link rel="icon" href="/favicon.ico" />
</svelte:head>

<PageBackdrop />

<div
	class="relative z-10 w-full overflow-x-clip transition-opacity duration-500 ease-out {ready
		? 'opacity-100'
		: 'opacity-0'}"
>
	{@render children()}
</div>
