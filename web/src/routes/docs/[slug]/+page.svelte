<script lang="ts">
	import Badge from '#app/components/Badge.svelte';
	import Rule from '#app/docs/Rule.svelte';
	import Typography from '#app/components/Typography.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	const doc = $derived(data.doc);
	const DocComponent = $derived(doc.component);
</script>

<svelte:head>
	<title>{doc.metadata.title} - modcrawl docs</title>
	{#if doc.metadata.description}
		<meta name="description" content={doc.metadata.description} />
	{/if}
</svelte:head>

<article class="w-full py-6 sm:py-10">
	<header class="mb-8">
		<Typography variant="h1" pixel class="mb-3 text-3xl sm:text-4xl md:text-5xl">
			{doc.metadata.title}
		</Typography>

		{#if doc.metadata.description}
			<p class="text-base text-neutral-300 sm:text-lg">
				{doc.metadata.description}
			</p>
		{/if}

		{#if doc.metadata.topics && doc.metadata.topics.length > 0}
			<div class="mt-4 flex flex-wrap gap-2">
				{#each doc.metadata.topics as topic (topic)}
					<Badge variant="topic">#{topic}</Badge>
				{/each}
			</div>
		{/if}

		<Rule class="mt-4" />
	</header>

	<div class="min-w-0">
		<DocComponent />
	</div>
</article>
