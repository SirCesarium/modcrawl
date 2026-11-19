<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children?: Snippet;
		class?: string;
		[key: string]: unknown;
	}

	let { children, class: className = '', ...restProps }: Props = $props();
</script>

<h4
	class="mt-5 mb-2 text-sm font-semibold text-neutral-200 sm:text-base {className}"
	{...restProps}
>
	{#if children}
		{@render children()}
	{/if}
</h4>
