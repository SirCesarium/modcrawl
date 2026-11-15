<script lang="ts">
	import Typography from '#app/components/Typography.svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		children?: Snippet;
		class?: string;
		[key: string]: unknown;
	}

	let { children, class: className = '', ...restProps }: Props = $props();
</script>

<Typography variant="h2" pixel class="mt-6 mb-3 scroll-mt-24 {className}" {...restProps}>
	{#if children}
		{@render children()}
	{/if}
</Typography>
