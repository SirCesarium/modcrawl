<script lang="ts">
	import DocArticle from '#app/docs/DocArticle.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<DocArticle doc={data.doc} />
