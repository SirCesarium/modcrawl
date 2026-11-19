<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children?: Snippet;
		class?: string;
		[key: string]: unknown;
	}

	let { children, class: className = '', ...restProps }: Props = $props();
</script>

<h1
	class="mt-10 mb-3 text-2xl font-semibold tracking-tight text-neutral-100 sm:text-3xl {className}"
	{...restProps}
>
	{#if children}
		{@render children()}
	{/if}
</h1>
