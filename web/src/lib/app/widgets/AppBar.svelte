<script lang="ts">
	import Button from '#lib/components/Button.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Typography from '#lib/components/Typography.svelte';

	interface Props {
		showLogo?: boolean;
	}

	let { showLogo = false }: Props = $props();
</script>

<header
	class="fixed top-0 right-0 left-0 z-50 flex h-16 items-center justify-between border-b px-4 transition-colors duration-300 sm:px-10 {showLogo
		? 'border-white/5 bg-neutral-950'
		: 'border-transparent bg-transparent'}"
>
	<div
		class={showLogo
			? 'flex translate-x-0 items-center gap-2 opacity-100 transition-all duration-300 ease-out'
			: 'pointer-events-none flex -translate-x-1 items-center gap-2 opacity-0 transition-all duration-300 ease-out'}
		aria-hidden={!showLogo}
	>
		<Icon name="modcrawl" size={24} alt="" aria-hidden="true" class="translate-y-0.5" />
		<Typography variant="h4" component="span" pixel uppercase>
			<span class="text-purple-300">mod</span>crawl
		</Typography>
	</div>

	<Button
		href="https://github.com/SirCesarium/modcrawl"
		target="_blank"
		variant="ghost"
		size="md"
		pixel
		uppercase
		icon="github"
		iconSize={24}
		class="shrink-0 text-neutral-100 md:text-lg"
	>
		source
	</Button>
</header>
