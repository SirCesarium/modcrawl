<script lang="ts">
	import { twMerge } from 'tailwind-merge';

	interface Props {
		name: string;
		size?: number | string;
		class?: string;
		[key: string]: unknown;
	}

	let { name, size = 16, class: className = '', ...restProps }: Props = $props();

	const computedClass = $derived(
		twMerge('inline-block shrink-0 select-none [image-rendering:pixelated]', className)
	);
</script>

<img
	src={`/icons/${name}.png`}
	alt={name}
	width={size}
	height={size}
	draggable={false}
	class={computedClass}
	{...restProps}
/>
