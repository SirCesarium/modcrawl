<script lang="ts">
	import Button from '#app/components/Button.svelte';

	interface Props {
		language?: string;
	}

	let { language = '' }: Props = $props();

	let copied = $state(false);
	let resetTimer: ReturnType<typeof setTimeout> | undefined;

	const copy = async (event: MouseEvent) => {
		const block = (event.currentTarget as HTMLElement).closest('[data-code-block]');
		const code = block?.querySelector('code');
		if (!code) return;

		try {
			await navigator.clipboard.writeText(code.textContent ?? '');
			copied = true;
			clearTimeout(resetTimer);
			resetTimer = setTimeout(() => {
				copied = false;
			}, 1500);
		} catch {
			copied = false;
		}
	};
</script>

<div class="flex w-full items-center justify-between gap-2 px-3 pt-3 sm:px-4">
	{#if language}
		<span class="font-pixel text-[11px] tracking-wider uppercase opacity-70">{language}</span>
	{/if}
	<Button variant="outline" size="xs" class="min-w-18" onclick={copy}>
		{copied ? 'Copied' : 'Copy'}
	</Button>
</div>
