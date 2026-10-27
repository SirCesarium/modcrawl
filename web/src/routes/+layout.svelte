<script lang="ts">
	import './layout.css';
	import PageBackdrop from '#lib/app/widgets/PageBackdrop.svelte';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();
</script>

<svelte:head>
	<link rel="icon" href="/favicon.ico" />
</svelte:head>

<PageBackdrop />

<div class="relative z-10 w-full overflow-x-clip">
	{@render children()}
</div>
