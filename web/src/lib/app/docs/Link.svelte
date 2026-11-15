<script lang="ts">
	import Button from '#app/components/Button.svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		href?: string;
		children?: Snippet;
		class?: string;
		[key: string]: unknown;
	}

	let { href = '#', children, class: className = '', ...restProps }: Props = $props();
</script>

<Button
	variant="link"
	{href}
	class={className}
	{...restProps}
>
	{#if children}
		{@render children()}
	{/if}
</Button>
