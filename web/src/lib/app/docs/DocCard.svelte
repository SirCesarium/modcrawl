<script lang="ts">
	interface Props {
		title: string;
		description: string;
		href?: string;
	}

	let { title, description, href = '#explore' }: Props = $props();
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- DocCard accepts arbitrary external and internal destinations. -->
<a
	{href}
	class="group rounded-md border border-white/10 bg-white/2 p-4 transition-colors hover:border-white/20 hover:bg-white/4 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-300"
>
	<span class="text-sm font-semibold text-neutral-100">{title}</span>
	<span class="mt-1 block text-sm leading-6 text-neutral-400">{description}</span>
</a>
<!-- eslint-enable svelte/no-navigation-without-resolve -->
