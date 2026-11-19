<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children?: Snippet;
	}

	let { children }: Props = $props();
</script>

<!-- Markdown tables need their own scroll container: a table box cannot scroll
     on its own without collapsing to content width. -->
<div class="my-6 overflow-x-auto rounded-md border border-white/10">
	<table class="w-full text-sm">
		{#if children}
			{@render children()}
		{/if}
	</table>
</div>
