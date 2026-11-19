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

<div class="flex w-full items-center justify-between gap-2 px-4 pt-3 pb-1">
	{#if language}
		<span class="font-mono text-[11px] tracking-wide text-neutral-500">{language}</span>
	{:else}
		<span></span>
	{/if}

	<Button
		variant="ghost"
		color="neutral"
		size="xs"
		class="min-w-16 opacity-100 transition-opacity sm:opacity-0 sm:group-focus-within/code:opacity-100 sm:group-hover/code:opacity-100 {copied
			? 'sm:opacity-100'
			: ''}"
		onclick={copy}
	>
		<span aria-live="polite">{copied ? 'Copied' : 'Copy'}</span>
	</Button>
</div>
