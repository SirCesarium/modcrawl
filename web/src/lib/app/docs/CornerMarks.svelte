<script lang="ts">
	interface Props {
		class?: string;
	}

	let { class: className = 'border-purple-100' }: Props = $props();
</script>

<span
	aria-hidden="true"
	class="pointer-events-none absolute -top-1 -left-1 z-10 size-2 scale-75 border-t-2 border-l-2 {className} opacity-0 transition duration-150 group-hover:scale-100 group-hover:opacity-100 group-focus-visible:scale-100 group-focus-visible:opacity-100 motion-reduce:transition-none"
></span>
<span
	aria-hidden="true"
	class="pointer-events-none absolute -top-1 -right-1 z-10 size-2 scale-75 border-t-2 border-r-2 {className} opacity-0 transition duration-150 group-hover:scale-100 group-hover:opacity-100 group-focus-visible:scale-100 group-focus-visible:opacity-100 motion-reduce:transition-none"
></span>
<span
	aria-hidden="true"
	class="pointer-events-none absolute -bottom-1 -left-1 z-10 size-2 scale-75 border-b-2 border-l-2 {className} opacity-0 transition duration-150 group-hover:scale-100 group-hover:opacity-100 group-focus-visible:scale-100 group-focus-visible:opacity-100 motion-reduce:transition-none"
></span>
<span
	aria-hidden="true"
	class="pointer-events-none absolute -right-1 -bottom-1 z-10 size-2 scale-75 border-r-2 border-b-2 {className} opacity-0 transition duration-150 group-hover:scale-100 group-hover:opacity-100 group-focus-visible:scale-100 group-focus-visible:opacity-100 motion-reduce:transition-none"
></span>
