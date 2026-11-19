<script lang="ts">
	import Badge from '#app/components/Badge.svelte';
	import type { LoadedDoc } from './types';

	interface Props {
		doc: LoadedDoc;
	}

	let { doc }: Props = $props();
	const DocComponent = $derived(doc.component);
</script>

<svelte:head>
	<title>{doc.metadata.title} · modcrawl docs</title>
	{#if doc.metadata.description}
		<meta name="description" content={doc.metadata.description} />
	{/if}
</svelte:head>

<article class="min-w-0">
	<header class="mb-8 border-b border-white/[0.07] pb-6">
		<h1
			class="text-[1.75rem] leading-tight font-bold tracking-tight text-neutral-100 sm:text-[2rem]"
		>
			{doc.metadata.title}
		</h1>

		{#if doc.metadata.description}
			<p class="mt-3 text-[15px] leading-7 text-neutral-400 sm:text-base">
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
	</header>

	<DocComponent />
</article>
