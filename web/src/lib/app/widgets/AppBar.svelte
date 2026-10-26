<script>
	import Icon from '#lib/components/Icon.svelte';
	import Typography from '#lib/components/Typography.svelte';
</script>

<header class="fixed top-0 right-0 left-0 z-50 flex justify-between px-10 py-4 backdrop-blur-lg">
	<Typography variant="h4" component="span" pixel uppercase
		><span class="text-purple-300">mod</span>crawl</Typography
	>

	<!-- TODO: create a Button component for this -->
	<div class="flex items-center gap-4">
		<Icon name="github" size={24} />
		<Typography variant="h6" bold component="span" pixel uppercase>source</Typography>
	</div>
</header>
