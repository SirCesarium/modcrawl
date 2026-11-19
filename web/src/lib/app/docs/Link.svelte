<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		href?: string;
		children?: Snippet;
		class?: string;
		[key: string]: unknown;
	}

	let { href = '#', children, class: className = '', ...restProps }: Props = $props();
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- Markdown links can point to arbitrary internal and external destinations. -->
<a
	{href}
	class="font-medium text-purple-300 underline decoration-purple-300/30 underline-offset-4 transition-colors hover:text-purple-200 hover:decoration-purple-300/60 {className}"
	{...restProps}
>
	{#if children}
		{@render children()}
	{/if}
</a>
<!-- eslint-enable svelte/no-navigation-without-resolve -->
