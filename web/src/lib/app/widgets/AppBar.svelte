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
	class="fixed inset-x-0 top-0 z-50 flex h-14 items-center justify-between border-b px-4 transition-colors duration-300 sm:h-16 sm:px-8 md:px-10 {showLogo
		? 'border-white/5 bg-neutral-950/80 backdrop-blur-md'
		: 'border-transparent bg-transparent'}"
>
	<div
		class="{showLogo
			? 'translate-x-0 opacity-100'
			: 'pointer-events-none -translate-x-1 opacity-0'} flex min-w-0 shrink items-center gap-1.5 transition-all duration-300 ease-out sm:gap-2"
		aria-hidden={!showLogo}
	>
		<Icon name="modcrawl" size={20} alt="" aria-hidden="true" class="translate-y-0.5 sm:size-6" />
		<Typography
			variant="h4"
			component="span"
			pixel
			uppercase
			class="truncate text-base sm:text-xl md:text-2xl"
		>
			<span class="text-purple-300">mod</span>crawl
		</Typography>
	</div>

	<Button
		href="https://github.com/SirCesarium/modcrawl"
		target="_blank"
		variant="ghost"
		size="sm"
		pixel
		uppercase
		icon="github"
		iconSize={20}
		iconClass="sm:size-6"
		aria-label="GitHub repository"
		class="shrink-0 gap-1.5 px-2.5 text-xs text-neutral-100 sm:h-10 sm:gap-2 sm:px-4 sm:text-base md:text-lg"
	>
		<span class="hidden sm:inline">source</span>
	</Button>
</header>
