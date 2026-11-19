<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children?: Snippet;
		class?: string;
		[key: string]: unknown;
	}

	let { children, class: className = '', ...restProps }: Props = $props();
</script>

<h2
	class="mt-10 mb-3 scroll-mt-20 text-xl font-semibold tracking-tight text-neutral-100 sm:text-[1.375rem] {className}"
	{...restProps}
>
	{#if children}
		{@render children()}
	{/if}
</h2>
