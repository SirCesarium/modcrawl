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

<Typography variant="h3" pixel class="mt-5 mb-2 scroll-mt-24 {className}" {...restProps}>
	{#if children}
		{@render children()}
	{/if}
</Typography>
