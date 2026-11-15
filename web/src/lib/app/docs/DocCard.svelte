<script lang="ts">
	import CornerMarks from './CornerMarks.svelte';

	interface Props {
		title: string;
		description: string;
		href?: string;
	}

	let { title, description, href = '#explore' }: Props = $props();
</script>

<a
	{href}
	class="group relative top-0 border border-white/10 p-4 transition-[top,background-color,border-color] duration-150 ease-out hover:-top-0.5 hover:border-solid hover:border-purple-300/60 hover:bg-purple-500/5 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-300 active:translate-x-0.75 active:translate-y-0.75 active:scale-[0.98] motion-reduce:transform-none motion-reduce:transition-none"
>
	<CornerMarks />

	<span class="font-pixel text-sm tracking-wide text-purple-300 uppercase">{title}</span>
	<span class="mt-1 block text-sm text-neutral-400">{description}</span>
</a>
