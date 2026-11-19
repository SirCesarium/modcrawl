<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children?: Snippet;
		class?: string;
		[key: string]: unknown;
	}

	let { children, class: className = '', ...restProps }: Props = $props();
</script>

<h3
	class="mt-7 mb-2 scroll-mt-20 text-base font-semibold text-neutral-100 sm:text-lg {className}"
	{...restProps}
>
	{#if children}
		{@render children()}
	{/if}
</h3>
