<script lang="ts">
	import TextInput from './TextInput.svelte';
	import NumberInput from './NumberInput.svelte';
	import FileInput from './FileInput.svelte';

	interface Props {
		number?: boolean;
		file?: boolean;
		folder?: boolean;
		type?: 'search' | 'text' | 'url' | 'email' | 'password' | 'number' | 'file';
		[key: string]: unknown;
	}

	let { number = false, file = false, folder = false, type, ...restProps }: Props = $props();

	const isNumber = $derived(number || type === 'number');
	const isFile = $derived(file || type === 'file');
	const textType = $derived(type === 'number' || type === 'file' ? undefined : type);
</script>

{#if folder || isFile}
	<FileInput {folder} {...restProps} />
{:else if isNumber}
	<NumberInput {...restProps} />
{:else}
	<TextInput type={textType} {...restProps} />
{/if}
