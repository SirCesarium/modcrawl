<script lang="ts">
	import Icon from '#lib/components/Icon.svelte';

	const delays = [0, 200];
</script>

<div
	class="pointer-events-none absolute inset-x-0 bottom-5 flex flex-col items-center sm:bottom-6"
	aria-hidden="true"
>
	{#each delays as delay (delay)}
		<Icon
			name="chevron"
			size={32}
			class="animate-scroll-chevron opacity-20 motion-reduce:animate-none motion-reduce:opacity-60"
			style="animation-delay: {delay}ms"
		/>
	{/each}
</div>
